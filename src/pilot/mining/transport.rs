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
    workers::types::ConductorContext,
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

        while !(ship.cargo.get_units_no_fuel() as f32
            / (ship.cargo.capacity
                - ship
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel))
                as f32
            > 0.9)
        {
            let next_mining_waypoint = self.get_next_mining_waypoint(ship).await?;

            last_waypoint = next_mining_waypoint.clone();

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

            self.handle_cargo_loading(ship, pilot).await?;
        }

        self.sell_all_cargo(pilot, ship, &waypoints, last_waypoint)
            .await?;

        pilot.cancellation_token.cancelled().await;
        Ok(())
    }

    async fn get_next_mining_waypoint(&self, ship: &mut ship::MyShip) -> Result<String> {
        let next_transport = self.context.mining_manager.get_next_transport(ship).await?;

        Ok(next_transport)
    }

    async fn handle_cargo_loading(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        // tell mining manager you have arrived
        // wait until storage is full or are told to leave
        //    in meantime, listen to mining manager and load cargo it tells you
        // cut connection to mining manager

        let mut rec = self
            .context
            .mining_manager
            .transport_contact(&ship.symbol)
            .await?;

        let _erg = self
            .context
            .mining_manager
            .transport_arrived(&ship.symbol, &ship.nav.waypoint_symbol)
            .await?;

        while !(ship.cargo.get_units_no_fuel() as f32
            / (ship.cargo.capacity
                - ship
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel))
                as f32
            > 0.9)
        {
            let msg = tokio::select! {
              _ = pilot.cancellation_token.cancelled() => {None},
              msg = rec.recv() => msg,
            };

            match msg {
                None => {
                    break;
                }
                Some(transfer_request) => {
                    self.handle_transfer_request(ship, transfer_request).await?;
                }
            }
        }

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

        let _erg = request
            .extractor_contact
            .send(extractor_req)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let transfer = receiver
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))?
            .ok_or("Failed to receive message")?;

        let _erg = ship
            .cargo
            .handle_cago_update(transfer.units, transfer.trade_symbol)?;

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
            let (next_waypoint, trade_symbols) =
                self.get_next_best_sell_waypoint(&ship).await.unwrap();
            ship.nav_to(
                &next_waypoint,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::MiningWaypoint(mining_waypoint.clone()),
            )
            .await?;

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
        trade_symbols: Vec<models::TradeSymbol>,
    ) -> Result<()> {
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
            ship.sell_cargo(api, &trade, amount, database_pool, reason.clone())
                .await?;
        }

        Ok(())
    }
}
