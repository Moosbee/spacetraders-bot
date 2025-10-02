use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use database::DatabaseConnector;
use log::debug;
use space_traders_client::models::{self};

use crate::{
    error::{self, Error, Result},
    manager::{
        contract_manager::ContractShipmentMessage,
        fleet_manager::message::{Budget, Priority, RequestedShipType, RequiredShips},
        Manager,
    },
    utils::ConductorContext,
};

use super::{
    message::ContractManagerMessage, messanger::ContractManagerMessanger, NextShipmentResp,
};

#[derive(Debug)]
pub struct ContractManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
    current_contract: Option<models::Contract>,
    running_shipments: Vec<database::ContractShipment>,
    reserved_funds: Option<database::ReservedFund>,
}

impl ContractManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ContractManagerMessage>,
        ContractManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created ContractManager channel");

        (receiver, ContractManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
    ) -> Self {
        debug!("Creating new ContractManager");
        Self {
            cancel_token,
            context,
            receiver,
            current_contract: None,
            running_shipments: Vec::new(),
            reserved_funds: None,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::contract_manager_worker",
        skip(self)
    )]
    async fn run_contract_worker(&mut self) -> Result<()> {
        debug!("Starting contract worker");
        let contracts = self.get_unfulfilled_contracts().await?;

        for contract in contracts.iter() {
            debug!("Contract found: {}", contract.id);
            let in_db =
                database::Contract::get_by_id(&self.context.database_pool, &contract.id).await?;

            if let Some(existing_contract) = in_db {
                if let Some(reserved_fund_id) = existing_contract.reserved_fund {
                    let fund = database::ReservedFund::get_by_id(
                        &self.context.database_pool,
                        &reserved_fund_id,
                    )
                    .await?;
                    self.reserved_funds = fund;
                }
            }

            database::Contract::insert_contract(
                &self.context.database_pool,
                contract.clone(),
                self.reserved_funds.as_ref().map(|r| r.id),
            )
            .await?;
        }

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

        debug!("Starting contract worker loop");

        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
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
            ContractShipmentMessage::RequestNext {
                ship_clone,
                can_start_new_contract,
                callback,
            } => {
                let next_shipment = self
                    .request_next_shipment(ship_clone, can_start_new_contract)
                    .await;

                debug!("Got contract: {:?}", next_shipment);

                let _send = callback.send(next_shipment);
            }
            ContractShipmentMessage::Failed {
                shipment,
                error,
                callback,
            } => {
                self.failed_shipment(shipment, &error).await?;

                callback.send(Ok(error)).unwrap();
            }
            ContractShipmentMessage::Finished { contract, shipment } => {
                self.finished_shipment(contract, shipment).await?;
            }
            ContractShipmentMessage::GetRunning { callback } => {
                callback.send(Ok(self.running_shipments.clone())).unwrap();
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::contract_manager_get_required_ships",
        skip(context)
    )]
    pub async fn get_required_ships(context: &ConductorContext) -> Result<RequiredShips> {
        let db_ships = database::ShipInfo::get_by_role(
            &context.database_pool,
            &database::ShipInfoRole::Contract,
        )
        .await?;
        let all_ships = context
            .ship_manager
            .get_all_clone()
            .await
            .into_values()
            .filter(|ship| {
                (ship.role == database::ShipInfoRole::Contract
                    || db_ships.iter().any(|db_ship| db_ship.symbol == ship.symbol))
                    && ship.cargo.capacity >= 40
            })
            .collect::<Vec<_>>();

        let headquarters = { context.run_info.read().await.headquarters.clone() };

        let mut contract_systems = HashSet::new();
        if contract_systems.is_empty() {
            contract_systems.insert(headquarters.clone());
        }
        let ships = if all_ships.is_empty() {
            HashMap::from_iter(
                vec![(
                    contract_systems.iter().next().unwrap().clone(),
                    vec![(
                        RequestedShipType::Transporter,
                        Priority::High,
                        Budget::Medium,
                        database::ShipInfoRole::Contract,
                    )],
                )]
                .into_iter(),
            )
        } else {
            HashMap::new()
        };
        Ok(RequiredShips { ships })
    }

    async fn request_next_shipment(
        &mut self,
        ship_clone: ship::MyShip,
        can_start_new_contract: bool,
    ) -> Result<NextShipmentResp> {
        debug!("Requesting next shipment for ship: {:?}", ship_clone.symbol);

        debug!("Current shipments {}", self.running_shipments.len());

        if self.running_shipments.is_empty()
            && self.current_contract.is_some()
            && self
                .current_contract
                .as_ref()
                .map(|c| self.can_fulfill_trade(c))
                .unwrap_or(false)
        {
            let id = self
                .current_contract
                .as_ref()
                .map(|c| c.id.clone())
                .unwrap();
            tracing::info!(contract_id = &id, "Can Fulfilled contract");

            let fulfill_contract_data = self.context.api.fulfill_contract(&id).await?;

            self.current_contract = None;

            database::Contract::insert_contract(
                &self.context.database_pool,
                *fulfill_contract_data.data.contract,
                self.reserved_funds.as_ref().map(|r| r.id),
            )
            .await?;

            database::Agent::insert(
                &self.context.database_pool,
                &database::Agent::from(*fulfill_contract_data.data.agent),
            )
            .await?;
        }

        if self.current_contract.is_none() {
            if can_start_new_contract {
                let has_done = self.get_new_contract(&ship_clone).await?;

                debug!("Has done: {}", has_done);

                if !has_done {
                    return Ok(NextShipmentResp::ComeBackLater);
                }
            } else {
                return Ok(NextShipmentResp::ComeBackLater);
            }
        }
        if !self.current_contract.as_ref().unwrap().accepted {
            let resp = self
                .context
                .api
                .accept_contract(&self.current_contract.as_ref().unwrap().id)
                .await?;

            self.current_contract = Some(*resp.data.contract.clone());

            database::Contract::insert_contract(
                &self.context.database_pool,
                *resp.data.contract,
                self.reserved_funds.as_ref().map(|r| r.id),
            )
            .await?;

            database::Agent::insert(
                &self.context.database_pool,
                &database::Agent::from(*resp.data.agent),
            )
            .await?;
        }

        let shipments = database::ContractShipment::get_by_ship_symbol(
            &self.context.database_pool,
            &ship_clone.symbol,
        )
        .await?
        .into_iter()
        .filter(|s| s.contract_id == self.current_contract.as_ref().unwrap().id)
        .filter(|s| s.status == database::ShipmentStatus::InTransit)
        .collect::<Vec<_>>();

        match shipments.len() {
            1 => {
                let shipment = shipments[0].clone();
                debug!("Ship already has {:?} in transit", shipment);
                self.running_shipments.push(shipment.clone());
                return Ok(NextShipmentResp::Shipment(
                    shipment,
                    self.reserved_funds.as_ref().map(|r| r.id),
                ));
            }
            _ if shipments.len() > 1 => {
                tracing::error!(
                    length = shipments.len(),
                    ship_symbol = ship_clone.symbol,
                    "Ship already has {} shipments in transit",
                    shipments.len()
                );
                panic!("Ship already has {} shipments in transit", shipments.len());
            }
            _ => {} // This arm is not necessary in this case, but it's good practice to include it
        }

        let contract = self.current_contract.as_ref().unwrap();

        let all_procurment = contract.terms.deliver.as_ref().unwrap();

        let all_procurment = all_procurment
            .iter()
            .map(|p| {
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

                let mut p = p.clone();
                let units_fulfilled = p.units_fulfilled + running;
                p.units_fulfilled = units_fulfilled.min(p.units_required);

                debug!("Calculated units fulfilled: {}", units_fulfilled);
                p
            })
            .filter(|c| {
                c.destination_symbol
                    .starts_with(&ship_clone.nav.system_symbol)
                    && c.units_fulfilled < c.units_required
            })
            .collect::<Vec<_>>();

        if all_procurment.is_empty() {
            debug!("No procurement tasks available");
            return Ok(NextShipmentResp::ComeBackLater);
        }

        let next_procurment = &all_procurment[0];
        debug!("Next procurement task: {:?}", next_procurment);

        let trade_symbol = models::TradeSymbol::from_str(&next_procurment.trade_symbol)
            .map_err(|err| Error::General(err.to_string()))?;

        let (purchase_volume, remaining) =
            self.calculate_purchase_volume(&ship_clone, next_procurment, &trade_symbol);
        debug!("Calculated purchase volume: {}", purchase_volume);

        let purchase_symbol = self
            .get_purchase_waypoint(&trade_symbol, &ship_clone.nav.system_symbol)
            .await?;
        debug!("Obtained purchase waypoint: {:?}", purchase_symbol);

        if self.reserved_funds.is_none() {
            let reservation = if let Some(purchase_price) = purchase_symbol.1 {
                debug!("Calculated purchase price: {}", purchase_price);
                let total_price = (purchase_price * remaining) as i64;

                let budget = self
                    .context
                    .budget_manager
                    .reserve_funds_with_remain(&self.context.database_pool, total_price, 30_000)
                    .await;

                debug!("Calculated budget: {:?}", budget);
                if budget.is_err() {
                    if let Err(e) = budget {
                        if let crate::error::Error::NotEnoughFunds {
                            remaining_funds,
                            required_funds,
                        } = e
                        {
                            debug!(
                                "Not enough budget for purchase has {} needed {}",
                                remaining_funds, required_funds
                            );
                            return Ok(super::NextShipmentResp::ComeBackLater);
                        } else {
                            debug!("Error reserving funds: {:?}", e);
                            return Err(e);
                        }
                    }
                }

                Some(budget.unwrap())
            } else {
                None
            };

            self.reserved_funds = reservation;

            database::Contract::update_reserved_fund(
                &self.context.database_pool,
                &contract.id,
                self.reserved_funds.as_ref().map(|r| r.id),
            )
            .await?;
        }

        let mut next_shipment = database::ContractShipment {
            contract_id: contract.id.clone(),
            trade_symbol,
            destination_symbol: next_procurment.destination_symbol.to_string(),
            units: purchase_volume,
            id: 0,
            ship_symbol: ship_clone.symbol.to_string(),
            purchase_symbol: purchase_symbol.0.to_owned(),
            status: database::ShipmentStatus::InTransit,
            ..Default::default()
        };

        let id =
            database::ContractShipment::insert_new(&self.context.database_pool, &next_shipment)
                .await?;
        debug!("Inserted new shipment with id: {}", id);

        let sql_shipment =
            database::ContractShipment::get_by_id(&self.context.database_pool, id).await?;

        next_shipment = sql_shipment;

        self.running_shipments.push(next_shipment.clone());

        Ok(NextShipmentResp::Shipment(
            next_shipment,
            self.reserved_funds.as_ref().map(|r| r.id),
        ))
    }

    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShip,
        procurement: &models::ContractDeliverGood,
        trade_symbol: &models::TradeSymbol,
    ) -> (i32, i32) {
        let remaining_required = procurement.units_required - procurement.units_fulfilled;
        (
            (ship.cargo.capacity - ship.cargo.units + ship.cargo.get_amount(trade_symbol))
                .min(remaining_required),
            remaining_required,
        )
    }

    async fn failed_shipment(
        &mut self,
        mut shipment: database::ContractShipment,
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

        shipment.status = database::ShipmentStatus::Failed;

        database::ContractShipment::insert(&self.context.database_pool, &shipment).await?;

        Ok(())
    }

    async fn finished_shipment(
        &mut self,
        contract: models::Contract,
        mut shipment: database::ContractShipment,
    ) -> Result<()> {
        debug!("Handling finished shipment: {:?}", shipment);
        database::Contract::insert_contract(
            &self.context.database_pool,
            contract.clone(),
            self.reserved_funds.as_ref().map(|r| r.id),
        )
        .await?;

        let pos = self
            .running_shipments
            .iter()
            .position(|s| s.id == shipment.id);

        if let Some(pos) = pos {
            self.running_shipments.remove(pos);
        }

        shipment.status = database::ShipmentStatus::Delivered;

        database::ContractShipment::insert(&self.context.database_pool, &shipment).await?;

        if self.can_fulfill_trade(&contract) {
            tracing::info!(contract_id = &contract.id, "Can Fulfilled contract");

            let fulfill_contract_data = self.context.api.fulfill_contract(&contract.id).await?;

            self.current_contract = None;

            database::Contract::insert_contract(
                &self.context.database_pool,
                *fulfill_contract_data.data.contract,
                self.reserved_funds.as_ref().map(|r| r.id),
            )
            .await?;

            database::Agent::insert(
                &self.context.database_pool,
                &database::Agent::from(*fulfill_contract_data.data.agent),
            )
            .await?;

            if let Some(reserved_funds) = &self.reserved_funds {
                let transactions = database::MarketTransaction::get_by_reason(
                    &self.context.database_pool,
                    database::TransactionReason::Contract(contract.id.clone()),
                )
                .await?;
                let funds = transactions
                    .iter()
                    .filter(|t| t.r#type == models::market_transaction::Type::Purchase)
                    .map(|t| t.total_price as i64)
                    .sum();
                self.context
                    .budget_manager
                    .complete_use_reservation(&self.context.database_pool, reserved_funds.id, funds)
                    .await?;
                self.reserved_funds = None;
            }
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
        let market_trade_goods =
            database::MarketTrade::get_last(&self.context.database_pool).await?;

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
        contract.terms.deliver.as_ref().is_some_and(|deliveries| {
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
        if self.current_contract.is_some() || !self.running_shipments.is_empty() {
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

        database::Contract::insert_contract(
            &self.context.database_pool,
            contract.clone(),
            self.reserved_funds.as_ref().map(|r| r.id),
        )
        .await?;

        let viable = self.is_contract_viable(&contract).await?;
        self.current_contract = Some(contract);
        debug!("New contract negotiated: {:?}", self.current_contract);

        Ok(viable)
    }

    async fn get_purchase_waypoint(
        &self,
        trade_symbol: &models::TradeSymbol,
        system_symbol: &str,
    ) -> Result<(String, Option<i32>)> {
        debug!(
            "Getting purchase waypoint for trade symbol: {:?}",
            trade_symbol
        );
        let market_trades =
            database::MarketTrade::get_last_by_symbol(&self.context.database_pool, trade_symbol)
                .await?
                .into_iter()
                .filter(|t| t.waypoint_symbol.starts_with(system_symbol))
                .collect::<Vec<_>>();
        let market_trade_goods: HashMap<(models::TradeSymbol, String), database::MarketTradeGood> =
            database::MarketTradeGood::get_last_by_symbol(&self.context.database_pool, trade_symbol)
                .await?
                .into_iter()
                .filter(|t| t.waypoint_symbol.starts_with(system_symbol))
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
        Ok((
            first_market.0.waypoint_symbol.clone(),
            first_market.1.as_ref().map(|t| t.purchase_price),
        ))
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
