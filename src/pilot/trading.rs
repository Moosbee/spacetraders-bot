use std::sync::{atomic::AtomicI32, Arc};

use log::{debug, info};

use crate::{
    error::{Error, Result},
    ship,
    sql::{self},
    types::ConductorContext,
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
        debug!("Starting trading cycle for ship {}", ship.symbol);

        ship.status = ship::ShipStatus::Trader {
            shipment_id: None,
            cycle: None,
            shipping_status: None,
            waiting_for_manager: true,
        };

        ship.notify().await;

        let route = self.context.trade_manager.get_route(ship).await?;
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status = ship::ShipStatus::Trader {
            shipment_id: Some(route.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify().await;

        info!("Starting trade route for ship {}: {}", ship.symbol, route);
        self.execute_trade(ship, &route, pilot).await?;
        let _completed_route = self.context.trade_manager.complete_trade(&route).await?;
        ship.status = ship::ShipStatus::Trader {
            shipment_id: None,
            cycle: None,
            shipping_status: None,
            waiting_for_manager: false,
        };
        if ship.role == sql::ShipInfoRole::TempTrader {
            ship.role = sql::ShipInfoRole::Manuel;
        }

        ship.notify().await;

        Ok(())
    }

    async fn execute_trade(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        debug!(
            "Executing trade for ship {} on route {:?}",
            ship.symbol, route
        );

        self.execute_purchase(ship, route, pilot).await?;

        self.execute_sale(ship, route).await?;

        debug!(
            "Trade execution completed for ship {} on route {:?}",
            ship.symbol, route
        );
        Ok(())
    }

    async fn execute_purchase(
        &self,
        ship: &mut ship::MyShip,
        route: &sql::TradeRoute,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        debug!(
            "Executing purchase for ship {} on route {:?}",
            ship.symbol, route
        );

        ship.status = ship::ShipStatus::Trader {
            shipment_id: Some(route.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify().await;

        if !ship.cargo.has(&route.symbol) {
            debug!(
                "Navigating to purchase waypoint: {}",
                route.purchase_waypoint
            );
            ship.nav_to(
                &route.purchase_waypoint,
                true,
                sql::TransactionReason::TradeRoute(route.id),
                &self.context,
            )
            .await?;

            ship.status = ship::ShipStatus::Trader {
                shipment_id: Some(route.id),
                cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
                shipping_status: Some(ship::ShippingStatus::Purchasing),
                waiting_for_manager: false,
            };

            ship.notify().await;

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
            let max_buy_volume = (ship.cargo.capacity - ship.cargo.units).min(route.trade_volume);
            let trade_volume = if budget < (purchase_price * max_buy_volume).into() {
                let trade_volume = (budget as f64 / purchase_price as f64).floor() as i32;
                debug!(
                    "Purchasing {} units of {} for {} due to budget constraint",
                    trade_volume, max_buy_volume, route.symbol
                );
                trade_volume.min(max_buy_volume)
            } else {
                max_buy_volume
            };

            debug!(
                "Purchasing cargo: {} units of {}",
                trade_volume, route.symbol
            );
            ship.purchase_cargo(
                &self.context.api,
                &route.symbol,
                trade_volume,
                &self.context.database_pool,
                sql::TransactionReason::TradeRoute(route.id),
            )
            .await?;
        }
        debug!(
            "Purchase completed for ship {} on route {:?}",
            ship.symbol, route
        );
        Ok(())
    }

    async fn execute_sale(&self, ship: &mut ship::MyShip, route: &sql::TradeRoute) -> Result<()> {
        debug!(
            "Executing sale for ship {} on route {:?}",
            ship.symbol, route
        );

        ship.status = ship::ShipStatus::Trader {
            shipment_id: Some(route.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToDelivery),
            waiting_for_manager: false,
        };

        ship.notify().await;

        debug!("Navigating to sell waypoint: {}", route.sell_waypoint);
        ship.nav_to(
            &route.sell_waypoint,
            true,
            sql::TransactionReason::TradeRoute(route.id),
            &self.context,
        )
        .await?;

        ship.status = ship::ShipStatus::Trader {
            shipment_id: Some(route.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::Delivering),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.ensure_docked(&self.context.api).await?;

        let cargo_volume = ship.cargo.get_amount(&route.symbol);
        debug!("Selling cargo: {} units of {}", cargo_volume, route.symbol);
        ship.sell_cargo(
            &self.context.api,
            &route.symbol,
            cargo_volume,
            &self.context.database_pool,
            sql::TransactionReason::TradeRoute(route.id),
        )
        .await?;

        debug!(
            "Sale completed for ship {} on route {:?}",
            ship.symbol, route
        );
        Ok(())
    }
}
