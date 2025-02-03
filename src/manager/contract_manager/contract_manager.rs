use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use log::{debug, info};
use space_traders_client::models::{self};

use crate::{
    error::{self, Error, Result},
    manager::Manager,
    ship,
    sql::{self, DatabaseConnector},
};

#[derive(Debug)]
pub enum ContractMessage {
    RequestNextShipment {
        ship_clone: ship::MyShip,
        can_start_new_contract: bool,
        callback: tokio::sync::oneshot::Sender<Result<NextShipmentResp>>,
    },
    FailedShipment {
        shipment: sql::ContractShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<Result<crate::error::Error>>,
    },
    FinishedShipment {
        contract: models::Contract,
        shipment: sql::ContractShipment,
    },
}

#[derive(Debug, Clone)]
pub enum NextShipmentResp {
    Shipment(sql::ContractShipment),
    ComeBackLater,
}

type ContractManagerMessage = ContractMessage;

#[derive(Debug)]
pub struct ContractManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
    current_contract: Option<models::Contract>,
    running_shipments: Vec<sql::ContractShipment>,
}

#[derive(Debug, Clone)]
pub struct ContractManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ContractManagerMessage>,
}

impl ContractManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ContractManagerMessage>,
        ContractManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created ContractManager channel");

        (receiver, ContractManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
    ) -> Self {
        debug!("Creating new ContractManager");
        Self {
            cancel_token,
            context,
            receiver,
            current_contract: None,
            running_shipments: Vec::new(),
        }
    }

    async fn run_contract_worker(&mut self) -> Result<()> {
        debug!("Starting contract worker");
        let contracts = self.get_unfulfilled_contracts().await?;

        match contracts.len() {
            0 => debug!("No unfulfilled contracts found"),
            1 => {
                self.current_contract = Some(contracts[0].clone());
                debug!("Current contract set: {:?}", self.current_contract);
            }
            _ => {
                panic!("Too many contracts");
            }
        }

        while !self.cancel_token.is_cancelled() {
            let message = self.receiver.recv().await;
            debug!("Received message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_contract_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    async fn handle_contract_message(&mut self, message: ContractManagerMessage) -> Result<()> {
        debug!("Handling contract message: {:?}", message);
        match message {
            ContractMessage::RequestNextShipment {
                ship_clone,
                can_start_new_contract,
                callback,
            } => {
                let contract = self
                    .request_next_shipment(ship_clone, can_start_new_contract)
                    .await;

                let _send = callback.send(contract);
            }
            ContractMessage::FailedShipment {
                shipment,
                error,
                callback,
            } => {
                let _fail = self.failed_shipment(shipment, &error).await?;

                let _sed = callback.send(Ok(error)).unwrap();
            }
            ContractMessage::FinishedShipment { contract, shipment } => {
                let _complete = self.finished_shipment(contract, shipment).await?;
            }
        }

        Ok(())
    }

    async fn request_next_shipment(
        &mut self,
        ship_clone: ship::MyShip,
        can_start_new_contract: bool,
    ) -> Result<NextShipmentResp> {
        debug!("Requesting next shipment for ship: {:?}", ship_clone.symbol);
        if self.current_contract.is_none() {
            if can_start_new_contract {
                let has_done = self.get_new_contract(&ship_clone).await?;

                if !has_done {
                    return Ok(NextShipmentResp::ComeBackLater);
                }
            } else {
                return Ok(NextShipmentResp::ComeBackLater);
            }
        }

        let contract = self.current_contract.as_ref().unwrap();
        let all_procurment = contract.terms.deliver.as_ref().unwrap();

        let all_procurment = all_procurment
            .iter()
            .filter(|p| {
                let fulfilled = p.units_fulfilled;
                let required = p.units_required;

                let running = self
                    .running_shipments
                    .iter()
                    .filter(|s| {
                        p.trade_symbol == s.trade_symbol.to_string()
                            && s.contract_id == contract.id
                            && s.destination_symbol == p.destination_symbol
                    })
                    .map(|s| s.units)
                    .sum::<i32>();
                let remaining = required - fulfilled - running;

                remaining > 0
            })
            .collect::<Vec<_>>();

        if all_procurment.is_empty() {
            debug!("No procurement tasks available");
            return Ok(NextShipmentResp::ComeBackLater);
        }

        let next_procurment = all_procurment[0];
        debug!("Next procurement task: {:?}", next_procurment);

        let purchase_volume = self.calculate_purchase_volume(&ship_clone, next_procurment);
        debug!("Calculated purchase volume: {}", purchase_volume);

        let trade_symbol = models::TradeSymbol::from_str(&next_procurment.trade_symbol)
            .map_err(|err| Error::General(err.to_string()))?;

        let purchase_symbol = self.get_purchase_waypoint(&trade_symbol).await?;
        debug!("Obtained purchase waypoint: {}", purchase_symbol);

        let mut next_shipment = sql::ContractShipment {
            contract_id: contract.id.clone(),
            trade_symbol: trade_symbol.clone(),
            destination_symbol: next_procurment.destination_symbol.to_string(),
            units: purchase_volume,
            id: 0,
            ship_symbol: ship_clone.symbol.to_string(),
            purchase_symbol: purchase_symbol.to_owned(),
            status: sql::ShipmentStatus::InTransit,
            ..Default::default()
        };

        let id =
            sql::ContractShipment::insert_new(&self.context.database_pool, &next_shipment).await?;
        debug!("Inserted new shipment with id: {}", id);

        next_shipment.id = id;

        return Ok(NextShipmentResp::Shipment(next_shipment));
    }

    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShip,
        procurement: &models::ContractDeliverGood,
    ) -> i32 {
        let remaining_required = procurement.units_required - procurement.units_fulfilled;
        (ship.cargo.capacity - ship.cargo.units).min(remaining_required)
    }

    async fn failed_shipment(
        &mut self,
        mut shipment: sql::ContractShipment,
        _error: &error::Error,
    ) -> Result<()> {
        debug!("Handling failed shipment: {:?}", shipment);
        let pos = self
            .running_shipments
            .iter()
            .position(|s| s.id == shipment.id);

        if let Some(pos) = pos {
            self.running_shipments.remove(pos);
        }

        shipment.status = sql::ShipmentStatus::Failed;

        sql::ContractShipment::insert(&self.context.database_pool, &shipment).await?;

        Ok(())
    }

    async fn finished_shipment(
        &mut self,
        contract: models::Contract,
        mut shipment: sql::ContractShipment,
    ) -> Result<()> {
        debug!("Handling finished shipment: {:?}", shipment);
        sql::Contract::insert_contract(&self.context.database_pool, contract.clone()).await?;

        let pos = self
            .running_shipments
            .iter()
            .position(|s| s.id == shipment.id);

        if let Some(pos) = pos {
            self.running_shipments.remove(pos);
        }

        shipment.status = sql::ShipmentStatus::Delivered;

        sql::ContractShipment::insert(&self.context.database_pool, &shipment).await?;

        if self.can_fulfill_trade(&contract) {
            let fulfill_contract_data = self.context.api.fulfill_contract(&contract.id).await?;

            info!(
                "Fulfilled contract: {}",
                fulfill_contract_data.data.contract.id
            );
            self.current_contract = None;

            sql::Contract::insert_contract(
                &self.context.database_pool,
                *fulfill_contract_data.data.contract,
            )
            .await?;

            sql::Agent::insert(
                &self.context.database_pool,
                &sql::Agent::from(*fulfill_contract_data.data.agent),
            )
            .await?;
        } else {
            self.current_contract = Some(contract);
        }

        Ok(())
    }

    async fn get_unfulfilled_contracts(&self) -> Result<Vec<models::Contract>> {
        debug!("Fetching unfulfilled contracts");
        let contracts = self.context.api.get_all_contracts(20).await?;
        Ok(contracts
            .into_iter()
            .filter(|c| !c.fulfilled && self.is_in_deadline(c))
            .collect::<Vec<_>>())
    }

    async fn is_contract_viable(&self, contract: &models::Contract) -> Result<bool> {
        debug!("Checking if contract is viable: {:?}", contract);
        if !self.is_in_deadline(contract) {
            return Ok(false);
        }

        match contract.r#type {
            models::contract::Type::Procurement => self.check_procurement_viability(contract).await,
            _ => panic!("Unimplemented contract type"),
        }
    }

    async fn check_procurement_viability(&self, contract: &models::Contract) -> Result<bool> {
        let Some(deliveries) = &contract.terms.deliver else {
            return Ok(false);
        };

        debug!(
            "Checking procurement viability for deliveries: {:?}",
            deliveries
        );
        let market_trade_goods = sql::MarketTrade::get_last(&self.context.database_pool).await?;

        for delivery in deliveries {
            if delivery.units_required <= delivery.units_fulfilled {
                continue;
            }

            let symbol = models::TradeSymbol::from_str(&delivery.trade_symbol)
                .map_err(|err| Error::General(err.to_string()))?;
            if !market_trade_goods
                .iter()
                .any(|trade| trade.symbol == symbol)
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn is_in_deadline(&self, contract: &models::Contract) -> bool {
        DateTime::parse_from_rfc3339(&contract.terms.deadline)
            .map(|deadline| Utc::now() < deadline)
            .unwrap_or(false)
    }

    fn can_fulfill_trade(&self, contract: &models::Contract) -> bool {
        contract.terms.deliver.as_ref().map_or(false, |deliveries| {
            deliveries
                .iter()
                .all(|d| d.units_fulfilled >= d.units_required)
        })
    }

    /// Negotiates a new contract for the given ship, and starts it if a contract
    /// is available. If a contract is already running, this function will panic.
    ///
    /// Ship MUST be docked
    ///
    /// # Arguments
    ///
    /// * `ship_clone`: The ship to negotiate a contract for.
    ///
    /// # Return
    ///
    /// Returns `true` if a contract was started, `false` if no contract was
    /// available.
    async fn get_new_contract(&mut self, ship_clone: &ship::MyShip) -> Result<bool> {
        debug!("Negotiating new contract for ship: {:?}", ship_clone.symbol);
        if self.current_contract.is_some() || self.running_shipments.len() > 0 {
            panic!("Already running a contract");
        }

        let current_nav = ship_clone.nav.get_status();
        if current_nav != models::ShipNavStatus::Docked {
            return Err(Error::General("Ship not docked".to_string()));
        }

        let contract_resp = self
            .context
            .api
            .negotiate_contract(&ship_clone.symbol)
            .await?;

        let contract = *contract_resp.data.contract;

        let viable = !self.is_contract_viable(&contract).await?;
        self.current_contract = Some(contract);
        debug!("New contract negotiated: {:?}", self.current_contract);

        Ok(viable)
    }

    async fn get_purchase_waypoint(&self, trade_symbol: &models::TradeSymbol) -> Result<String> {
        debug!(
            "Getting purchase waypoint for trade symbol: {:?}",
            trade_symbol
        );
        let market_trades =
            sql::MarketTrade::get_last_by_symbol(&self.context.database_pool, trade_symbol).await?;
        let market_trade_goods: HashMap<(models::TradeSymbol, String), sql::MarketTradeGood> =
            sql::MarketTradeGood::get_last_by_symbol(&self.context.database_pool, trade_symbol)
                .await?
                .into_iter()
                .map(|t| ((t.symbol, t.waypoint_symbol.clone()), t))
                .collect::<HashMap<_, _>>();

        let mut trades = market_trades
            .into_iter()
            .map(|t| {
                let trade_good = market_trade_goods.get(&(t.symbol, t.waypoint_symbol.clone()));

                (t, trade_good.cloned())
            })
            .collect::<Vec<_>>();

        trades.sort_by(|a, b| {
            if let (Some(a), Some(b)) = (a.1.as_ref(), b.1.as_ref()) {
                a.purchase_price.cmp(&b.purchase_price)
            } else if a.1.is_some() {
                Ordering::Less
            } else if b.1.is_some() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let first_market = trades
            .first()
            .ok_or(Into::<Error>::into("No valid market found"))?;

        debug!("Selected market: {:?}", first_market);
        Ok(first_market.0.waypoint_symbol.clone())
    }
}

impl Manager for ContractManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_contract_worker().await })
    }

    fn get_name(&self) -> &str {
        "ContractManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
