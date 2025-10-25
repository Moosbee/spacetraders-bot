use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use chrono::Utc;
use database::DatabaseConnector;
use space_traders_client::models::{self};
use tracing::debug;
use utils::get_system_symbol;

use crate::{
    error::{Error, Result},
    manager::{
        construction_manager::message::ConstructionMessage,
        Manager,
    },
    utils::ConductorContext,
};

use super::{message::ConstructionManagerMessage, messanger::ConstructionManagerMessanger};

#[derive(Debug)]
pub struct ConstructionManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
    running_shipments: Vec<database::ConstructionShipment>,
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
            let waypoints = database::Waypoint::get_by_system(&self.context.database_pool, system)
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
                    .map(|m| database::ConstructionMaterial::from(m, &waypoint.symbol))
                    .collect::<Vec<_>>();

                database::ConstructionMaterial::insert_bulk(
                    &self.context.database_pool,
                    &materials,
                )
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

        Ok(())
    }

    // #[tracing::instrument(
    //     level = "info",
    //     name = "spacetraders::manager::construction_manager::get_required_ships",
    //     skip(context)
    // )]
    // pub async fn get_required_ships(context: &ConductorContext) -> Result<RequiredShips> {
    //     // we need one transporter(39+ cargo space) in our headquarters as long as their are unfinished constructions in the main system
    //     let db_ships = database::ShipInfo::get_by_role(
    //         &context.database_pool,
    //         &database::ShipInfoRole::Construction,
    //     )
    //     .await?;
    //     let all_ships = context
    //         .ship_manager
    //         .get_all_clone()
    //         .await
    //         .into_values()
    //         .filter(|ship| {
    //             (ship.role == database::ShipInfoRole::Construction
    //                 || db_ships.iter().any(|db_ship| db_ship.symbol == ship.symbol))
    //                 && ship.cargo.capacity >= 40
    //         })
    //         .collect::<Vec<_>>();

    //     let headquarters = { context.run_info.read().await.headquarters.clone() };

    //     let headquarters = get_system_symbol(&headquarters);

    //     let headquarter_constructions =
    //         database::Waypoint::get_by_system(&context.database_pool, &headquarters)
    //             .await?
    //             .into_iter()
    //             .filter(|w| w.is_under_construction)
    //             .collect::<Vec<_>>();
    //     debug!(
    //         "headquarters: {}, headquarter_constructions: {}, all_ships: {}",
    //         headquarters,
    //         headquarter_constructions.len(),
    //         all_ships.len()
    //     );
    //     let ships = if !headquarter_constructions.is_empty() && all_ships.is_empty() {
    //         HashMap::from_iter(
    //             vec![(
    //                 headquarters.clone(),
    //                 vec![(
    //                     RequestedShipType::Transporter,
    //                     Priority::Low,
    //                     Budget::High,
    //                     database::ShipInfoRole::Construction,
    //                 )],
    //             )]
    //             .into_iter(),
    //         )
    //     } else {
    //         HashMap::new()
    //     };
    //     Ok(RequiredShips { ships })
    // }

    async fn request_next_shipment(
        &mut self,
        ship_clone: ship::MyShip,
    ) -> std::result::Result<super::NextShipmentResp, crate::error::Error> {
        let shipments =
            database::ConstructionShipment::get_all_in_transit(&self.context.database_pool).await?;
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

        let construction_materials =
            database::ConstructionMaterial::get_unfulfilled(&self.context.database_pool).await?;

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

        let next_material: &database::ConstructionMaterial = construction_materials
            .iter()
            .min_by_key(|c| ((c.fulfilled as f64 / c.required as f64) * 10000.0) as i64)
            .unwrap();

        let trade_symbol = next_material.trade_symbol;

        let (purchase_volume, remaining) =
            self.calculate_purchase_volume(&ship_clone, next_material, &trade_symbol);
        debug!("Calculated purchase volume: {}", purchase_volume);

        let purchase_symbol = self
            .get_purchase_waypoint(&trade_symbol, &ship_clone.nav.system_symbol)
            .await?;
        debug!("Obtained purchase waypoint: {:?}", purchase_symbol);

        let reservation = if let Some(purchase_price) = purchase_symbol.1 {
            debug!("Calculated purchase price: {}", purchase_price);
            let total_price = (purchase_price * (purchase_volume * 2).min(remaining)) as i64;

            let budget = self
                .context
                .budget_manager
                .reserve_funds_with_remain(&self.context.database_pool, total_price, 1_000_000)
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
            database::ConstructionShipment::get_by_id(&self.context.database_pool, id)
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

        database::ConstructionShipment::insert(&self.context.database_pool, &shipment).await?;

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
            )
            .await?;
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

        database::ConstructionShipment::insert(&self.context.database_pool, &shipment).await?;

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
            database::Waypoint::insert(&self.context.database_pool, &((&(*wp.data)).into()))
                .await?;
        }

        Ok(())
    }

    fn calculate_purchase_volume(
        &self,
        ship: &ship::MyShip,
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
