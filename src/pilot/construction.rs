use std::sync::{atomic::AtomicI32, Arc};

use space_traders_client::models;
use tracing::debug;
use tracing::instrument;
use tracing::warn;

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

    #[instrument(level = "info", name = "spacetraders::pilot::construction::pilot_construction", skip(self, pilot, fleet, ship_assignment, construction_config), fields(self.ship_symbol = %self.ship_symbol, construction_shipment, fleet_id = fleet.id, ship_assignment_id = ship_assignment.id))]
    pub async fn execute_pilot_circle(
        &self,
        pilot: &super::Pilot,
        fleet: database::Fleet,
        ship_assignment: database::ShipAssignment,
        construction_config: database::ConstructionFleetConfig,
    ) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

    debug!(ship_symbol = %ship.symbol, "Requesting next shipment for ship");

        let shipment = self
            .context
            .construction_manager
            .next_shipment(ship.clone())
            .await?;

    debug!(shipment = ?shipment, "Next shipment");

        // if (self.count.load(std::sync::atomic::Ordering::SeqCst) == 0) {
        //     self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        //     return self.do_elsewhere(ship).await;
        // }

        let shipment = match shipment {
            NextShipmentResp::Shipment(construct_shipment) => construct_shipment,
            NextShipmentResp::ComeBackLater => {
                debug!("No shipment available, doing something else");
                return self.do_elsewhere(ship, pilot).await;
            }
        };

        tracing::Span::current().record("construction_shipment", format!("{:?}", shipment));

        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Unknown),
            waiting_for_manager: false,
        };

        ship.notify(true).await;

        let storage_count = ship.cargo.get_amount(&shipment.trade_symbol);

    debug!(storage_count, "Storage count");

        if storage_count != shipment.units {
            debug!("Purchasing cargo");
            let er = self.purchase_cargo(ship, &shipment).await;
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

        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: None,
            shipment_id: None,
            shipping_status: None,
            waiting_for_manager: false,
        };

        ship.notify(true).await;

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut ship::MyShip, pilot: &super::Pilot) -> Result<()> {
        let temp_assignment = self
            .context
            .fleet_manager
            .get_new_temp_assignment(ship)
            .await?;
        if temp_assignment.is_none() {
            warn!("No temp assignment available, skipping");
            tokio::time::sleep(std::time::Duration::from_millis(60_000)).await;
            return Ok(());
        }

        Ok(())
    }

    async fn purchase_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: &database::ConstructionShipment,
    ) -> Result<()> {
        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify(true).await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &shipment.purchase_waypoint,
            true,
            database::TransactionReason::Construction(shipment.id),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn.clone(),
        )
        .await?;

        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Purchasing),
            waiting_for_manager: false,
        };

        ship.notify(true).await;

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

        debug!(units_needed, current_price, "Purchasing units and cost");

        let cost = ship
            .purchase_cargo(
                &self.context.api,
                &shipment.trade_symbol,
                units_needed,
                &self.context.database_pool,
                database::TransactionReason::Construction(shipment.id),
                update_funds_fn,
            )
            .await?;

        if let Some(reservation_id) = shipment.reserved_fund {
            self.context
                .budget_manager
                .use_reservation(&self.context.database_pool, reservation_id, cost)
                .await?;
        }

        Ok(())
    }

    async fn deliver_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: database::ConstructionShipment,
    ) -> Result<(models::Construction, database::ConstructionShipment)> {
        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::InTransitToDelivery),
            waiting_for_manager: false,
        };

        ship.notify(true).await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &shipment.construction_site_waypoint,
            true,
            database::TransactionReason::Construction(shipment.id),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;

        ship.status.status = ship::AssignmentStatus::Construction {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipment_id: Some(shipment.id),
            shipping_status: Some(ship::ShippingStatus::Delivering),
            waiting_for_manager: false,
        };

        ship.notify(true).await;

        ship.ensure_docked(&self.context.api).await?;

        let units_to_deliver = ship
            .cargo
            .get_amount(&shipment.trade_symbol)
            .min(shipment.units);

    debug!(units_to_deliver, "Delivering units");

        let response = ship
            .supply_construction(shipment.trade_symbol, units_to_deliver, &self.context.api)
            .await?;

        Ok((*response.data.construction, shipment))
    }
}
