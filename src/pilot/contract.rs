use std::sync::{atomic::AtomicI32, Arc};

use log::debug;
use tracing::instrument;

use crate::{
    error::{Error, Result},
    manager::contract_manager::NextShipmentResp,
    utils::ConductorContext,
};

pub struct ContractPilot {
    context: ConductorContext,
    ship_symbol: String,
    count: Arc<AtomicI32>,
}

impl ContractPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            context,
            ship_symbol,
            count: Arc::new(AtomicI32::new(0)),
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_contract", skip(self, pilot), fields(self.ship_symbol = %self.ship_symbol, contract_shipment = tracing::field::Empty, contract_id = tracing::field::Empty))]
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next shipment for ship: {:?}", ship.symbol);

        let shipment = self
            .context
            .contract_manager
            .request_next_shipment(ship)
            .await?;

        debug!("Next shipment: {:?}", shipment);

        let (shipment, reservation_id) = match shipment {
            NextShipmentResp::Shipment(contract_shipment, reservation_id) => {
                (contract_shipment, reservation_id)
            }
            NextShipmentResp::ComeBackLater => {
                debug!("No shipment available, doing something else");
                return self.do_elsewhere(ship).await;
            }
        };

        tracing::Span::current().record("contract_shipment", format!("{:?}", shipment));
        tracing::Span::current().record("contract_id", &shipment.contract_id);

        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::Unknown),
            waiting_for_manager: false,
        };

        ship.notify().await;

        let storage_count = ship.cargo.get_amount(&shipment.trade_symbol);

        debug!("Storage count: {}", storage_count);

        if storage_count != shipment.units {
            debug!("Purchasing cargo");
            let er = self.purchase_cargo(ship, &shipment, reservation_id).await;
            if let Err(e) = er {
                let erg = self
                    .context
                    .contract_manager
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
                .contract_manager
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
            .contract_manager
            .complete_shipment(shipment, contract)
            .await?;

        ship.status = ship::ShipStatus::Contract {
            contract_id: None,
            run_id: None,
            cycle: None,
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
        shipment: &database::ContractShipment,
        reservation_id: Option<i64>,
    ) -> Result<()> {
        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify().await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &shipment.purchase_symbol,
            true,
            database::TransactionReason::Contract(shipment.contract_id.clone()),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn.clone(),
        )
        .await?;

        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
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

        debug!(
            "Purchasing units: {} should cost: {} funds",
            units_needed, current_price
        );

        let cost = ship
            .purchase_cargo(
                &self.context.api,
                &shipment.trade_symbol,
                units_needed,
                &self.context.database_pool,
                database::TransactionReason::Contract(shipment.contract_id.clone()),
                update_funds_fn,
            )
            .await?;

        if let Some(reservation_id) = reservation_id {
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
        shipment: database::ContractShipment,
    ) -> Result<(
        space_traders_client::models::Contract,
        database::ContractShipment,
    )> {
        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToDelivery),
            waiting_for_manager: false,
        };

        ship.notify().await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &shipment.destination_symbol,
            true,
            database::TransactionReason::Contract(shipment.contract_id.clone()),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;

        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
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
            .deliver_contract(
                &shipment.contract_id,
                shipment.trade_symbol,
                units_to_deliver,
                &self.context.api,
            )
            .await?;

        Ok((*response.data.contract, shipment))
    }
}
