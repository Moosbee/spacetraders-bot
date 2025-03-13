use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc},
};

use log::{debug, info, warn};
use space_traders_client::models;

use crate::{
    error::Result,
    manager::mining_manager::{ExtractorTransferRequest, TransportTransferRequest},
    ship,
    sql::{self, TransactionReason},
    types::ConductorContext,
};

use super::{MiningShipAssignment, TransporterState};

pub struct TransportPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl TransportPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    pub async fn execute_transport_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        let mut last_waypoint = ship.nav.waypoint_symbol.clone();

        ship.status = ship::ShipStatus::Mining {
            assignment: MiningShipAssignment::Transporter {
                state: TransporterState::Unknown,
                waypoint_symbol: None,
                cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            },
        };
        ship.notify().await;

        while !(ship.cargo.get_units_no_fuel() as f32
            / (ship.cargo.capacity
                - ship
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel))
                as f32
            > 0.95)
        {
            let next_mining_waypoint = self.get_next_mining_waypoint(ship).await;
            debug!("Next transport mining waypoint: {:?}", next_mining_waypoint);
            if next_mining_waypoint.is_err() {
                let next_err = next_mining_waypoint.unwrap_err();
                if let crate::error::Error::General(err_r) = &next_err {
                    if err_r == "No routes found" {
                        info!("No more mining waypoints");
                        tokio::time::sleep(std::time::Duration::from_millis(
                            1000 + rand::random::<u64>() % 500,
                        ))
                        .await;
                        break;
                    }
                }
                return Err(next_err);
            }

            let next_mining_waypoint = next_mining_waypoint.unwrap();

            last_waypoint = next_mining_waypoint.clone();

            ship.status = ship::ShipStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::InTransitToAsteroid,
                    waypoint_symbol: Some(next_mining_waypoint.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            ship.nav_to_prepare(
                &next_mining_waypoint,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::MiningWaypoint(next_mining_waypoint.clone()),
                true,
            )
            .await?;

            debug!("Navigated to waypoint: {}", next_mining_waypoint);

            self.handle_cargo_loading(ship, pilot).await?;
        }
        self.sell_all_cargo(pilot, ship, &waypoints, last_waypoint)
            .await?;

        ship.status = ship::ShipStatus::Mining {
            assignment: MiningShipAssignment::Transporter {
                state: TransporterState::Unknown,
                waypoint_symbol: None,
                cycles: None,
            },
        };
        ship.notify().await;

        Ok(())
    }

    async fn get_next_mining_waypoint(&self, ship: &mut ship::MyShip) -> Result<String> {
        let next_transport = self.context.mining_manager.get_next_transport(ship).await?;

        debug!("Next transport mining waypoint: {}", next_transport);

        Ok(next_transport)
    }

    async fn handle_cargo_loading(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        debug!("Initiating cargo loading for ship: {}", ship.symbol);
        // tell mining manager you have arrived
        // wait until storage is full or are told to leave
        //    in meantime, listen to mining manager and load cargo it tells you
        // cut connection to mining manager

        let mut rec = self
            .context
            .mining_manager
            .transport_contact(&ship.symbol)
            .await?;
        debug!("Transport contact established for ship: {}", ship.symbol);

        let _erg = self
            .context
            .mining_manager
            .transport_arrived(&ship.symbol, &ship.nav.waypoint_symbol)
            .await?;
        debug!(
            "Transport arrived notification sent for ship: {}, waypoint: {}",
            ship.symbol, ship.nav.waypoint_symbol
        );

        while !(ship.cargo.get_units_no_fuel() as f32
            / (ship.cargo.capacity
                - ship
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel))
                as f32
            > 0.95)
        {
            ship.status = ship::ShipStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::WaitingForCargo,
                    waypoint_symbol: Some(ship.nav.waypoint_symbol.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            let msg = tokio::select! {
                _ = pilot.cancellation_token.cancelled() => {
                    debug!("Cancellation token received for ship: {}", ship.symbol);
                    None
                },
                msg = rec.recv() => msg,
            };

            ship.status = ship::ShipStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::LoadingCargo,
                    waypoint_symbol: Some(ship.nav.waypoint_symbol.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            match msg {
                None => {
                    debug!(
                        "No more messages; shutting down transport for ship: {}",
                        ship.symbol
                    );
                    break;
                }
                Some(transfer_request) => {
                    debug!("Received transfer request for ship: {}", ship.symbol);
                    let erg = self.handle_transfer_request(ship, transfer_request).await;
                    if let Err(err) = erg {
                        log::error!(
                            "Failed to handle transfer request: {} and also {:?} on ship: {}",
                            err,
                            err,
                            ship.symbol
                        );
                        return Err(err);
                    }
                }
            }
        }

        debug!(
            "Finalizing cargo loading; shutting down transport for ship: {}",
            ship.symbol
        );
        drop(rec);

        Ok(())
    }

    async fn handle_transfer_request(
        &self,
        ship: &mut ship::MyShip,
        request: TransportTransferRequest,
    ) -> Result<()> {
        let (callback, receiver) = tokio::sync::oneshot::channel();

        let extractor_req = ExtractorTransferRequest {
            from_symbol: request.from_symbol.clone(),
            to_symbol: ship.symbol.clone(),
            amount: request.amount,
            trade_symbol: request.trade_symbol,
            callback,
        };

        let erg = request.extractor_contact.send(extractor_req).await;

        if let Err(err) = erg {
            // the mining ship had dropped
            log::error!("Failed to send message to extractor: {}", err);
            let _erg = request.callback.send(());

            return Ok(());
        }

        let transfer = receiver.await;

        let transfer = match transfer {
            Ok(transfer) => transfer,
            Err(e) => {
                log::error!("Failed to receive message from extractor: {}", e);
                return Ok(());
            }
        };

        let transfer = if let Some(transfer) = transfer {
            transfer
        } else {
            return Ok(());
        };

        ship.cargo
            .handle_cago_update(transfer.units, transfer.trade_symbol)?;

        ship.notify().await;

        let _erg = request.callback.send(());

        Ok(())
    }

    async fn sell_all_cargo(
        &self,
        pilot: &crate::pilot::Pilot,
        ship: &mut ship::MyShip,
        waypoints: &std::collections::HashMap<String, space_traders_client::models::Waypoint>,
        mining_waypoint: String,
    ) -> Result<()> {
        while ship.cargo.get_units_no_fuel() > 0 {
            if pilot.cancellation_token.is_cancelled() {
                info!("Transport cycle cancelled for {} ", ship.symbol);
                break;
            }

            ship.status = ship::ShipStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::InTransitToMarket,
                    waypoint_symbol: Some(mining_waypoint.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;
            let (next_waypoint, trade_symbols) =
                self.get_next_best_sell_waypoint(ship).await.unwrap();
            ship.nav_to(
                &next_waypoint,
                true,
                waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::MiningWaypoint(mining_waypoint.clone()),
            )
            .await?;

            ship.status = ship::ShipStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::SellingCargo,
                    waypoint_symbol: Some(mining_waypoint.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            self.handle_cargo_selling(
                ship,
                &self.context.api,
                &self.context.database_pool,
                TransactionReason::MiningWaypoint(mining_waypoint.clone()),
                trade_symbols,
            )
            .await?;
        }

        Ok(())
    }

    async fn get_next_best_sell_waypoint(
        &self,
        ship: &ship::MyShip,
    ) -> Option<(String, Vec<models::TradeSymbol>)> {
        let cargo_data = &ship.cargo;

        let all_trades = sql::MarketTradeGood::get_last(&self.context.database_pool)
            .await
            .unwrap();

        let filtered_trades = all_trades
            .into_iter()
            .map(|t| (cargo_data.get_amount(&t.symbol), t))
            .filter(|(amount, _)| *amount > 0)
            .collect::<Vec<_>>();

        if filtered_trades.is_empty() {
            return None;
        }

        let mut waypoints: HashMap<String, Vec<(sql::MarketTradeGood, i32, i32)>> = HashMap::new();

        for (amount, trade) in filtered_trades {
            let wp = waypoints.entry(trade.waypoint_symbol.clone()).or_default();
            let price = trade.sell_price;
            wp.push((trade, amount, amount * price));
        }

        let erg = waypoints
            .into_iter()
            .map(|(wps, v)| {
                let total_ammount = v.iter().map(|(_, amount, _)| *amount).sum::<i32>();
                let total_price = v.iter().map(|(_, _, price)| *price).sum::<i32>();
                (wps, total_ammount, total_price, v)
            })
            .collect::<Vec<_>>();

        let way_p = erg.iter().max_by(|a, b| a.2.cmp(&b.2));

        way_p.map(|w| {
            (
                w.0.clone(),
                w.3.iter().map(|(t, _, _)| t.symbol).collect::<Vec<_>>(),
            )
        })
    }

    async fn handle_cargo_selling(
        &self,
        ship: &mut ship::MyShip,
        api: &crate::api::Api,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
        trade_symbols: Vec<models::TradeSymbol>,
    ) -> Result<()> {
        let possible_trades = ship.get_market_info(api, database_pool).await?;

        ship.ensure_docked(api).await?;

        for trade in trade_symbols {
            if !possible_trades.iter().any(|t| t.symbol == trade) {
                warn!(
                    "Trade symbol {} not found in market: {}",
                    trade, ship.nav.waypoint_symbol
                );
                continue;
            }
            let amount = ship.cargo.get_amount(&trade);
            if amount == 0 {
                info!("Skipping {} as cargo is empty", trade);
                continue;
            }
            debug!("Selling {} units of {} for {}", amount, trade, ship.symbol);
            ship.sell_cargo(api, &trade, amount, database_pool, reason.clone())
                .await?;
        }

        Ok(())
    }
}
