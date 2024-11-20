// Trade execution
use std::sync::Arc;

use log::info;

use crate::{
    ship,
    sql::{self, DatabaseConnector},
    workers::{
        trading::{routes_track_keeper::RoutesTrackKeeper, t_types::ConcreteTradeRoute},
        types::ConductorContext,
    },
};

#[derive(Debug, Clone)]
pub struct TradeProcessor {
    context: ConductorContext,
    running_routes: Arc<RoutesTrackKeeper>,
}

impl TradeProcessor {
    pub fn new(context: ConductorContext, running_routes: Arc<RoutesTrackKeeper>) -> Self {
        Self {
            context,
            running_routes,
        }
    }

    pub async fn process_trade_route(
        &self,
        ship: &mut ship::MyShip,
        route: ConcreteTradeRoute,
    ) -> anyhow::Result<()> {
        if self.running_routes.is_locked(&route.clone().into()) {
            return Err(anyhow::anyhow!("Route has been locked"));
        }

        self.running_routes.lock(&route.clone().into())?;

        info!("Processing route: {}", route);

        let trade_record = self.record_trade_start(&route).await?;

        self.execute_trade(ship, &route, trade_record.id).await?;

        self.complete_trade_record(trade_record).await?;

        info!("Completed route: {}", route);

        self.running_routes.unlock(&route.into());
        Ok(())
    }

    pub async fn execute_trade(
        &self,
        ship: &mut ship::MyShip,
        route: &ConcreteTradeRoute,
        route_id: i32,
    ) -> anyhow::Result<()> {
        self.execute_purchase(ship, &route, route_id).await?;
        self.execute_sale(ship, &route, route_id).await?;

        Ok(())
    }

    async fn record_trade_start(
        &self,
        route: &ConcreteTradeRoute,
    ) -> anyhow::Result<sql::TradeRoute> {
        let sql_route: sql::TradeRoute = route.clone().into();
        let id = sql::TradeRoute::insert_new(&self.context.database_pool, &sql_route).await?;
        Ok(sql::TradeRoute { id, ..sql_route })
    }

    async fn execute_purchase(
        &self,
        ship: &mut ship::MyShip,
        route: &ConcreteTradeRoute,
        trade_id: i32,
    ) -> anyhow::Result<()> {
        if !ship.cargo.has(&route.symbol) {
            let waypoints = self
                .context
                .all_waypoints
                .get(&ship.nav.system_symbol)
                .unwrap()
                .clone();

            ship.nav_to(
                &route.purchase_wp_symbol,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                sql::TransactionReason::TradeRoute(trade_id),
            )
            .await?;

            ship.ensure_docked(&self.context.api).await?;

            ship.purchase_cargo(
                &self.context.api,
                route.symbol,
                route.trip_units,
                &self.context.database_pool,
                sql::TransactionReason::TradeRoute(trade_id),
            )
            .await?;
        }
        Ok(())
    }

    async fn execute_sale(
        &self,
        ship: &mut ship::MyShip,
        route: &ConcreteTradeRoute,
        trade_id: i32,
    ) -> anyhow::Result<()> {
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        ship.nav_to(
            &route.sell_wp_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::TradeRoute(trade_id),
        )
        .await?;

        ship.ensure_docked(&self.context.api).await?;

        let cargo_volume = ship.cargo.get_amount(&route.symbol);
        ship.sell_cargo(
            &self.context.api,
            route.symbol,
            cargo_volume,
            &self.context.database_pool,
            sql::TransactionReason::TradeRoute(trade_id),
        )
        .await?;

        Ok(())
    }

    async fn complete_trade_record(&self, trade_route: sql::TradeRoute) -> anyhow::Result<()> {
        let completed_route = trade_route.complete();
        sql::TradeRoute::insert(&self.context.database_pool, &completed_route).await?;
        Ok(())
    }
}
