use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use log::{debug, info, warn};
use space_traders_client::models::TradeSymbol;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{
        self,
        my_ship_update::{MyShipUpdate, ShipUpdate, TransferRequest},
    },
    sql::{self, MarketTradeGood, TransactionReason},
};

use super::mining_manager::{MiningManager, WaypointInfo};

#[derive(Debug, Clone)]
pub struct TransportProcessor {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<MiningManager>,
}

use thiserror::Error;

#[derive(Error, Debug)]
enum GetNextShipToUnloadError {
    #[error("No ships available at waypoint: {waypoint}")]
    NoShips { waypoint: String },
    #[error("No data available for waypoint: {waypoint}")]
    NoWaypointData { waypoint: String },
    #[error("No trade symbol found: {trade_symbol} at waypoint: {waypoint}")]
    NoTradeSymbol {
        trade_symbol: TradeSymbol,
        waypoint: String,
    },
    #[error("No cargo units available at waypoint: {waypoint}")]
    NoCargoUnits { waypoint: String },
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

            let mut mining_waypoints: String = "Start".to_string();

            if !(ship.cargo.get_units_no_fuel() as f32 / ship.cargo.capacity as f32 > 0.5) {
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

                mining_waypoints = ship.nav.waypoint_symbol.clone();

                let route = routes.last().unwrap();
                debug!("Route: {:?}", route);

                ship.nav_to(
                    &route.0,
                    true,
                    &waypoints,
                    &self.context.api,
                    self.context.database_pool.clone(),
                    TransactionReason::MiningWaypoint(mining_waypoints.clone()),
                )
                .await?;

                self.handle_cargo_loading(&mut ship.clone()).await?;
            }

