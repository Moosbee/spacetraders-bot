use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    time::Duration,
};

use anyhow::{Context, Error, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use space_traders_client::models::{self, Contract, ContractDeliverGood, TradeSymbol, Waypoint};
use tokio_util::sync::CancellationToken;

use crate::{
    api::Api,
    config::CONFIG,
    ship,
    sql::{self, DatabaseConnector},
};

// Constants

pub struct ContractFleet {
    context: super::types::ConductorContext,
    please_stop: CancellationToken,
}

impl ContractFleet {
    #[allow(dead_code)]
    pub fn new_box(context: super::types::ConductorContext) -> Box<Self> {
        Box::new(ContractFleet {
            context,
            please_stop: CancellationToken::new(),
        })
    }

    async fn run_contract_workers(&self) -> Result<()> {
        info!("Starting contract workers");

        if !CONFIG.contracts.active {
            info!("contract workers not active, exiting");

            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(CONFIG.contracts.start_sleep_duration)).await;

        let contract_ships = self.get_contract_ships()?;
        let primary_ship = &contract_ships[0];

        // Process existing contracts
        let mut contract_queue = self.get_unfulfilled_contracts().await?;
        while let Some(contract) = contract_queue.pop_front() {
            self.process_contract(&contract, primary_ship, -1).await?;
        }

        // Process new contracts
        for i in 0..CONFIG.contracts.max_contracts {
            if self.please_stop.is_cancelled() {
                break;
            }
            info!("Negotiating new contract");
            let next_contract = self.negotiate_next_contract(&contract_ships).await?;
            self.process_contract(&next_contract, primary_ship, i)
                .await?;
        }

        info!("Contract workers done");
        Ok(())
    }

    async fn process_contract(
        &self,
        contract: &Contract,
        ship_symbol: &str,
        num: i32,
    ) -> Result<()> {
        info!("Processing contract: {:?}", contract);

        if !self.is_contract_viable(contract).await? {
            return Err(Error::msg("Contract is not viable"));
        }

        let contract = if !contract.accepted {
            info!("Accepting contract: {}", contract.id);
            let accept_data = self.context.api.accept_contract(&contract.id).await?;
            sql::Agent::insert(
                &self.context.database_pool,
                &sql::Agent::from(*accept_data.data.agent),
            )
            .await?;

            accept_data.data.contract.as_ref().clone()
        } else {
            contract.clone()
        };

        let waypoints = self
            .context
            .all_waypoints
            .get(&self.get_system_symbol(&contract))
            .context("System not found")?
            .clone();

        sql::Contract::insert_contract(&self.context.database_pool, contract.clone()).await?;

        let finished_contract = self
            .execute_trade_contract(
                contract.clone(),
                ship_symbol.to_string(),
                waypoints.values().cloned().collect(),
                num,
            )
            .await
            .unwrap();

        if self.can_fulfill_trade(&finished_contract) {
            let fulfill_contract_data = self
                .context
                .api
                .fulfill_contract(&finished_contract.id)
                .await?;

            sql::Agent::insert(
                &self.context.database_pool,
                &sql::Agent::from(*fulfill_contract_data.data.agent),
            )
            .await?;

            sql::Contract::insert_contract(
                &self.context.database_pool,
                (*fulfill_contract_data.data.contract).clone(),
            )
            .await?;

            info!("Contract fulfilled: {}", contract.id);
            Ok(())
        } else {
            Err(Error::msg("Contract could not be fulfilled"))
        }
    }

    async fn execute_trade_contract(
        &self,
        contract: Contract,
        ship_symbol: String,
        waypoints: Vec<Waypoint>,
        num: i32,
    ) -> Result<Contract> {
        let procurements = contract
            .terms
            .deliver
            .clone()
            .context("No delivery terms")?;
        let mut current_contract = contract;
        let waypoints_map: HashMap<_, _> = waypoints
            .iter()
            .map(|w| (w.symbol.clone(), w.clone()))
            .collect();

        let mut ship = self
            .context
            .ship_manager
            .get_mut(&ship_symbol)
            .context("Ship not found")?;

        ship.role = ship::Role::Contract(Some((current_contract.id.clone(), num)));
        ship.notify().await;

        for mut procurement in procurements {
            let trade_symbol = TradeSymbol::from_str(&procurement.trade_symbol)?;
            let buy_waypoint = self.get_purchase_waypoint(&procurement).await?;

            info!("Buy waypoint: {} {:?}", buy_waypoint, procurement);
            while procurement.units_fulfilled < procurement.units_required {
                current_contract = self
                    .handle_procurement_cycle(
                        &mut ship,
                        &current_contract,
                        &procurement,
                        trade_symbol,
                        &buy_waypoint,
                        &waypoints_map,
                        sql::TransactionReason::Contract(current_contract.id.clone()),
                    )
                    .await?;

                // sql::update_contract(&self.context.database_pool, &current_contract).await;
                sql::Contract::insert_contract(
                    &self.context.database_pool,
                    current_contract.clone(),
                )
                .await?;

                procurement = self.get_updated_procurement(&current_contract, &procurement)?;
            }
        }

        ship.role = ship::Role::Contract(None);
        ship.notify().await;

        Ok(current_contract)
    }

    async fn negotiate_next_contract(
        &self,
        contract_ships: &Vec<String>,
    ) -> Result<Contract, Error> {
        for ship in contract_ships.iter() {
            let mut current_ship = self.context.ship_manager.get_mut(ship).unwrap();
            let current_nav = current_ship.nav.get_status();
            if current_nav != models::ShipNavStatus::InTransit {
                current_ship.ensure_docked(&self.context.api).await?;

                let next_contract = self
                    .context
                    .api
                    .negotiate_contract(&current_ship.symbol)
                    .await?;
                return Ok(*next_contract.data.contract);
            }
        }

        Err(Error::msg(
            "No ships available to negotiate contracts. Could not negotiate next contract",
        ))
    }

    // Helper functions
    fn get_system_symbol(&self, contract: &Contract) -> String {
        let waypoint_symbol = &contract.terms.deliver.as_ref().unwrap()[0].destination_symbol;
        Api::system_symbol(waypoint_symbol)
    }

    fn get_contract_ships(&self) -> Result<Vec<String>> {
        let ships: Vec<String> = self
            .context
            .ship_roles
            .iter()
            .filter(|(_, role)| {
                let role = if let ship::Role::Contract(_) = role {
                    true
                } else {
                    false
                };

                role
            })
            .map(|(symbol, _)| symbol.clone())
            .collect();

        if ships.is_empty() {
            return Err(Error::msg("No ships assigned to contract role"));
        }
        Ok(ships)
    }

    async fn get_unfulfilled_contracts(&self) -> Result<VecDeque<Contract>> {
        let contracts = self.context.api.get_all_contracts(20).await?;
        Ok(VecDeque::from(
            contracts
                .into_iter()
                .filter(|c| !c.fulfilled && self.is_in_deadline(c))
                .collect::<Vec<_>>(),
        ))
    }

    async fn is_contract_viable(&self, contract: &Contract) -> Result<bool> {
        if !self.is_in_deadline(contract) {
            return Ok(false);
        }

        match contract.r#type {
            models::contract::Type::Procurement => self.check_procurement_viability(contract).await,
            _ => Ok(false),
        }
    }

    fn is_in_deadline(&self, contract: &Contract) -> bool {
        DateTime::parse_from_rfc3339(&contract.terms.deadline)
            .map(|deadline| Utc::now() < deadline)
            .unwrap_or(false)
    }

    async fn check_procurement_viability(&self, contract: &Contract) -> Result<bool> {
        let Some(deliveries) = &contract.terms.deliver else {
            return Ok(false);
        };

        let market_trade_goods = sql::MarketTrade::get_last(&self.context.database_pool).await?;

        for delivery in deliveries {
            if delivery.units_required <= delivery.units_fulfilled {
                continue;
            }

            let symbol = TradeSymbol::from_str(&delivery.trade_symbol)?;
            if !market_trade_goods
                .iter()
                .any(|trade| trade.symbol == symbol)
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn get_purchase_waypoint(&self, procurement: &ContractDeliverGood) -> Result<String> {
        let trade_symbol = TradeSymbol::from_str(&procurement.trade_symbol)?;
        let market_trades =
            sql::MarketTrade::get_last_by_symbol(&self.context.database_pool, &trade_symbol)
                .await?;
        let market_trade_goods =
            sql::MarketTradeGood::get_last_by_symbol(&self.context.database_pool, &trade_symbol)
                .await?;

        if market_trades.len() == market_trade_goods.len() {
            let best_market = market_trade_goods
                .iter()
                .min_by_key(|trade| trade.purchase_price)
                .context("No valid market found")?;

            debug!("Selected market: {:?}", best_market);
            Ok(best_market.waypoint_symbol.clone())
        } else {
            let first_market = market_trades.first().context("No valid market found")?;
            debug!("Selected market: {:?}", first_market);
            Ok(first_market.waypoint_symbol.clone())
        }
    }

    fn get_updated_procurement(
        &self,
        contract: &Contract,
        current_procurement: &ContractDeliverGood,
    ) -> Result<ContractDeliverGood> {
        contract
            .terms
            .deliver
            .as_ref()
            .context("No delivery terms")?
            .iter()
            .find(|p| {
                p.trade_symbol == current_procurement.trade_symbol
                    && p.destination_symbol == current_procurement.destination_symbol
            })
            .cloned()
            .context("Procurement not found")
    }

    fn can_fulfill_trade(&self, contract: &Contract) -> bool {
        contract.terms.deliver.as_ref().map_or(false, |deliveries| {
            deliveries
                .iter()
                .all(|d| d.units_fulfilled >= d.units_required)
        })
    }

    async fn handle_procurement_cycle(
        &self,
        ship: &mut ship::MyShip,
        contract: &Contract,
        procurement: &ContractDeliverGood,
        trade_symbol: TradeSymbol,
        buy_waypoint: &str,
        waypoints: &HashMap<String, Waypoint>,
        reason: sql::TransactionReason,
    ) -> Result<Contract> {
        if !ship.cargo.has(&trade_symbol) {
            ship.nav_to(
                buy_waypoint,
                true,
                waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                reason.clone(),
            )
            .await
            .unwrap();
            ship.update_market(&self.context.api, &self.context.database_pool)
                .await?;

            ship.ensure_docked(&self.context.api).await.unwrap();

            let purchase_volume = self.calculate_purchase_volume(ship, procurement);
            ship.purchase_cargo(
                &self.context.api,
                trade_symbol,
                purchase_volume,
                &self.context.database_pool,
                reason.clone(),
            )
            .await
            .unwrap();
        }

        ship.nav_to(
            &procurement.destination_symbol,
            true,
            waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            reason,
        )
        .await
        .unwrap();
        ship.ensure_docked(&self.context.api).await.unwrap();

        let cargo_amount = ship.cargo.get_amount(&trade_symbol);
        let delivery_result = ship
            .deliver_contract(&contract.id, trade_symbol, cargo_amount, &self.context.api)
            .await
            .unwrap();

        Ok(*delivery_result.data.contract)
    }

    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShip,
        procurement: &ContractDeliverGood,
    ) -> i32 {
        let remaining_required = procurement.units_required - procurement.units_fulfilled;
        (ship.cargo.capacity - ship.cargo.units).min(remaining_required)
    }
}

impl super::types::Conductor for ContractFleet {
    /// Main contract conductor that manages contract operations
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_contract_workers().await })
    }

    fn get_name(&self) -> String {
        "ContractFleet".to_string()
    }

    fn get_cancel_token(&self) -> CancellationToken {
        self.please_stop.clone()
    }
}
