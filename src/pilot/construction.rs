use std::sync::{atomic::AtomicI32, Arc};

use log::debug;
use space_traders_client::models;

use crate::{
    error::{Error, Result},
    manager::construction_manager::NextShipmentResp,
    utils::ConductorContext,
};

pub struct ConstructionPilot {
    context: ConductorContext,
    ship_symbol: String,
    count: Arc<AtomicI32>,
}

impl ConstructionPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            context,
            ship_symbol,
            count: Arc::new(AtomicI32::new(0)),
        }
    }
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next shipment for ship: {:?}", ship.symbol);

        let shipment = self
            .context
            .construction_manager
            .next_shipment(ship.clone())
            .await?;

        debug!("Next shipment: {:?}", shipment);

        let shipment = match shipment {
            NextShipmentResp::Shipment(construct_shipment) => construct_shipment,
            NextShipmentResp::ComeBackLater => {
                debug!("No shipment available, doing something else");
                return self.do_elsewhere(ship).await;
            }
        };

        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status = ship::ShipStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Unknown),
            waiting_for_manager: false,
        };

        ship.notify().await;

        let storage_count = ship.cargo.get_amount(&shipment.trade_symbol);

        debug!("Storage count: {}", storage_count);

        if storage_count != shipment.units {
            debug!("Purchasing cargo");
            let er = self.purchase_cargo(ship, &shipment, pilot).await;
            if let Err(e) = er {
                let erg = self
                    .context
                    .construction_manager
                    .fail_shipment(shipment, e)
                    .await?;
                if let Error::NotEnoughFunds { .. } = erg {
                    return Ok(());
                }

                return Err(erg);
            }
        }

        debug!("Delivering cargo");

        let del_erg = self.deliver_cargo(ship, shipment.clone()).await;

        if let Err(e) = del_erg {
            let erg = self
                .context
                .construction_manager
                .fail_shipment(shipment, e)
                .await?;
            if let Error::NotEnoughFunds { .. } = erg {
                return Ok(());
            }

            return Err(erg);
        }

        let (contract, shipment) = del_erg.unwrap();

        debug!("Completing shipment");

        self.context
            .construction_manager
            .complete_shipment(shipment, contract)
            .await?;

        ship.status = ship::ShipStatus::Construction {
            cycle: None,
            shipment_id: None,
            shipping_status: None,
            waiting_for_manager: false,
        };

        ship.notify().await;

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut ship::MyShip) -> Result<()> {
        ship.status = ship::ShipStatus::Manuel;
        ship.role = database::ShipInfoRole::TempTrader;
        debug!("Doing something else");
        ship.notify().await;

        Ok(())
    }

    async fn purchase_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: &database::ConstructionShipment,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        ship.status = ship::ShipStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.nav_to(
            &shipment.purchase_waypoint,
            true,
            database::TransactionReason::Construction(shipment.id),
            &self.context.database_pool,
            &self.context.api,
        )
        .await?;

        ship.status = ship::ShipStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Purchasing),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.ensure_docked(&self.context.api).await?;

        let market_info = ship
            .get_market_info(&self.context.api, &self.context.database_pool)
            .await?;
        let market_trade = market_info
            .iter()
            .find(|t| t.symbol == shipment.trade_symbol)
            .unwrap();

        let units_needed = shipment.units - ship.cargo.get_amount(&shipment.trade_symbol);

        let current_price = (market_trade.purchase_price * units_needed) as i64;

        let budget = pilot.get_budget().await?;

        if budget < current_price {
            debug!(
                "Not enough funds to purchase units: {} should cost: {} funds has {} funds",
                units_needed, current_price, budget
            );
            return Err(Error::NotEnoughFunds {
                remaining_funds: budget,
                required_funds: current_price,
            });
        }

        debug!(
            "Purchasing units: {} should cost: {} funds has {} funds",
            units_needed, current_price, budget
        );

        ship.purchase_cargo(
            &self.context.api,
            &shipment.trade_symbol,
            units_needed,
            &self.context.database_pool,
            database::TransactionReason::Construction(shipment.id),
        )
        .await?;

        Ok(())
    }

    async fn deliver_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: database::ConstructionShipment,
    ) -> Result<(models::Construction, database::ConstructionShipment)> {
        ship.status = ship::ShipStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::InTransitToDelivery),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.nav_to(
            &shipment.construction_site_waypoint,
            true,
            database::TransactionReason::Construction(shipment.id),
            &self.context.database_pool,
            &self.context.api,
        )
        .await?;

        ship.status = ship::ShipStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Delivering),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.ensure_docked(&self.context.api).await?;

        let units_to_deliver = ship
            .cargo
            .get_amount(&shipment.trade_symbol)
            .min(shipment.units);

        debug!("Delivering units: {}", units_to_deliver);

        let response = ship
            .supply_construction(shipment.trade_symbol, units_to_deliver, &self.context.api)
            .await?;

        Ok((*response.data.construction, shipment))
    }
}