            while ship.cargo.get_units_no_fuel() > 0 {
                if self.cancellation_token.is_cancelled() {
                    info!("Transport cycle cancelled for {} ", ship.symbol);
                    break;
                }
                let (next_waypoint, trade_symbols) =
                    self.get_next_best_sell_waypoint(&ship).await.unwrap();
                ship.nav_to(
                    &next_waypoint,
                    true,
                    &waypoints,
                    &self.context.api,
                    self.context.database_pool.clone(),
                    TransactionReason::MiningWaypoint(mining_waypoints.clone()),
                )
                .await?;

                self.handle_cargo_selling(
                    ship,
                    &self.context.api,
                    &self.context.database_pool,
                    TransactionReason::MiningWaypoint(mining_waypoints.clone()),
                    trade_symbols,
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn handle_cargo_loading(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        if ship.cargo.units >= ship.cargo.capacity {
            return Ok(());
            // return Err(anyhow::anyhow!("Cargo full"));
        }

        let mut previous_trade_symbol: Option<TradeSymbol> = None;

        debug!("Loading cargo for ship: {}", ship.symbol);

        self.mining_places.up_date(&ship.nav.waypoint_symbol).await;

        while ship.cargo.units < ship.cargo.capacity {
            if self.cancellation_token.is_cancelled() {
                break;
            }
            ship.try_recive_update(&self.context.api).await;
            let unload = self
                .get_next_ship_to_unload(&ship.nav.waypoint_symbol, previous_trade_symbol)
                .await;
            debug!("Unload: {:?} {}", unload, ship.symbol);
            if let Ok((ship_to_unload, trade_symbol, units)) = unload {
                previous_trade_symbol = Some(trade_symbol.clone());
                let free_space = ship.cargo.capacity - ship.cargo.units;
                let units = units.min(free_space);
                if units == 0 {
                    continue;
                }

                let (callback_sender, mut callback) = tokio::sync::mpsc::channel(5);

                ship.broadcaster.sender.send(MyShipUpdate {
                    symbol: ship_to_unload.symbol.clone(),
                    update: ShipUpdate::TransferRequest(TransferRequest {
                        trade_symbol,
                        units,
                        target: ship.symbol.clone(),
                        callback: callback_sender,
                    }),
                })?;

                let reg = callback.recv().await;
                debug!(
                    "Loaded cargo for ship: {} {} {} {:?}",
                    ship.symbol, trade_symbol, units, reg
                );
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
                debug!("jniodoinenioeinoeaion {}", ship.broadcaster.receiver.len());
                ship.try_recive_update(&self.context.api).await;
                debug!("mdemdemdeeee {}", ship.broadcaster.receiver.len());

                debug!(
                    "Loaded cargos for ship erl: {} {} {} {}",
                    ship.symbol, trade_symbol, units, ship.cargo.units
                );
            } else {
                let error = unload.unwrap_err();

                match error {
                    GetNextShipToUnloadError::NoShips { waypoint: _ } => {
                        let _erg = select! {
                          _ = self.cancellation_token.cancelled() => {0},
                          _ = ship.sleep(
                              std::time::Duration::from_millis(1000 + rand::random::<u64>() % 1000),
                              &self.context.api,
                          ) => {1},
                        };
                    }
                    GetNextShipToUnloadError::NoWaypointData { waypoint: _ } => {
                        return Err(anyhow::anyhow!("No waypoint data"));
                    }
                    GetNextShipToUnloadError::NoTradeSymbol {
                        trade_symbol: _,
                        waypoint: _,
                    } => {
                        previous_trade_symbol = None;
                    }
                    GetNextShipToUnloadError::NoCargoUnits { waypoint: _ } => {
                        let _erg = select! {
                          _ = self.cancellation_token.cancelled() => {0},
                          _ = ship.sleep(
                              std::time::Duration::from_millis(1000 + rand::random::<u64>() % 1000),
                              &self.context.api,
                          ) => {1},
                        };
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_next_ship_to_unload(
        &self,
        waypoint: &str,
        previous_trade_symbol: Option<TradeSymbol>,
    ) -> Result<(ship::MyShip, TradeSymbol, i32), GetNextShipToUnloadError> {
        let waypoint_data = self.mining_places.get_info(waypoint).await;

        if waypoint_data.is_none() {
            return Err(GetNextShipToUnloadError::NoWaypointData {
                waypoint: waypoint.to_string(),
            });
        }
        let waypoint_data = waypoint_data.unwrap();

        let ships = self
            .context
            .ship_manager
            .get_all_clone()
            .iter()
            .map(|s| s.1.clone())
            .filter(|s| !s.nav.is_in_transit() && s.cargo.units > 0)
            .filter(|s| waypoint_data.1.contains(&s.symbol))
            .collect::<Vec<_>>();

        if ships.is_empty() {
            return Err(GetNextShipToUnloadError::NoShips {
                waypoint: waypoint.to_string(),
            });
        }

        let trade_symbol = if let Some(previous_trade_symbol) = previous_trade_symbol {
            previous_trade_symbol
        } else {
            let mut cargo_items: HashMap<TradeSymbol, i32> = HashMap::new();

            for ship in ships.iter() {
                for (trade_symbol, cargo_amount) in ship.cargo.inventory.iter() {
                    let cargo_amount = cargo_amount;
                    cargo_items.insert(trade_symbol.clone(), *cargo_amount);
                }
            }
            let erg = cargo_items
                .iter()
                .max_by_key(|(_, v)| **v)
                .map(|(k, _)| k.clone());

            match erg {
                Some(trade_symbol) => trade_symbol,
                None => {
                    return Err(GetNextShipToUnloadError::NoCargoUnits {
                        waypoint: waypoint.to_string(),
                    })
                }
            }
        };

        let ship = ships
            .iter()
            .map(|s| (s.cargo.get_amount(&trade_symbol), s))
            .filter(|s| s.0 > 0)
            .max_by(|a, b| a.0.cmp(&b.0));

        if ship.is_none() {
            return Err(GetNextShipToUnloadError::NoTradeSymbol {
                trade_symbol,
                waypoint: waypoint.to_string(),
            });
        }

        let ship = ship.unwrap();

        Ok((
            ship.1.clone(),
            trade_symbol,
            ship.1.cargo.get_amount(&trade_symbol),
        ))
    }

    async fn get_next_best_sell_waypoint(
        &self,
        ship: &ship::MyShip,
    ) -> Option<(String, Vec<TradeSymbol>)> {
        let cargo_data = &ship.cargo;

        let all_trades = MarketTradeGood::get_last(&self.context.database_pool)
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

        let mut waypoints: HashMap<String, Vec<(MarketTradeGood, i32, i32)>> = HashMap::new();

        for (amount, trade) in filtered_trades {
            let wp = waypoints
                .entry(trade.waypoint_symbol.clone())
                .or_insert(Vec::new());
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
                w.3.iter()
                    .map(|(t, _, _)| t.symbol.clone())
                    .collect::<Vec<_>>(),
            )
        })
    }

    async fn handle_cargo_selling(
        &self,
        ship: &mut ship::MyShip,
        api: &crate::api::Api,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
        trade_symbols: Vec<TradeSymbol>,
    ) -> anyhow::Result<()> {
        let possible_trades = ship.get_market_info(api, database_pool).await?;

        ship.ensure_docked(api).await?;

        for trade in trade_symbols {
            if possible_trades.iter().find(|t| t.symbol == trade).is_none() {
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
            ship.sell_cargo(api, trade, amount, database_pool, reason.clone())
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
