use std::{collections::HashMap, time::Duration, vec};

use database::{DatabaseConnectorAsync, DbPool};
use tracing::debug;
use tracing::Instrument;
use utils::{distance_between_waypoints, WaypointCan};

use crate::{
    error::{Error, Result},
    manager::{scrapping_manager::priority_calculator, Manager},
    utils::ConductorContext,
};

use super::{message::ScrappingManagerMessage, messanger::ScrappingManagerMessanger};

pub type ScrappingManagerReceiver = tokio::sync::mpsc::Receiver<ScrappingManagerMessage>;

#[derive(Debug)]
pub struct ScrappingManager {
    fast_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: ScrappingManagerReceiver,
    scrap_waypoints: HashMap<String, String>,
}

impl ScrappingManager {
    pub fn create() -> (ScrappingManagerReceiver, ScrappingManagerMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ScrappingManagerMessanger::new(sender))
    }

    pub fn new(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: ScrappingManagerReceiver,
    ) -> Self {
        Self {
            fast_cancel_token,
            slow_cancel_token,
            context,
            receiver,
            scrap_waypoints: HashMap::new(),
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::scrapping_manager::scrapping_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_scrapping_worker(&mut self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis({
            self.context.config.read().await.scrapper_start_sleep
        }))
        .await;

        let (agent_join_handle, system_join_handle) = self.init_other_scrappers().await?;

        while !self.slow_cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.slow_cancel_token.cancelled() => {
                    tracing::info!("ScrappingManager slow cancel token triggered");
                    None
                }
            };
            debug!(
                "Received scrappingManager message: {:?}",
                message.as_ref().map(|m| m.to_string())
            );

            match message {
                Some(message) => {
                    self.handle_scrap_message(message).await?;
                }
                None => break,
            }
        }

        let agent_errs = agent_join_handle.await;
        let system_errs = system_join_handle.await;

        match agent_errs {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => tracing::error!(err = ?err, "Failed to update agents"),
            Err(err) => tracing::error!(err = ?err, "JoinFailed to update agents"),
        }

        match system_errs {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => tracing::error!(err = ?err, "Failed to update systems"),
            Err(err) => tracing::error!(err = ?err, "JoinFailed to update systems"),
        }

        Ok(())
    }

    async fn init_other_scrappers(
        &self,
    ) -> Result<(
        tokio::task::JoinHandle<std::result::Result<(), crate::error::Error>>,
        tokio::task::JoinHandle<std::result::Result<(), crate::error::Error>>,
    )> {
        let erg = { self.context.config.read().await.scrap_agents };
        let agent_join_handle = if erg {
            let api = self.context.api.clone();
            let database_pool = self.context.database_pool.clone();
            let slow_cancel_token = self.slow_cancel_token.child_token();
            let fast_cancel_token = self.fast_cancel_token.child_token();
            let interval = 1000 * 60 * 60;
            tokio::spawn(async move {
                let _erg = tokio::select! {
                  _ = fast_cancel_token.cancelled() => {
                    tracing::info!("ScrappingManager agent worker fast cancel token triggered");
                    Ok(())
                  },
                  erg = Self::run_agent_worker(&api, &database_pool, slow_cancel_token, interval) => erg,
                };
                Ok(())
            })
        } else {
            tokio::spawn(async move { Ok(()) })
        };

        let erg = { self.context.config.read().await.update_all_systems };
        let system_join_handle: tokio::task::JoinHandle<
            std::result::Result<(), crate::error::Error>,
        > = if erg {
            let api = self.context.api.clone();
            let database_pool = self.context.database_pool.clone();
            let fast_cancel_token = self.fast_cancel_token.child_token();

            tokio::spawn(
                async move {
                    let _erg = tokio::select! {
                      _ = fast_cancel_token.cancelled() => {
                        tracing::info!("ScrappingManager system worker fast cancel token triggered");
                        Ok(())
                      },
                      erg = Self::run_system_worker(&api, &database_pool) => erg,
                    }?;

                    Ok(())
                }
                .instrument(tracing::info_span!(
                    "spacetraders::manager::scrapping_update_systems"
                )),
            )
        } else {
            tokio::spawn(async move { Ok(()) })
        };

        Ok((agent_join_handle, system_join_handle))
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::scrapping_agent_worker",
        skip(api, database_pool, slow_cancel_token)
    )]
    async fn run_agent_worker(
        api: &space_traders_client::Api,
        database_pool: &DbPool,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        interval: u64,
    ) -> Result<()> {
        while !slow_cancel_token.is_cancelled() {
            tokio::time::sleep(Duration::from_millis(interval)).await;
            super::utils::update_all_agents(api, database_pool).await?;
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::scrapping_manager::scrapping_manager_handle_scrap_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_scrap_message(&mut self, message: super::message::ScrapMessage) -> Result<()> {
        self.context.scrapping_manager.set_busy(true);

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
        self.context.scrapping_manager.set_busy(false);

        Ok(())
    }

    async fn complete_scrapping(
        &mut self,
        ship_clone: ship::MyShip,
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
        ship_clone: ship::MyShip,
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
        ship_clone: ship::MyShip,
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
        ship_clone: &ship::MyShip,
    ) -> Result<Vec<(database::Waypoint, chrono::DateTime<chrono::Utc>)>> {
        let system_symbol = ship_clone.nav.system_symbol.clone();

        let system_wps = database::Waypoint::get_by_system(
            &self.context.database_pool,
            &system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

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

        let mut waypoints: Vec<(database::Waypoint, chrono::DateTime<chrono::Utc>)> = vec![];

        for wp in wps {
            let market_trade_goods = database::MarketTradeGood::get_last_by_waypoint(
                &self.context.database_pool,
                &wp.symbol,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;

            if !wp.is_charted() || market_trade_goods.is_empty() {
                waypoints.push((wp, chrono::DateTime::<chrono::Utc>::MIN_UTC));
                continue;
            }

            let max_update_interval = { self.context.config.read().await.max_update_interval };

            let next_time = priority_calculator::get_waypoint_time(
                market_trade_goods
                    .into_iter()
                    .map(From::from)
                    .collect::<Vec<_>>()
                    .as_slice(),
                max_update_interval,
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

    async fn run_system_worker(
        api: &space_traders_client::Api,
        database_pool: &DbPool,
    ) -> Result<()> {
        crate::manager::scrapping_manager::utils::update_all_systems(&database_pool, &api).await?;
        let gates =
            database::Waypoint::get_all(&database_pool, database::PaginatedQuery::unpaged())
                .await?
                .items
                .into_iter()
                .filter(|w| w.is_jump_gate())
                .filter(|w| w.is_charted())
                .map(|w| {
                    let chart = w.is_charted();
                    (w.system_symbol, w.symbol, chart)
                })
                .collect::<Vec<_>>();
        let jump_gates =
            crate::manager::scrapping_manager::utils::get_all_jump_gates(&api, gates).await?;

        let jump_gates_len = jump_gates.len();
        crate::manager::scrapping_manager::utils::update_jump_gates(&database_pool, jump_gates)
            .await?;
        debug!("Updated jump gates {}", jump_gates_len);

        Ok(())
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
        &self.slow_cancel_token
    }
}
