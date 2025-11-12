use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc},
};

use ship::status::{MiningShipAssignment, TransporterState};
use space_traders_client::models;
use tracing::debug;
use tracing::instrument;

use crate::{
    error::Result,
    manager::mining_manager::{ExtractorTransferRequest, TransportTransferRequest},
    utils::ConductorContext,
};

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

    #[instrument(level = "info", name = "spacetraders::pilot::mining::pilot_transport", skip(self, pilot, ship), fields(self.ship_symbol = pilot.ship_symbol))]
    pub async fn execute_transport_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let waypoints =
            database::Waypoint::get_by_system(&self.context.database_pool, &ship.nav.system_symbol)
                .await?
                .into_iter()
                .map(|w| (w.symbol.clone(), w))
                .collect::<HashMap<_, _>>();

        let mut last_waypoint = ship.nav.waypoint_symbol.clone();

        ship.status.status = ship::AssignmentStatus::Mining {
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
            debug!(next_mining_waypoint = ?next_mining_waypoint, "Next transport mining waypoint");
            if next_mining_waypoint.is_err() {
                let next_err = next_mining_waypoint.unwrap_err();
                if let crate::error::Error::General(err_r) = &next_err {
                    if err_r == "No routes found" {
                        tracing::info!("No more mining waypoints");
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

            ship.status.status = ship::AssignmentStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::InTransitToAsteroid,
                    waypoint_symbol: Some(next_mining_waypoint.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            let budget_manager = self.context.budget_manager.clone();

            let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

            ship.nav_to_prepare(
                &next_mining_waypoint,
                true,
                database::TransactionReason::MiningWaypoint(next_mining_waypoint.clone()),
                true,
                &self.context.database_pool,
                &self.context.api,
                update_funds_fn,
            )
            .await?;

            debug!(next_mining_waypoint = %next_mining_waypoint, "Navigated to waypoint");

            self.handle_cargo_loading(ship, pilot).await?;
        }
        self.sell_all_cargo(pilot, ship, &waypoints, last_waypoint)
            .await?;

        ship.status.status = ship::AssignmentStatus::Mining {
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

    debug!(next_transport = %next_transport, "Next transport mining waypoint");

        Ok(next_transport)
    }

    async fn handle_cargo_loading(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
    debug!(ship_symbol = %ship.symbol, "Initiating cargo loading for ship");
        // tell mining manager you have arrived
        // wait until storage is full or are told to leave
        //    in meantime, listen to mining manager and load cargo it tells you
        // cut connection to mining manager

        let mut rec = self
            .context
            .mining_manager
            .transport_contact(&ship.symbol)
            .await?;
    debug!(ship_symbol = %ship.symbol, "Transport contact established for ship");

        let _erg = self
            .context
            .mining_manager
            .transport_arrived(&ship.symbol, &ship.nav.waypoint_symbol)
            .await?;
        debug!(ship_symbol = %ship.symbol, waypoint_symbol = %ship.nav.waypoint_symbol, "Transport arrived notification sent");

        while !(ship.cargo.get_units_no_fuel() as f32
            / (ship.cargo.capacity
                - ship
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel))
                as f32
            > 0.95)
        {
            ship.status.status = ship::AssignmentStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::WaitingForCargo,
                    waypoint_symbol: Some(ship.nav.waypoint_symbol.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            let msg = tokio::select! {
                _ = pilot.cancellation_token.cancelled() => {
                    debug!(ship_symbol = %ship.symbol, "Cancellation token received");
                    None
                },
                msg = rec.recv() => msg,
            };

            ship.status.status = ship::AssignmentStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::LoadingCargo,
                    waypoint_symbol: Some(ship.nav.waypoint_symbol.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;

            match msg {
                None => {
                    debug!(ship_symbol = %ship.symbol, "No more messages; shutting down transport");
                    break;
                }
                Some(transfer_request) => {
                    debug!(ship_symbol = %ship.symbol, "Received transfer request for ship");
                    let from = transfer_request.from_symbol.clone();
                    let to = transfer_request.to_symbol.clone();
                    let erg = self.handle_transfer_request(ship, transfer_request).await;
                    if let Err(error) = erg {
                        tracing::error!(
                            ship_symbol = ship.symbol,
                            to_symbol = to,
                            from_symbol = from,
                            error = format!("{} {:?}", error, error),
                            "Transfer request failed",
                        );
                        return Err(error);
                    }
                }
            }
        }

        debug!(ship_symbol = %ship.symbol, "Finalizing cargo loading; shutting down transport");
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
            tracing::error!(
              error=?err,
              "Failed to send message to extractor",
            );
            let _erg = request.callback.send(());

            return Ok(());
        }

        let transfer = receiver.await;

        let transfer = match transfer {
            Ok(transfer) => transfer,
            Err(e) => {
                tracing::error!(
                  error=?e,
                  "Failed to send message to extractor",
                );
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
        waypoints: &std::collections::HashMap<String, database::Waypoint>,
        mining_waypoint: String,
    ) -> Result<()> {
        while ship.cargo.get_units_no_fuel() > 0 {
            if pilot.cancellation_token.is_cancelled() {
                tracing::info!(symbol = %ship.symbol, "Transport cycle cancelled");
                break;
            }

            ship.status.status = ship::AssignmentStatus::Mining {
                assignment: MiningShipAssignment::Transporter {
                    state: TransporterState::InTransitToMarket,
                    waypoint_symbol: Some(mining_waypoint.clone()),
                    cycles: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
                },
            };
            ship.notify().await;
            let (next_waypoint, trade_symbols) =
                self.get_next_best_sell_waypoint(ship).await.unwrap();

            let budget_manager = self.context.budget_manager.clone();

            let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

            ship.nav_to(
                &next_waypoint,
                true,
                database::TransactionReason::MiningWaypoint(mining_waypoint.clone()),
                &self.context.database_pool,
                &self.context.api,
                update_funds_fn,
            )
            .await?;

            ship.status.status = ship::AssignmentStatus::Mining {
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
                database::TransactionReason::MiningWaypoint(mining_waypoint.clone()),
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

        let all_trades = database::MarketTradeGood::get_last_by_system(
            &self.context.database_pool,
            &ship.nav.system_symbol,
        )
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

        let mut waypoints: HashMap<String, Vec<_>> = HashMap::new();

        for (amount, trade) in filtered_trades {
            let wp = waypoints.entry(trade.waypoint_symbol.clone()).or_default();
            let price = trade.sell_price;
            let trade_supply = trade.supply;
            wp.push((trade, amount, amount * price, trade_supply));
        }

        let mut erg = waypoints
            .into_iter()
            .map(|(wps, v)| {
                let median_supply_level = calculate_median(
                    v.iter()
                        .map(|(_, _, _, supply)| *supply)
                        .collect::<Vec<_>>(),
                );
                let total_ammount = v.iter().map(|(_, amount, _, _)| *amount).sum::<i32>();
                let total_price = v.iter().map(|(_, _, price, _)| *price).sum::<i32>();
                (wps, total_ammount, total_price, v, median_supply_level)
            })
            .collect::<Vec<_>>();

        // let way_p = erg.iter().max_by(|a, b| a.2.cmp(&b.2));
        erg.sort_by(|a, b| a.2.cmp(&b.2));
        let way_p = erg.iter().min_by(|a, b| a.4.cmp(&b.4));

        way_p.map(|w| {
            (
                w.0.clone(),
                w.3.iter().map(|(t, _, _, _)| t.symbol).collect::<Vec<_>>(),
            )
        })
    }

    async fn handle_cargo_selling(
        &self,
        ship: &mut ship::MyShip,
        api: &space_traders_client::Api,
        database_pool: &database::DbPool,
        reason: database::TransactionReason,
        trade_symbols: Vec<models::TradeSymbol>,
    ) -> Result<()> {
        let possible_trades = ship.get_market_info(api, database_pool).await?;

        ship.ensure_docked(api).await?;

        for trade in trade_symbols {
            if !possible_trades.iter().any(|t| t.symbol == trade) {
                tracing::warn!(
                    ship_symbol = ship.symbol,
                    trade_symbol = trade.to_string(),
                    "Skipping trade as not in market"
                );
                continue;
            }
            let amount = ship.cargo.get_amount(&trade);
            if amount == 0 {
                tracing::info!(trade = ?trade, "Skipping trade as cargo is empty");
                continue;
            }
            debug!(amount = amount, trade = ?trade, ship_symbol = %ship.symbol, "Selling units of trade for ship");

            let budget_manager = self.context.budget_manager.clone();

            let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

            ship.sell_cargo(
                api,
                &trade,
                amount,
                database_pool,
                reason.clone(),
                update_funds_fn,
            )
            .await?;
        }

        Ok(())
    }
}

fn calculate_median(vec: Vec<models::SupplyLevel>) -> Option<models::SupplyLevel> {
    let len = vec.len();

    let mut sorted_vec = vec.clone();
    sorted_vec.sort_unstable();

    sorted_vec.get(len / 2).cloned()
}
