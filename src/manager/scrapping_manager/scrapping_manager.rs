use std::{
    collections::{HashMap, HashSet},
    time::Duration,
    vec,
};

use log::{debug, error, info};

use crate::{
    config::CONFIG,
    error::{Error, Result},
    manager::{scrapping_manager::priority_calculator, Manager},
    sql::{self, DbPool},
    types::{ConductorContext, WaypointCan},
    utils::distance_between_waypoints,
};

use super::{message::ScrappingManagerMessage, messanger::ScrappingManagerMessanger};

#[derive(Debug)]
pub struct ScrappingManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
    scrap_waypoints: HashMap<String, String>,
    max_update_interval: i64, // in seconds
}

impl ScrappingManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
        ScrappingManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ScrappingManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
            scrap_waypoints: HashMap::new(),
            // max_update_interval: 60 * 10,
            // max_update_interval: 60 * 25,
            max_update_interval: 60 * 20,
        }
    }

    async fn run_scrapping_worker(&mut self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(CONFIG.market.start_sleep_duration)).await;

        let agent_join_handle = if CONFIG.market.agents {
            let api = self.context.api.clone();
            let database_pool = self.context.database_pool.clone();
            let cancel_token = self.cancel_token.child_token();
            let interval = 1000 * 60 * 60;
            tokio::spawn(async move {
                Self::run_agent_worker(&api, &database_pool, cancel_token, interval).await
            })
        } else {
            tokio::spawn(async move { Ok(()) })
        };

        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            debug!("Received scrappingManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_scrap_message(message).await?;
                }
                None => break,
            }
        }

        let agent_errs = agent_join_handle.await;

        match agent_errs {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => error!("Failed to update agents: {}", err),
            Err(err) => error!("JoinFailed to update agents: {}", err),
        }

        info!("ScrappingManager done");

        Ok(())
    }

    async fn run_agent_worker(
        api: &crate::api::Api,
        database_pool: &DbPool,
        cancel_token: tokio_util::sync::CancellationToken,
        interval: u64,
    ) -> Result<()> {
        while !cancel_token.is_cancelled() {
            tokio::time::sleep(Duration::from_millis(interval)).await;
            super::utils::update_all_agents(api, database_pool).await?;
        }

        Ok(())
    }

    pub async fn get_system(&self) -> Vec<String> {
        let systems = self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .iter()
            .map(|s| s.1.nav.system_symbol.clone())
            .collect::<HashSet<_>>();
        systems.into_iter().collect()
    }

    async fn handle_scrap_message(&mut self, message: super::message::ScrapMessage) -> Result<()> {
        match message {
            super::message::ScrapMessage::Next {
                ship_clone,
                callback,
            } => {
                let next_resp = self.next_scrapping(ship_clone).await?;
                callback.send(next_resp).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
            super::message::ScrapMessage::Complete {
                ship_clone,
                waypoint_symbol,
            } => self.complete_scrapping(ship_clone, waypoint_symbol).await?,
            super::message::ScrapMessage::Fail {
                ship_clone,
                waypoint_symbol,
            } => self.fail_scrapping(ship_clone, waypoint_symbol).await?,
            super::message::ScrapMessage::GetAll {
                ship_clone,
                callback,
            } => {
                let resp = self.get_all_sorted(&ship_clone).await?;
                callback
                    .send(resp.iter().map(|s| (s.0.symbol.clone(), s.1)).collect())
                    .map_err(|e| {
                        crate::error::Error::General(format!("Failed to send message: {:?}", e))
                    })?
            }
        }

        Ok(())
    }

    async fn complete_scrapping(
        &mut self,
        ship_clone: crate::ship::MyShip,
        waypoint_symbol: String,
    ) -> Result<()> {
        let ship_symbol = self.scrap_waypoints.get(&waypoint_symbol);

        if let Some(ship_symbol) = ship_symbol {
            if ship_symbol == &ship_clone.symbol {
                self.scrap_waypoints.remove(&waypoint_symbol);
            }
        }

        Ok(())
    }

    async fn fail_scrapping(
        &mut self,
        ship_clone: crate::ship::MyShip,
        waypoint_symbol: String,
    ) -> Result<()> {
        let ship_symbol = self.scrap_waypoints.get(&waypoint_symbol);

        if let Some(ship_symbol) = ship_symbol {
            if ship_symbol == &ship_clone.symbol {
                self.scrap_waypoints.remove(&waypoint_symbol);
            }
        }

        Ok(())
    }

    async fn next_scrapping(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> Result<super::message::ScrapResponse> {
        let waypoints = self.get_all_sorted(&ship_clone).await?;

        if let Some((wp, date)) = waypoints.first() {
            self.scrap_waypoints
                .insert(wp.symbol.clone(), ship_clone.symbol.clone());

            Ok(super::message::ScrapResponse::Scrapping {
                waypoint_symbol: wp.symbol.clone(),
                date: *date,
            })
        } else {
            Ok(super::message::ScrapResponse::Unassigned)
        }
    }

    async fn get_all_sorted(
        &mut self,
        ship_clone: &crate::ship::MyShip,
    ) -> Result<Vec<(sql::Waypoint, chrono::DateTime<chrono::Utc>)>> {
        let system_symbol = ship_clone.nav.system_symbol.clone();

        let system_wps =
            sql::Waypoint::get_by_system(&self.context.database_pool, &system_symbol).await?;

        let ship_wp = system_wps
            .iter()
            .find(|w| w.symbol == ship_clone.nav.waypoint_symbol)
            .ok_or(Error::General("Waypoint not found".to_string()))?
            .clone();

        let wps = system_wps
            .into_iter()
            .filter(|w| w.is_marketplace())
            .filter(|w| !self.scrap_waypoints.contains_key(&w.symbol))
            .collect::<Vec<_>>();

        let mut waypoints: Vec<(sql::Waypoint, chrono::DateTime<chrono::Utc>)> = vec![];

        for wp in wps {
            let market_trade_goods =
                sql::MarketTradeGood::get_last_by_waypoint(&self.context.database_pool, &wp.symbol)
                    .await?;

            if !wp.is_charted() || market_trade_goods.is_empty() {
                waypoints.push((wp, chrono::DateTime::<chrono::Utc>::MIN_UTC));
                continue;
            }

            let next_time = priority_calculator::get_waypoint_time(
                market_trade_goods
                    .into_iter()
                    .map(From::from)
                    .collect::<Vec<_>>()
                    .as_slice(),
                self.max_update_interval,
            )?;

            waypoints.push((wp, next_time));
        }

        waypoints.sort_by(|a, b| a.1.cmp(&b.1));

        waypoints.sort_by(|a, b| {
            // the first waypoint is the closest
            if a.0.symbol == b.0.symbol {
                return std::cmp::Ordering::Equal;
            }
            if a.0.symbol == ship_wp.symbol {
                return std::cmp::Ordering::Less;
            }
            if b.0.symbol == ship_wp.symbol {
                return std::cmp::Ordering::Greater;
            }
            (distance_between_waypoints((&a.0).into(), (&ship_wp).into()) as i32)
                .cmp(&(distance_between_waypoints((&b.0).into(), (&ship_wp).into()) as i32))
        });

        let mut past_waypoints = Vec::new();
        let mut future_waypoints = Vec::new();

        for (wp, time) in waypoints {
            if time < chrono::Utc::now() {
                past_waypoints.push((wp, time));
            } else {
                future_waypoints.push((wp, time));
            }
        }

        let mut waypoints = Vec::new();
        waypoints.extend(past_waypoints.into_iter());
        waypoints.extend(future_waypoints.into_iter());

        Ok(waypoints)
    }
}

impl Manager for ScrappingManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_scrapping_worker().await })
    }

    fn get_name(&self) -> &str {
        "ScrappingManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
