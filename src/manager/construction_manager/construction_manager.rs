use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use chrono::Utc;
use log::debug;
use space_traders_client::models;

use crate::{
    config::CONFIG,
    error::{Error, Result},
    manager::{construction_manager::message::ConstructionMessage, Manager},
    ship,
    sql::{self, DatabaseConnector},
    types::ConductorContext,
};

use super::{message::ConstructionManagerMessage, messanger::ConstructionManagerMessanger};

#[derive(Debug)]
pub struct ConstructionManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
    running_shipments: Vec<sql::ConstructionShipment>,
}

impl ConstructionManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
        ConstructionManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ConstructionManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
            // current_contract: None,
            running_shipments: Vec::new(),
        }
    }

    async fn get_budget(&self) -> Result<i64> {
        let agent = sql::Agent::get_last_by_symbol(&self.context.database_pool, &CONFIG.symbol)
            .await?
            .ok_or(Error::General("Agent not found".to_string()))?;
        Ok(agent.credits - 1_000_000)
    }

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
            let waypoints = sql::Waypoint::get_by_system(&self.context.database_pool, system)
                .await?
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
                    .map(|m| sql::ConstructionMaterial::from(m, &waypoint.symbol))
                    .collect::<Vec<_>>();

                sql::ConstructionMaterial::insert_bulk(&self.context.database_pool, &materials)
                    .await?;
            }
        }

        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
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

    async fn handle_construction_message(
        &mut self,
        message: super::message::ConstructionMessage,
    ) -> Result<()> {
        match message {
            ConstructionMessage::RequestNextShipment {
                ship_clone,
                callback,
            } => {
                let next_shipment = self.request_next_shipment(ship_clone).await;

                debug!("Got shipment: {:?}", next_shipment);

                let _send = callback.send(next_shipment);
            }
            ConstructionMessage::FailedShipment {
                shipment,
                error,
                callback,
            } => {
                let _fail = self.fail_shipment(shipment, &error).await?;

                let _sed = callback.send(error).unwrap();
            }
            ConstructionMessage::FinishedShipment {
                construction,
                shipment,
            } => {
                let _complete = self.finish_shipment(construction, shipment).await?;
            }
        }

        Ok(())
    }

    async fn request_next_shipment(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> std::result::Result<super::NextShipmentResp, crate::error::Error> {
        let construction_materials =
            sql::ConstructionMaterial::get_unfulfilled(&self.context.database_pool).await?;

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
            return Ok(super::NextShipmentResp::ComeBackLater);
        }

        let next_material: &sql::ConstructionMaterial = construction_materials
            .iter()
            .min_by_key(|c| ((c.fulfilled as f64 / c.required as f64) * 10000.0) as i64)
            .unwrap();

        let trade_symbol = next_material.trade_symbol.clone();

        let (purchase_volume, remaining) =
            self.calculate_purchase_volume(&ship_clone, &next_material, &trade_symbol);
        debug!("Calculated purchase volume: {}", purchase_volume);

        let purchase_symbol = self
            .get_purchase_waypoint(&trade_symbol, &ship_clone.nav.system_symbol)
            .await?;
        debug!("Obtained purchase waypoint: {:?}", purchase_symbol);

        if let Some(purchase_price) = purchase_symbol.1 {
            debug!("Calculated purchase price: {}", purchase_price);
            let total_price = (purchase_price * remaining) as i64;

            let budget = self.get_budget().await?;
            debug!("Calculated budget: {}", budget);
            if total_price > budget {
                debug!(
                    "Not enough budget for purchase has {} needed {}",
                    total_price, budget
                );
                return Ok(super::NextShipmentResp::ComeBackLater);
            }
        }

        let next_shipment = sql::ConstructionShipment {
            id: 0,
            material_id: next_material.id,
            construction_site_waypoint: next_material.waypoint_symbol.clone(),
            ship_symbol: ship_clone.symbol.clone(),
            trade_symbol,
            units: purchase_volume,
            purchase_waypoint: purchase_symbol.0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            status: sql::ShipmentStatus::InTransit,
        };

        let id = sql::ConstructionShipment::insert_new(&self.context.database_pool, &next_shipment)
            .await?;

        let sql_shipment = sql::ConstructionShipment::get_by_id(&self.context.database_pool, id)
            .await?
            .ok_or(crate::error::Error::General(format!(
                "Failed to get shipment by id: {}",
                id
            )))?;

        self.running_shipments.push(sql_shipment.clone());

        return Ok(super::NextShipmentResp::Shipment(sql_shipment));
    }

    async fn fail_shipment(
        &mut self,
        mut shipment: crate::sql::ConstructionShipment,
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

        shipment.status = sql::ShipmentStatus::Failed;

        sql::ConstructionShipment::insert(&self.context.database_pool, &shipment).await?;

        Ok(())
    }

    async fn finish_shipment(
        &mut self,
        construction: space_traders_client::models::Construction,
        mut shipment: crate::sql::ConstructionShipment,
    ) -> Result<()> {
        let materials = construction
            .materials
            .iter()
            .map(|m| sql::ConstructionMaterial::from(m, &construction.symbol))
            .collect::<Vec<_>>();

        sql::ConstructionMaterial::insert_bulk(&self.context.database_pool, &materials).await?;

        let pos = self
            .running_shipments
            .iter()
            .position(|s| s.id == shipment.id);

        if let Some(pos) = pos {
            self.running_shipments.remove(pos);
        }

        shipment.status = sql::ShipmentStatus::Delivered;

        sql::ConstructionShipment::insert(&self.context.database_pool, &shipment).await?;

        Ok(())
    }

    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShip,
        shipment: &sql::ConstructionMaterial,
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
    ) -> Result<(String, Option<i32>)> {
        debug!(
            "Getting purchase waypoint for trade symbol: {:?}",
            trade_symbol
        );
        let market_trades =
            sql::MarketTrade::get_last_by_symbol(&self.context.database_pool, trade_symbol)
                .await?
                .into_iter()
                .filter(|t| t.waypoint_symbol.starts_with(system_symbol))
                .collect::<Vec<_>>();
        let market_trade_goods: HashMap<(models::TradeSymbol, String), sql::MarketTradeGood> =
            sql::MarketTradeGood::get_last_by_symbol(&self.context.database_pool, trade_symbol)
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
        &self.cancel_token
    }
}
