use std::sync::Arc;

use chrono::Utc;
use log::{debug, info};
use space_traders_client::models::{waypoint, TradeSymbol};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{
        self,
        my_ship_update::{MyShipUpdate, ShipUpdate, TransferRequest},
    },
    sql::{self, TransactionReason},
};

use super::mining_manager::{MiningManager, WaypointInfo};

#[derive(Debug, Clone)]
pub struct TransportProcessor {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<MiningManager>,
}

impl TransportProcessor {
    pub fn new(
        context: crate::workers::types::ConductorContext,
        cancellation_token: CancellationToken,
        mining_places: Arc<MiningManager>,
    ) -> TransportProcessor {
        TransportProcessor {
            context,
            cancellation_token,
            mining_places,
        }
    }

    pub async fn run_transporter_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(
            500 + rand::random::<u64>() % 500,
        ))
        .await;

        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        for _i in 0..CONFIG.mining.max_extractions_per_miner {
            if self.cancellation_token.is_cancelled() {
                info!("Transport cycle cancelled for {} ", ship.symbol);
                break;
            }
            let route = self.calculate_waypoint_urgencys().await;
            debug!("Routes: {:?}", route);
            let routes = route.iter().filter(|r| r.1 > 0).collect::<Vec<_>>();

            if routes.is_empty() {
                info!("No routes found for {}", ship.symbol);
                tokio::time::sleep(std::time::Duration::from_millis(
                    5000 + rand::random::<u64>() % 5000,
                ))
                .await;
                continue;
            }

            let route = routes.last().unwrap();
            debug!("Route: {:?}", route);

            ship.nav_to(
                &route.0,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::None,
            )
            .await?;

            self.handle_cargo_loading(&mut ship.clone()).await?;

            while ship.cargo.get_units_no_fuel() > 0 {
                if self.cancellation_token.is_cancelled() {
                    info!("Transport cycle cancelled for {} ", ship.symbol);
                    break;
                }
                let next_waypoint = self.get_next_best_sell_waypoint(&ship).await.unwrap();
                ship.nav_to(
                    &next_waypoint.symbol,
                    true,
                    &waypoints,
                    &self.context.api,
                    self.context.database_pool.clone(),
                    TransactionReason::None,
                )
                .await?;

                self.handle_cargo_selling(
                    ship,
                    &self.context.api,
                    &self.context.database_pool,
                    TransactionReason::None,
                    false,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn handle_cargo_loading(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        if ship.cargo.units >= ship.cargo.capacity {
            return Err(anyhow::anyhow!("Cargo full"));
        }

        while ship.cargo.units < ship.cargo.capacity {
            if self.cancellation_token.is_cancelled() {
                break;
            }
            if let Some((ship_to_unload, trade_symbol, units)) = self
                .get_next_ship_to_unload(&ship.nav.waypoint_symbol)
                .await
            {
                let free_space = ship.cargo.capacity - ship.cargo.units;
                let units = units.min(free_space);

                let (callback_sender, mut callback) = tokio::sync::mpsc::channel(1);

                ship.broadcaster.sender.send(MyShipUpdate {
                    symbol: ship_to_unload.symbol.clone(),
                    update: ShipUpdate::TransferRequest(TransferRequest {
                        trade_symbol,
                        units,
                        target: ship_to_unload.symbol.clone(),
                        callback: callback_sender,
                    }),
                })?;

                let _ = callback.recv().await;
                if callback.len() != 0 {
                    log::warn!(
                        "Callback not empty for ship: {} {} {}",
                        ship.symbol,
                        ship_to_unload.symbol,
                        callback.len()
                    );
                }
                callback.close();
                drop(callback);
                ship.try_recive_update(&self.context.api).await;
            } else {
                let _erg = select! {
                  _ = self.cancellation_token.cancelled() => {0},
                  _ = ship.sleep(
                      std::time::Duration::from_millis(1000 + rand::random::<u64>() % 1000),
                      &self.context.api,
                  ) => {1},
                };
            }
        }

        Ok(())
    }

    async fn get_next_ship_to_unload(
        &self,
        waypoint: &str,
    ) -> Option<(ship::MyShip, TradeSymbol, i32)> {
        todo!()
    }

    async fn get_next_best_sell_waypoint(&self, ship: &ship::MyShip) -> Option<waypoint::Waypoint> {
        todo!()
    }

    async fn handle_cargo_selling(
        &self,
        ship: &mut ship::MyShip,
        api: &crate::api::Api,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
        sell_to_export: bool,
    ) -> anyhow::Result<()> {
        let possible_trades = ship.get_market_info(api, database_pool).await?;

        for trade in possible_trades.iter() {
            if !sell_to_export
                && trade.r#type == space_traders_client::models::market_trade_good::Type::Export
            {
                continue;
            }
            let amount = ship.cargo.get_amount(&trade.symbol);
            ship.sell_cargo(api, trade.symbol, amount, database_pool, reason.clone())
                .await?;
        }

        Ok(())
    }

    async fn calculate_waypoint_urgencys(&self) -> Vec<(String, u32)> {
        let waypoints = self.mining_places.get_all().await;
        let the_ships: std::collections::HashMap<String, ship::MyShip> =
            self.context.ship_manager.get_all_clone();

        let mut erg = waypoints
            .iter()
            .map(|wp| Self::calculate_waypoint_urgency(wp.1, the_ships.clone()))
            .collect::<Vec<_>>();

        erg.sort_by(|a, b| b.1.cmp(&a.1));

        erg
    }

    fn calculate_waypoint_urgency(
        wp: &WaypointInfo,
        ships: std::collections::HashMap<String, ship::MyShip>,
    ) -> (String, u32) {
        let (units_sum, capacity_sum) =
            wp.1.iter()
                .map(|s| ships.get(s).unwrap())
                .filter(|sh| sh.nav.waypoint_symbol == wp.0 && !sh.nav.is_in_transit())
                .map(|sh| (sh.cargo.units, sh.cargo.capacity))
                .fold((0, 0), |(units_sum, capacity_sum), (units, capacity)| {
                    (units_sum + units, capacity_sum + capacity)
                });

        let cargo_ratio = (units_sum as f32 / capacity_sum as f32) * 100.0;
        let cargo_ratio = if cargo_ratio.is_nan() {
            0.0
        } else {
            cargo_ratio
        };

        let since_last = wp.2 - Utc::now();

        let urgency = since_last.num_seconds() * cargo_ratio as i64;

        (wp.0.clone(), urgency as u32)
    }
}
