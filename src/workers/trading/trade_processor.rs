// Trade execution
use std::sync::Arc;

use log::{info, warn};

use crate::{
    ship,
    sql::{self, DatabaseConnector},
    workers::{trading::routes_track_keeper::RoutesTrackKeeper, types::ConductorContext},
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
        route: sql::TradeRoute,
        num: u32,
    ) -> anyhow::Result<()> {
        let locked = self.running_routes.lock(&route.clone().into());

        if let Err(e) = locked {
            warn!("Failed to lock route: {} {} {}", e, route, ship.symbol);
            return Err(e);
        }

        info!("Processing route: {}", route);

        let trade_record = self.record_trade_start(&route).await?;

        ship.role = ship::Role::Trader(Some((trade_record.id, num)));
        ship.notify().await;

        self.execute_trade(ship, &route, trade_record.id).await?;

        self.complete_trade_record(trade_record).await?;

        ship.role = ship::Role::Trader(None);
        ship.notify().await;

        info!("Completed route: {}", route);

        self.running_routes.unlock(&route.into());
        Ok(())
    }

    pub async fn execute_trade(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
        route_id: i32,
    ) -> anyhow::Result<()> {
        self.execute_purchase(ship, &route, route_id).await?;
        self.execute_sale(ship, &route, route_id).await?;

        Ok(())
    }

    async fn record_trade_start(&self, route: &sql::TradeRoute) -> anyhow::Result<sql::TradeRoute> {
        if route.id == 0 {
            let id = sql::TradeRoute::insert_new(&self.context.database_pool, route).await?;
            Ok(sql::TradeRoute {
                id,
                ..route.clone()
            })
        } else {
            sql::TradeRoute::insert(&self.context.database_pool, route).await?;
            Ok(route.clone())
        }
    }

    async fn execute_purchase(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
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
                &route.purchase_waypoint,
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
                route.trade_volume,
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
        route: &sql::TradeRoute,
        trade_id: i32,
    ) -> anyhow::Result<()> {
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
