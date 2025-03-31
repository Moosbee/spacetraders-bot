use std::sync::{atomic::AtomicI32, Arc};

use log::debug;

use crate::{
    error::{Error, Result},
    manager::contract_manager::{ContractShipmentMessage, NextShipmentResp},
    ship, sql,
    types::ConductorContext,
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

    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next shipment for ship: {:?}", ship.symbol);

        let shipment = self.request_next_shipment(ship).await?;

        debug!("Next shipment: {:?}", shipment);

        let shipment = match shipment {
            NextShipmentResp::Shipment(contract_shipment) => contract_shipment,
            NextShipmentResp::ComeBackLater => {
                debug!("No shipment available, doing something else");
                return self.do_elsewhere(ship).await;
            }
        };

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
            let er = self.purchase_cargo(ship, &shipment, pilot).await;
            if let Err(e) = er {
                let erg = self.fail_shipment(shipment, e).await?;
                if let Error::NotEnoughFunds { .. } = erg {
                    return Ok(());
                }

                return Err(erg);
            }
        }

        debug!("Delivering cargo");

        let del_erg = self.deliver_cargo(ship, shipment.clone()).await;

        if let Err(e) = del_erg {
            let erg = self.fail_shipment(shipment, e).await?;
            if let Error::NotEnoughFunds { .. } = erg {
                return Ok(());
            }

            return Err(erg);
        }

        let (contract, shipment) = del_erg.unwrap();

        debug!("Completing shipment");

        self.complete_shipment(shipment, contract).await?;

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
        ship.role = sql::ShipInfoRole::TempTrader;
        debug!("Doing something else");
        ship.notify().await;

        Ok(())
    }

    async fn request_next_shipment(&self, ship: &ship::MyShip) -> Result<NextShipmentResp> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let message = ContractShipmentMessage::RequestNext {
            ship_clone: ship.clone(),
            callback: sender,
            can_start_new_contract: true,
        };
        self.context
            .contract_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver.await.map_err(|e| {
            Error::General(format!("Failed to get contract request message: {}", e))
        })?;

        debug!("Got response: {:?}", resp);

        resp
    }

    async fn fail_shipment(&self, shipment: sql::ContractShipment, error: Error) -> Result<Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = ContractShipmentMessage::Failed {
            shipment,
            error,
            callback: sender,
        };
        self.context
            .contract_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get contract fail message: {}", e)))?;

        debug!("Got response: {:?}", resp);

        resp
    }

    async fn complete_shipment(
        &self,
        shipment: sql::ContractShipment,
        contract: space_traders_client::models::Contract,
    ) -> Result<()> {
        let message = ContractShipmentMessage::Finished { contract, shipment };

        debug!("Sending message: {:?}", message);

        self.context
            .contract_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    async fn purchase_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: &sql::ContractShipment,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToPurchase),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.nav_to(
            &shipment.purchase_symbol,
            true,
            sql::TransactionReason::Contract(shipment.contract_id.clone()),
            &self.context,
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
            sql::TransactionReason::Contract(shipment.contract_id.clone()),
        )
        .await?;

        Ok(())
    }

    async fn deliver_cargo(
        &self,
        ship: &mut ship::MyShip,
        shipment: sql::ContractShipment,
    ) -> Result<(
        space_traders_client::models::Contract,
        sql::ContractShipment,
    )> {
        ship.status = ship::ShipStatus::Contract {
            contract_id: Some(shipment.contract_id.clone()),
            run_id: Some(shipment.id),
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            shipping_status: Some(ship::ShippingStatus::InTransitToDelivery),
            waiting_for_manager: false,
        };

        ship.notify().await;

        ship.nav_to(
            &shipment.destination_symbol,
            true,
            sql::TransactionReason::Contract(shipment.contract_id.clone()),
            &self.context,
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
