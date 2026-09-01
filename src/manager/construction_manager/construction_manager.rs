use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use chrono::Utc;
use database::{ConstructionFleetConfig, DatabaseConnectorAsync, PaginatedQuery};
use space_traders_client::models::{self};
use tracing::debug;
use utils::{WaypointCan, get_system_symbol};

use crate::{
    error::{Error, Result},
    manager::{Manager, construction_manager::message::ConstructionMessage},
    utils::ConductorContext,
};

use super::{message::ConstructionManagerMessage, messanger::ConstructionManagerMessanger};

pub type ConstructionManagerReceiver = tokio::sync::mpsc::Receiver<ConstructionManagerMessage>;
#[derive(Debug)]
pub struct ConstructionManager {
    fast_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: ConstructionManagerReceiver,
    running_shipments: Vec<database::ConstructionShipment>,
}

impl ConstructionManager {
    pub fn create() -> (ConstructionManagerReceiver, ConstructionManagerMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ConstructionManagerMessanger::new(sender))
    }

    pub fn new(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: ConstructionManagerReceiver,
    ) -> Self {
        Self {
            fast_cancel_token,
            slow_cancel_token,
            context,
            receiver,
            // current_contract: None,
            running_shipments: Vec::new(),
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::construction_manager::construction_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_construction_worker(&mut self) -> Result<()> {
        let systems_to_search_for_construction = self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .into_iter()
            .map(|s| s.1.nav.system_symbol)
            .collect::<HashSet<_>>();

        for system in systems_to_search_for_construction.iter() {
            let waypoints = database::Waypoint::get_by_system(
                &self.context.database_pool,
                system,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items
            .into_iter()
            .filter(|w| w.is_under_construction)
            .collect::<Vec<_>>();

            for waypoint in waypoints.iter() {
                let construction = self
                    .context
                    .api
                    .get_construction(&waypoint.system_symbol, &waypoint.symbol)
                    .await?;
                debug!("Got construction: {:?}", construction);

                let materials = construction
                    .data
                    .materials
                    .iter()
                    .map(|m| database::ConstructionMaterial::from(m, &waypoint.symbol))
                    .collect::<Vec<_>>();

                database::ConstructionMaterial::insert_bulk(
                    &self.context.database_pool,
                    &materials,
                )
                .await?;
            }
        }

        let fast_cancel_token = self.fast_cancel_token.clone();
        tokio::select! {
            _ = fast_cancel_token.cancelled() => {
                tracing::info!("ConstructionManager fast cancel token triggered");
                Ok(())
            },
            erg = self.run_construction_worker_loop() => erg,
        }?;

        Ok(())
    }

    async fn run_construction_worker_loop(&mut self) -> Result<()> {
        while !self.slow_cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.slow_cancel_token.cancelled() => {
                    tracing::info!("ConstructionManager slow cancel token triggered");
                    None
                }
            };
            debug!("Received ConstructionManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_construction_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::construction_manager::construction_manager_handle_construction_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_construction_message(
        &mut self,
        message: super::message::ConstructionMessage,
    ) -> Result<()> {
        self.context.construction_manager.set_busy(true);

        match message {
            ConstructionMessage::RequestNextShipment {
                ship_clone,
                callback,
                construction_config,
            } => {
                let next_shipment = self
                    .request_next_shipment(ship_clone, construction_config)
                    .await;

                debug!("Got shipment: {:?}", next_shipment);

                let _send = callback.send(next_shipment);
            }
            ConstructionMessage::FailedShipment {
                shipment,
                error,
                callback,
            } => {
                self.fail_shipment(shipment, &error).await?;

                callback.send(error).unwrap();
            }
            ConstructionMessage::FinishedShipment {
                construction,
                shipment,
            } => {
                self.finish_shipment(construction, shipment).await?;
            }
            ConstructionMessage::GetRunning { callback } => {
                callback.send(Ok(self.running_shipments.clone())).unwrap();
            }
        }
        self.context.construction_manager.set_busy(false);

        Ok(())
    }

    async fn request_next_shipment(
        &mut self,
        ship_clone: ship::MyShipCopy,
        construction_config: ConstructionFleetConfig,
    ) -> std::result::Result<super::NextShipmentResp, crate::error::Error> {
        let shipments = database::ConstructionShipment::get_all_in_transit(
            &self.context.database_pool,
            PaginatedQuery::unpaged(),
        )
        .await?
        .items;
        let running_shipments = shipments
            .iter()
            .filter(|s| s.status == database::ShipmentStatus::InTransit)
            .filter(|s| s.ship_symbol == ship_clone.symbol)
            .collect::<Vec<_>>();

        if !running_shipments.is_empty() {
            let next_shipment = running_shipments.iter().min_by_key(|s| s.id).unwrap();

            self.running_shipments.push((**next_shipment).clone());

            return Ok(super::NextShipmentResp::Shipment((**next_shipment).clone()));
        }

        let construction_materials = database::ConstructionMaterial::get_unfulfilled(
            &self.context.database_pool,
            PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let construction_materials = construction_materials
            .into_iter()
            .map(|mut c| {
                let running = self
                    .running_shipments
                    .iter()
                    .filter(|s| s.material_id == c.id)
                    .map(|s| s.units)
                    .sum::<i32>();

                c.fulfilled = (c.fulfilled + running).min(c.required);

                c
            })
            .filter(|c| c.fulfilled < c.required)
            .filter(|c| c.waypoint_symbol.starts_with(&ship_clone.nav.system_symbol))
            .collect::<Vec<_>>();

        if construction_materials.is_empty() {
            debug!("No more constructions");
            return Ok(super::NextShipmentResp::ComeBackLater);
        }

        // let next_material: &database::ConstructionMaterial = construction_materials
        //     .iter()
        //     .min_by_key(|c| ((c.fulfilled as f64 / c.required as f64) * 10000.0) as i64)
        //     .unwrap();

        // (construction_material, trade_symbol, (waypoint_symbol, price, supply), purchase_volume, remaining, total_price)
        let mut materials: Vec<(
            database::ConstructionMaterial,
            models::TradeSymbol,
            (String, Option<i32>, Option<models::SupplyLevel>),
            i32,
            i32,
            i32,
        )> = Vec::new();

        for material in construction_materials.iter() {
            let trade_symbol = material.trade_symbol;

            let (purchase_volume, remaining) =
                self.calculate_purchase_volume(&ship_clone, material, &trade_symbol);
            debug!("Calculated purchase volume: {}", purchase_volume);

            let purchase_symbol = self
                .get_purchase_waypoint(&trade_symbol, &ship_clone.nav.system_symbol)
                .await?;
            debug!("Obtained purchase waypoint: {:?}", purchase_symbol);
            let purchase_price = purchase_symbol.1.unwrap_or(10);

            materials.push((
                material.clone(),
                trade_symbol,
                purchase_symbol,
                purchase_volume,
                remaining,
                purchase_volume * purchase_price,
            ));
        }

        let (
            next_material,
            trade_symbol,
            purchase_symbol,
            purchase_volume,
            remaining,
            _total_price,
        ) = materials
            .into_iter()
            .min_by(|a, b| {
                compare_construction_materials(a, b, &construction_config.construction_mode)
            })
            .unwrap();

        let reservation = if let Some(purchase_price) = purchase_symbol.1 {
            debug!("Calculated purchase price: {}", purchase_price);
            let total_price = (purchase_price * (purchase_volume * 2).min(remaining)) as i64;

            let budget = self
                .context
                .budget_manager
                .reserve_funds_with_remain(&self.context.database_pool, total_price, 1_000_000)
                .await;

            debug!("Calculated budget: {:?}", budget);
            if budget.is_err()
                && let Err(e) = budget
            {
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

            Some(budget.unwrap())
        } else {
            None
        };

        let next_shipment = database::ConstructionShipment {
            id: 0,
            material_id: next_material.id,
            construction_site_waypoint: next_material.waypoint_symbol.clone(),
            ship_symbol: ship_clone.symbol.clone(),
            trade_symbol,
            units: purchase_volume,
            purchase_waypoint: purchase_symbol.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: database::ShipmentStatus::InTransit,
            reserved_fund: reservation.map(|r| r.id),
        };

        let id =
            database::ConstructionShipment::insert_new(&self.context.database_pool, &next_shipment)
                .await?;

        let sql_shipment =
            database::ConstructionShipment::get_by_id(&self.context.database_pool, &id)
                .await?
                .ok_or(crate::error::Error::General(format!(
                    "Failed to get shipment by id: {}",
                    id
                )))?;

        self.running_shipments.push(sql_shipment.clone());

        Ok(super::NextShipmentResp::Shipment(sql_shipment))
    }

    async fn fail_shipment(
        &mut self,
        mut shipment: database::ConstructionShipment,
        _error: &crate::error::Error,
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

        if let Some(reserved_fund_id) = shipment.reserved_fund {
            self.context
                .budget_manager
                .cancel_reservation(&self.context.database_pool, reserved_fund_id)
                .await?;
        }

        database::ConstructionShipment::upsert(&self.context.database_pool, &shipment).await?;

        Ok(())
    }

    async fn finish_shipment(
        &mut self,
        construction: space_traders_client::models::Construction,
        mut shipment: database::ConstructionShipment,
    ) -> Result<()> {
        let materials = construction
            .materials
            .iter()
            .map(|m| database::ConstructionMaterial::from(m, &construction.symbol))
            .collect::<Vec<_>>();

        database::ConstructionMaterial::insert_bulk(&self.context.database_pool, &materials)
            .await?;

        let pos = self
            .running_shipments
            .iter()
            .position(|s| s.id == shipment.id);

        if let Some(pos) = pos {
            self.running_shipments.remove(pos);
        }

        if let Some(reserved_fund_id) = shipment.reserved_fund {
            let transactions = database::MarketTransaction::get_by_reason(
                &self.context.database_pool,
                database::TransactionReason::Construction(shipment.id),
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;
            let funds = transactions
                .iter()
                .filter(|t| t.r#type == models::market_transaction::Type::Purchase)
                .map(|t| t.total_price as i64)
                .sum();
            self.context
                .budget_manager
                .complete_use_reservation(&self.context.database_pool, reserved_fund_id, funds)
                .await?;
        }

        shipment.status = database::ShipmentStatus::Delivered;

        database::ConstructionShipment::upsert(&self.context.database_pool, &shipment).await?;

        let waypoint = shipment.construction_site_waypoint.clone();

        if materials
            .iter()
            .filter(|c| c.waypoint_symbol == waypoint)
            .all(|c| c.fulfilled == c.required)
        {
            let system_waypoint = get_system_symbol(&waypoint);
            let wp = self
                .context
                .api
                .get_waypoint(&system_waypoint, &waypoint)
                .await?;
            let waypoint = (&(*wp.data)).into();
            database::Waypoint::upsert(&self.context.database_pool, &waypoint).await?;
            if waypoint.is_jump_gate() {
                self.context
                    .fleet_manager
                    .populate_from_jump_gate(waypoint.symbol)
                    .await?;
            }
        }

        Ok(())
    }

    /// returns purchase volume and remaining volume
    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShipCopy,
        shipment: &database::ConstructionMaterial,
        trade_symbol: &models::TradeSymbol,
    ) -> (i32, i32) {
        let remaining_required = shipment.required - shipment.fulfilled;
        (
            (ship.cargo.capacity - ship.cargo.units + ship.cargo.get_amount(trade_symbol))
                .min(remaining_required),
            remaining_required,
        )
    }

    async fn get_purchase_waypoint(
        &self,
        trade_symbol: &models::TradeSymbol,
        system_symbol: &str,
    ) -> Result<(String, Option<i32>, Option<models::SupplyLevel>)> {
        debug!(
            "Getting purchase waypoint for trade symbol: {:?}",
            trade_symbol
        );
        let market_trades = database::MarketTrade::get_last_by_symbol(
            &self.context.database_pool,
            trade_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
        .into_iter()
        .filter(|t| t.waypoint_symbol.starts_with(system_symbol))
        .collect::<Vec<_>>();
        let market_trade_goods: HashMap<(models::TradeSymbol, String), database::MarketTradeGood> =
            database::MarketTradeGood::get_last_by_symbol(
                &self.context.database_pool,
                trade_symbol,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items
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
            first_market.1.as_ref().map(|t| t.supply),
        ))
    }
}

fn compare_construction_materials(
    // (construction_material, trade_symbol, (waypoint_symbol, price, supply), purchase_volume, remaining, total_price)
    a: &(
        database::ConstructionMaterial,
        models::TradeSymbol,
        (String, Option<i32>, Option<models::SupplyLevel>),
        i32,
        i32,
        i32,
    ),
    // (construction_material, trade_symbol, (waypoint_symbol, price, supply), purchase_volume, remaining, total_price)
    b: &(
        database::ConstructionMaterial,
        models::TradeSymbol,
        (String, Option<i32>, Option<models::SupplyLevel>),
        i32,
        i32,
        i32,
    ),
    construction_mode: &database::ConstructionMode,
) -> Ordering {
    match construction_mode {
        database::ConstructionMode::LowestPurchaseCost => a.5.cmp(&b.5),
        database::ConstructionMode::LowestAbsoluteProgress => a.0.fulfilled.cmp(&b.0.fulfilled),
        database::ConstructionMode::LowestPercentProgress => {
            let a_percent = (a.0.fulfilled as f64 / a.0.required as f64) * 100.0;
            let b_percent = (b.0.fulfilled as f64 / b.0.required as f64) * 100.0;
            a_percent.partial_cmp(&b_percent).unwrap()
        }
        database::ConstructionMode::BestPurchaseSupply => {
            a.2.2
                .unwrap_or(models::SupplyLevel::Moderate)
                .cmp(&b.2.2.unwrap_or(models::SupplyLevel::Moderate))
                .reverse()
                .then_with(|| a.5.cmp(&b.5))
        }
    }
}

impl Manager for ConstructionManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_construction_worker().await })
    }

    fn get_name(&self) -> &str {
        "ConstructionManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.slow_cancel_token
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use space_traders_client::models::{SupplyLevel, TradeSymbol};

    use super::*;

    fn material(fulfilled: i32, required: i32) -> database::ConstructionMaterial {
        database::ConstructionMaterial {
            id: 1,
            waypoint_symbol: "X1-TEST".to_string(),
            trade_symbol: TradeSymbol::Iron,
            required,
            fulfilled,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn entry(
        fulfilled: i32,
        required: i32,
        supply: Option<SupplyLevel>,
        purchase_volume: i32,
        remaining: i32,
        total_price: i32,
    ) -> (
        database::ConstructionMaterial,
        TradeSymbol,
        (String, Option<i32>, Option<SupplyLevel>),
        i32,
        i32,
        i32,
    ) {
        (
            material(fulfilled, required),
            TradeSymbol::Iron,
            ("X1-MARKET".to_string(), Some(100), supply),
            purchase_volume,
            remaining,
            total_price,
        )
    }

    #[test]
    fn lowest_purchase_cost_orders_by_total_price() {
        let cheap = entry(0, 100, None, 10, 100, 500);
        let expensive = entry(0, 100, None, 10, 100, 900);

        assert_eq!(
            compare_construction_materials(
                &cheap,
                &expensive,
                &database::ConstructionMode::LowestPurchaseCost
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_construction_materials(
                &expensive,
                &cheap,
                &database::ConstructionMode::LowestPurchaseCost
            ),
            Ordering::Greater
        );
        assert_eq!(
            compare_construction_materials(
                &cheap,
                &cheap,
                &database::ConstructionMode::LowestPurchaseCost
            ),
            Ordering::Equal
        );
    }

    #[test]
    fn lowest_absolute_progress_orders_by_fulfilled() {
        let less_progress = entry(10, 100, None, 5, 90, 100);
        let more_progress = entry(50, 100, None, 5, 50, 100);

        assert_eq!(
            compare_construction_materials(
                &less_progress,
                &more_progress,
                &database::ConstructionMode::LowestAbsoluteProgress
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_construction_materials(
                &more_progress,
                &less_progress,
                &database::ConstructionMode::LowestAbsoluteProgress
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn lowest_percent_progress_orders_by_percent_not_absolute() {
        let low_pct = entry(10, 100, None, 5, 90, 100);
        let high_pct = entry(20, 100, None, 5, 80, 100);

        assert_eq!(
            compare_construction_materials(
                &low_pct,
                &high_pct,
                &database::ConstructionMode::LowestPercentProgress
            ),
            Ordering::Less
        );
    }

    #[test]
    fn lowest_percent_progress_uses_ratio() {
        // 5/10 = 50% vs 40/100 = 40%: absolute progress favors the latter, percent favors it too.
        let high_pct = entry(5, 10, None, 5, 5, 100);
        let low_pct = entry(40, 100, None, 5, 60, 100);

        assert_eq!(
            compare_construction_materials(
                &high_pct,
                &low_pct,
                &database::ConstructionMode::LowestPercentProgress
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn best_purchase_supply_prefers_higher_supply() {
        let low = entry(0, 100, Some(SupplyLevel::Limited), 10, 100, 100);
        let high = entry(0, 100, Some(SupplyLevel::Abundant), 10, 100, 100);

        assert_eq!(
            compare_construction_materials(
                &low,
                &high,
                &database::ConstructionMode::BestPurchaseSupply
            ),
            Ordering::Greater
        );
        assert_eq!(
            compare_construction_materials(
                &high,
                &low,
                &database::ConstructionMode::BestPurchaseSupply
            ),
            Ordering::Less
        );
    }

    #[test]
    fn best_purchase_supply_treats_missing_supply_as_moderate() {
        let none = entry(0, 100, None, 10, 100, 100);
        let scarce = entry(0, 100, Some(SupplyLevel::Scarce), 10, 100, 100);
        let moderate = entry(0, 100, Some(SupplyLevel::Moderate), 10, 100, 100);

        // None defaults to Moderate, which outranks Scarce.
        assert_eq!(
            compare_construction_materials(
                &scarce,
                &none,
                &database::ConstructionMode::BestPurchaseSupply
            ),
            Ordering::Greater
        );
        // None and Moderate are equivalent.
        assert_eq!(
            compare_construction_materials(
                &none,
                &moderate,
                &database::ConstructionMode::BestPurchaseSupply
            ),
            Ordering::Equal
        );
    }
}
