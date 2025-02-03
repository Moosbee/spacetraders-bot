use std::sync::{atomic::AtomicI32, Arc};

use log::debug;

use crate::{
    error::{Error, Result},
    manager::contract_manager::{ContractMessage, NextShipmentResp},
    ship, sql,
    workers::types::ConductorContext,
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

        let _complete_erg = self.complete_shipment(shipment, contract).await?;

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut ship::MyShip) -> Result<()> {
        ship.role = ship::Role::Manuel;
        debug!("Doing something else");
        todo!();
        // ship.notify().await;

        // Ok(())
    }

    async fn request_next_shipment(&self, ship: &ship::MyShip) -> Result<NextShipmentResp> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let message = ContractMessage::RequestNextShipment {
            ship_clone: ship.clone(),
            callback: sender,
            can_start_new_contract: true,
        };
        let _erg = self
            .context
            .contract_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get message: {}", e)))?;

        debug!("Got response: {:?}", resp);

        resp
    }

    async fn fail_shipment(&self, shipment: sql::ContractShipment, error: Error) -> Result<Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = ContractMessage::FailedShipment {
            shipment,
            error,
            callback: sender,
        };
        let _erg = self
            .context
            .contract_manager
            .sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get message: {}", e)))?;

        debug!("Got response: {:?}", resp);

        resp
    }

    async fn complete_shipment(
        &self,
        shipment: sql::ContractShipment,
        contract: space_traders_client::models::Contract,
    ) -> Result<()> {
        let message = ContractMessage::FinishedShipment {
            contract: contract,
            shipment: shipment,
        };

        debug!("Sending message: {:?}", message);

        let _erg = self
            .context
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
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .map(|w| w.clone())
            .ok_or(Error::General("System not found".to_string()))?;

        ship.nav_to(
            &shipment.purchase_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::Contract(shipment.contract_id.clone()),
        )
        .await?;

        ship.ensure_docked(&self.context.api).await?;

        let market_info = ship
            .get_market_info(&self.context.api, &self.context.database_pool)
            .await?;
        let market_trade = market_info
            .iter()
            .find(|t| t.symbol == shipment.trade_symbol)
            .unwrap();
        let current_price = market_trade.purchase_price as i64;

        let budget = pilot.get_budget().await?;

        if budget < current_price {
            return Err(Error::NotEnoughFunds {
                remaining_funds: budget,
                required_funds: current_price,
            });
        }

        let units_needed = shipment.units - ship.cargo.get_amount(&shipment.trade_symbol);

        debug!("Purchasing units: {}", units_needed);

        let _erg = ship
            .purchase_cargo(
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
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .map(|w| w.clone())
            .ok_or(Error::General("System not found".to_string()))?;

        ship.nav_to(
            &shipment.destination_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::Contract(shipment.contract_id.clone()),
        )
        .await?;

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
