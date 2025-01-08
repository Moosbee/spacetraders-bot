use std::sync::{atomic::AtomicI32, Arc};

use crate::{
    error::{Error, Result},
    manager::trade_manager::TradeManagerMessage,
    ship,
    sql::{self},
    workers::types::ConductorContext,
};

pub struct TradingPilot {
    context: ConductorContext,
    ship_symbol: String,
    count: Arc<AtomicI32>,
}

impl TradingPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            context,
            ship_symbol,
            count: Arc::new(AtomicI32::new(0)),
        }
    }
    pub async fn execute_pilot_circle(&self, pilot: &crate::pilot::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;
        let route = self.get_route(ship).await?;
        let _route_erg = self.execute_trade(ship, &route, pilot).await?;
        let _completed_route = self.complete_trade(route).await?;
        Ok(())
    }

    async fn get_route(&self, ship: &mut ship::MyShip) -> Result<sql::TradeRoute> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = TradeManagerMessage::RequestNextTradeRoute {
            ship_clone: ship.clone(),
            callback: sender,
        };

        let _erg = self
            .context
            .trade_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get message: {}", e)))?;

        resp
    }

    async fn complete_trade(&self, trade_route: sql::TradeRoute) -> Result<sql::TradeRoute> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = TradeManagerMessage::CompleteTradeRoute {
            trade_route: trade_route,
            callback: sender,
        };

        let _erg = self
            .context
            .trade_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get message: {}", e)))?;

        resp
    }

    async fn execute_trade(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let num = self.count.load(std::sync::atomic::Ordering::Relaxed);
        ship.role = ship::Role::Trader(Some((route.id, num)));
        self.execute_purchase(ship, route, pilot).await?;
        self.execute_sale(ship, route).await?;

        Ok(())
    }

    async fn execute_purchase(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        if !ship.cargo.has(&route.symbol) {
            let waypoints = self
                .context
                .all_waypoints
                .get(&ship.nav.system_symbol)
                .unwrap()
                .clone();

            ship.nav_to(
                &route.purchase_waypoint,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                sql::TransactionReason::TradeRoute(route.id),
            )
            .await?;

            ship.ensure_docked(&self.context.api).await?;

            let market_info = ship
                .get_market_info(&self.context.api, &self.context.database_pool)
                .await?;

            let purchase_price = market_info
                .iter()
                .find(|m| m.symbol == route.symbol)
                .ok_or(Error::General(format!(
                    "No market info for {}",
                    route.symbol
                )))?
                .purchase_price;

            let budget = pilot.get_budget().await?;
            let trade_volume = if budget < (purchase_price * route.trade_volume).into() {
                let trade_volume = (budget as f64 / purchase_price as f64).floor() as i32;
                trade_volume
            } else {
                route.trade_volume
            };

            ship.purchase_cargo(
                &self.context.api,
                route.symbol,
                trade_volume,
                &self.context.database_pool,
                sql::TransactionReason::TradeRoute(route.id),
            )
            .await?;
        }
        Ok(())
    }

    async fn execute_sale(&self, ship: &mut ship::MyShip, route: &sql::TradeRoute) -> Result<()> {
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        ship.nav_to(
            &route.sell_waypoint,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::TradeRoute(route.id),
        )
        .await?;

        ship.ensure_docked(&self.context.api).await?;

        let cargo_volume = ship.cargo.get_amount(&route.symbol);
        ship.sell_cargo(
            &self.context.api,
            route.symbol,
            cargo_volume,
            &self.context.database_pool,
            sql::TransactionReason::TradeRoute(route.id),
        )
        .await?;

        Ok(())
    }
}
