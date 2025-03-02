use std::sync::{atomic::AtomicI32, Arc};

use futures::FutureExt;
use log::{debug, info};
use space_traders_client::models;

use crate::{
    config::CONFIG,
    error::Result,
    manager::mining_manager::{ActionType, ExtractorTransferRequest, TransferResult},
    ship,
    sql::{self, DatabaseConnector, TransactionReason},
    types::{safely_get_map, ConductorContext},
};

pub struct ExtractionPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl ExtractionPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    pub async fn execute_extraction_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        is_syphon: bool,
    ) -> Result<()> {
        debug!("Executing extraction circle for ship: {}", ship.symbol);
        let waypoint_symbol = self
            .context
            .mining_manager
            .get_waypoint(&ship, is_syphon)
            .await?;

        debug!("Mining Waypoint: {}", waypoint_symbol);

        self.go_to_waypoint(ship, &waypoint_symbol).await?;

        self.context.mining_manager.notify_waypoint(ship).await?;
        let mut rec = self
            .context
            .mining_manager
            .extractor_contact(&ship.symbol)
            .await?;

        let i = self.wait_for_extraction(ship, pilot, &mut rec).await?;

        if i == 0 {
            debug!("No extraction for ship: {}", ship.symbol);
            self.context.mining_manager.unassign_waypoint(ship).await?;
            return Ok(());
        }

        let done = if !self.has_space(ship) {
            debug!("No space on ship: {}", ship.symbol);
            let pin_sleep = tokio::time::sleep(std::time::Duration::from_millis(
                1000 + rand::random::<u64>() % 10000,
            ));
            let pin_sleep_pined = std::pin::pin!(pin_sleep);

            let i = self
                .wait_for(ship, pilot, &mut rec, pin_sleep_pined)
                .await?;
            if i == 0 {
                debug!("No extraction after waiting for ship: {}", ship.symbol);
                self.context.mining_manager.unassign_waypoint(ship).await?;
                return Ok(());
            }
            0
        } else {
            self.extract(ship, is_syphon).await?;
            self.eject_blacklist(ship).await?;

            1
        };

        if done == 1 || rand::random::<u64>() % 10 == 0 {
            self.context
                .mining_manager
                .extraction_complete(&ship.symbol, &ship.nav.waypoint_symbol)
                .await?;
        }

        let _i = self.wait_for_extraction(ship, pilot, &mut rec).await?;

        debug!("Dropping extractor contact for ship: {}", ship.symbol);

        drop(rec);

        self.context.mining_manager.unassign_waypoint(ship).await?;

        debug!("Extraction circle complete for ship: {}", ship.symbol);
        Ok(())
    }

    async fn go_to_waypoint(&self, ship: &mut ship::MyShip, waypoint_symbol: &str) -> Result<()> {
        debug!("Going to waypoint: {}", waypoint_symbol);
        if ship.nav.waypoint_symbol == waypoint_symbol {
            debug!("Already at waypoint: {}", waypoint_symbol);
            return Ok(());
        }

        let system_symbol = ship.nav.system_symbol.clone();

        if !waypoint_symbol.contains(&system_symbol) {
            return Err("Waypoint is not in ship's system".into());
        }

        let waypoints = safely_get_map(&self.context.all_waypoints, &system_symbol)
            .unwrap()
            .clone();

        ship.nav_to(
            &waypoint_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            TransactionReason::MiningWaypoint(waypoint_symbol.to_string()),
        )
        .await?;

        debug!("Arrived at waypoint: {}", waypoint_symbol);
        Ok(())
    }

    fn has_space(&self, ship: &mut ship::MyShip) -> bool {
        debug!(
            "Checking space on ship: {} can store: {} units has: {}",
            ship.symbol, ship.cargo.capacity, ship.cargo.units
        );
        ship.cargo.units < ship.cargo.capacity
    }

    async fn extract(&self, ship: &mut ship::MyShip, is_syphon: bool) -> Result<()> {
        debug!("Extracting on ship: {}", ship.symbol);
        if ship.is_on_cooldown() {
            debug!("Ship is on cooldown: {}", ship.symbol);
            return Err("Ship is on cooldown".into());
        }

        ship.ensure_undocked(&self.context.api).await?;

        let action = if is_syphon {
            ActionType::Siphon
        } else {
            ActionType::Extract
        };
        // let action = ActionType::get_action(&ship).ok_or("Invalid ship role")?;

        match action {
            ActionType::Extract => {
                let erg = ship.extract(&self.context.api).await;
                match erg {
                    Err(space_traders_client::apis::Error::ResponseError(e)) => {
                        if e.entity
                            .as_ref()
                            .map(|e| {
                                e.error.code == models::error_codes::SHIP_EXTRACT_DESTABILIZED_ERROR
                            })
                            .unwrap_or(false)
                        {
                            log::warn!(
                                "Waypoint {} is destabilized by {}",
                                ship.nav.waypoint_symbol,
                                ship.symbol
                            );

                            let new_wp = sql::Waypoint::get_by_symbol(
                                &self.context.database_pool,
                                &ship.nav.waypoint_symbol,
                            )
                            .await?;
                            let mut wp = if let Some(new_wp) = new_wp {
                                new_wp
                            } else {
                                let new_wp = self
                                    .context
                                    .api
                                    .get_waypoint(
                                        &ship.nav.system_symbol,
                                        &ship.nav.waypoint_symbol,
                                    )
                                    .await?;
                                (&(*new_wp.data)).into()
                            };
                            wp.unstable_since = Some(chrono::Utc::now().naive_local());
                            sql::Waypoint::insert(&self.context.database_pool, &wp).await?;
                        } else {
                            return Err(space_traders_client::apis::Error::ResponseError(e).into());
                        }
                    }
                    Err(e) => return Err(e.into()),
                    Ok(erg) => {
                        info!(
                            "Extracted on ship: {} erg {:?} events: {:?}",
                            erg.data.extraction.ship_symbol,
                            erg.data.extraction.r#yield,
                            erg.data.events
                        );
                    }
                }
            }
            ActionType::Siphon => {
                let erg = ship.siphon(&self.context.api).await?;
                info!(
                    "Siphoned on ship: {} erg {:?} events: {:?}",
                    erg.data.siphon.ship_symbol, erg.data.siphon.r#yield, erg.data.events
                );
            }
        }

        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!(
            "Count after extraction: {}",
            self.count.load(std::sync::atomic::Ordering::Relaxed)
        );

        ship.notify().await;

        debug!("Extracted on ship: {}", ship.symbol);
        Ok(())
    }

    async fn eject_blacklist(&self, ship: &mut ship::MyShip) -> Result<()> {
        debug!("Ejecting blacklist on ship: {}", ship.symbol);
        let cargo = ship.cargo.inventory.clone();
        for item in cargo.iter() {
            if CONFIG.mining.blacklist.contains(&item.0) {
                debug!("Ejecting: {:?}", item);
                ship.jettison(&self.context.api, *item.0, *item.1).await?;
            }
        }
        Ok(())
    }

    async fn wait_for_extraction(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        receiver: &mut tokio::sync::mpsc::Receiver<ExtractorTransferRequest>,
    ) -> Result<i32> {
        let ship_future = ship.wait_for_cooldown();
        let ship_future_pined = std::pin::pin!(ship_future);
        let erg = self
            .wait_for(ship, pilot, receiver, ship_future_pined)
            .await?;
        Ok(erg)
    }

    async fn wait_for(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        receiver: &mut tokio::sync::mpsc::Receiver<ExtractorTransferRequest>,
        future: impl std::future::Future<Output = ()> + Unpin,
    ) -> Result<i32> {
        //needs revisit

        // tell mining manager you can transfer your cargo
        // wait until cooldown is done
        //    in meantime, listen to mining manager and transfer cargo it tells you
        // cut connection to mining manager

        let mut fused_future = future.fuse();

        loop {
            let i = {
                tokio::select! {
                    _ = pilot.cancellation_token.cancelled() => {(None,0)},// it's the end of the Programm we don't care(for now)
                    _ = &mut fused_future => {(None,1)},
                    msg = receiver.recv() => {(msg,2)},
                }
            };

            match i {
                (Some(msg), _) => {
                    self.handle_extractor_transfer_request(ship, msg).await?;
                }
                (None, 1) => {
                    debug!("Cooldown is done for ship: {}", ship.symbol);
                    return Ok(1);
                }
                (None, 2) => {
                    debug!("No more messages for ship: {}", ship.symbol);
                    return Ok(2);
                }
                (None, _) => {
                    debug!("No more messages for ship: {}", ship.symbol);
                    return Ok(0);
                }
            }
        }
    }

    async fn handle_extractor_transfer_request(
        &self,
        ship: &mut ship::MyShip,
        request: ExtractorTransferRequest,
    ) -> Result<()> {
        debug!("Handling transfer request for ship: {}", ship.symbol);
        let erg = ship
            .simple_transfer_cargo(
                request.trade_symbol,
                request.amount,
                &self.context.api,
                &request.to_symbol,
            )
            .await;

        let transfer_result = match erg {
            Ok(_erg) => Some(TransferResult {
                trade_symbol: request.trade_symbol,
                units: request.amount,
            }),
            Err(_) => None,
        };

        let _erg = request.callback.send(transfer_result);

        debug!("Handled transfer request for ship: {}", ship.symbol);
        Ok(())
    }
}
