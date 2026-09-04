use std::collections::{BTreeSet, HashMap, HashSet};

use database::DatabaseConnectorAsync;
use tracing::{debug, warn};

use crate::{
    error::Result,
    manager::{
        Manager,
        fleet_manager::{
            message::ShipProcurementMessage, messanger::ShipProcurementMessanger,
            ship_capabilities::ShipCapabilities, ship_worth::ShipWorth,
        },
    },
    utils::ConductorContext,
};

pub type ShipProcurementReceiver = tokio::sync::mpsc::Receiver<ShipProcurementMessage>;

pub struct ShipProcurementManager {
    fast_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    receiver: ShipProcurementReceiver,
    context: ConductorContext,
    jump_gate: Option<ship::autopilot::JumpGateRouterCache>,
}

impl ShipProcurementManager {
    pub fn create() -> (ShipProcurementReceiver, ShipProcurementMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created ShipProcurementManager channel");

        (receiver, ShipProcurementMessanger::new(sender))
    }

    pub fn new(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: ShipProcurementReceiver,
    ) -> Self {
        debug!("Creating new ShipProcurementManager");
        Self {
            fast_cancel_token,
            slow_cancel_token,
            context,
            receiver,
            jump_gate: None,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::ship_procurement_manager::ship_procurement_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_ship_procurement_worker(&mut self) -> Result<()> {
        let fast_cancel_token = self.fast_cancel_token.clone();

        tokio::select! {
            _ = fast_cancel_token.cancelled() => {
                tracing::info!("ShipProcurementManager fast cancel token triggered");
                Ok(())
            },
            erg = self.run_ship_procurement_worker_loop() => erg,
        }?;

        Ok(())
    }

    async fn run_ship_procurement_worker_loop(&mut self) -> Result<()> {
        while !self.slow_cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.slow_cancel_token.cancelled() => {
                    tracing::info!("ShipProcurementManager slow cancel token triggered");
                    None
                }
            };
            debug!("Received ShipProcurementManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_ship_procurement_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::ship_procurement_manager::handle_ship_procurement_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_ship_procurement_message(
        &mut self,
        message: ShipProcurementMessage,
    ) -> Result<()> {
        self.context.ship_procurement_manager.set_busy(true);
        match message {
            ShipProcurementMessage::ScrapperAtShipyard {
                waypoint_symbol,
                ship_symbol,
                callback,
            } => {
                let erg = self
                    .handle_scrapper_at_shipyard(&waypoint_symbol, &ship_symbol)
                    .await;
                if let Err(send_err) = callback.send(ship_symbol) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
                erg?;
            }
        }
        self.context.ship_procurement_manager.set_busy(false);

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::ship_procurement_manager::handle_scrapper_at_shipyard",
        skip(self),
        err(Debug)
    )]
    async fn handle_scrapper_at_shipyard(
        &mut self,
        waypoint_symbol: &str,
        _ship_symbol: &str,
    ) -> Result<()> {
        let stop = { self.context.config.read().await.ship_purchase_stop };
        if stop {
            return Ok(());
        }

        let ships_purchasable = database::ShipyardShip::get_last_by_waypoint(
            &self.context.database_pool,
            waypoint_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let open_assignments = database::ShipAssignment::get_open_assignments(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let assignments_count = open_assignments.len();

        // get all the open assignments that can be fulfilled from this shipyard

        let ship_frames = database::FrameInfo::get_all(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
        .into_iter()
        .map(|f| (f.symbol, f))
        .collect::<HashMap<_, _>>();

        let fulfillable_assignments = open_assignments
            .into_iter()
            .filter(|assignment| {
                ships_purchasable.iter().any(|shipyard_ship| {
                    let ship_frame = ship_frames.get(&shipyard_ship.frame_type);
                    if let Some(ship_frame) = ship_frame {
                        let ship_capabilities = ShipCapabilities::get_shipyard_ship_capabilities(
                            shipyard_ship,
                            ship_frame,
                        );

                        ship_capabilities.capable(assignment)
                    } else {
                        false
                    }
                })
            })
            .collect::<Vec<_>>();

        debug!(
            fulfillable_assignments_count = fulfillable_assignments.len(),
            assignments_count = assignments_count,
            "Got fulfill assignments from open assignments",
        );

        // get for those assignments all other shipyard and shipyard_ships

        let all_shipyard_ships = database::ShipyardShip::get_last_paginated(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
        .into_iter()
        .filter_map(|shipyard_ship| {
            let ship_frame = ship_frames.get(&shipyard_ship.frame_type);
            if let Some(ship_frame) = ship_frame {
                let ship_capabilities =
                    ShipCapabilities::get_shipyard_ship_capabilities(&shipyard_ship, ship_frame);
                Some((shipyard_ship, ship_capabilities))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        let fleets = database::Fleet::get_by_ids(
            &self.context.database_pool,
            fulfillable_assignments
                .iter()
                .map(|assignment| assignment.fleet_id)
                .collect::<HashSet<_>>(),
        )
        .await?;

        // per assignment calculate all the things

        debug!(
            all_shipyard_ships_length = all_shipyard_ships.len(),
            "Information collected"
        );

        let assignments = self
            .calculate_local_global_prices(
                &fulfillable_assignments,
                &all_shipyard_ships,
                &fleets,
                waypoint_symbol,
            )
            .await?;

        let assignments = assignments
            .iter()
            .filter(|comparison| !comparison.purchasable_subset.is_empty())
            .collect::<Vec<_>>();

        debug!(
          assignments_count = assignments.len(),
          fulfillable_assignments = ?fulfillable_assignments,
            "Filtered assignments",
        );

        if assignments.is_empty() {
            return Ok(());
        }

        let mut used_shipyard_ships = HashSet::new();

        for assignment_comparison in assignments {
            if let Some(shipyard_ship_worth) = assignment_comparison.purchasable_subset.first() {
                if used_shipyard_ships.contains(&shipyard_ship_worth.shipyard_ship.ship_type) {
                    continue;
                }

                let reservation = self
                    .context
                    .budget_manager
                    .reserve_funds_with_remain(
                        &self.context.database_pool,
                        shipyard_ship_worth.total_price,
                        assignment_comparison.assignment.credits_threshold as i64,
                    )
                    .await;

                let reservation = if let Err(e) = reservation {
                    if let crate::error::Error::NotEnoughFunds {
                        remaining_funds,
                        required_funds,
                    } = e
                    {
                        warn!(
                            "Not enough funds to purchase ship: {:?}. Remaining: {}, Required: {}",
                            shipyard_ship_worth, remaining_funds, required_funds
                        );
                        continue;
                    } else {
                        return Err(e);
                    }
                } else {
                    reservation?
                };

                used_shipyard_ships.insert(shipyard_ship_worth.shipyard_ship.ship_type);

                self.purchase_ship(
                    shipyard_ship_worth.shipyard_ship,
                    assignment_comparison.assignment,
                    shipyard_ship_worth.fleet,
                    &reservation,
                )
                .await?;
            }
        }

        return Ok(());
    }

    /// will fail if no ship is at the shipyard
    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::ship_procurement_manager::purchase_ship",
        skip(self),
        err(Debug)
    )]
    async fn purchase_ship(
        &self,
        shipyard_ship: &database::ShipyardShip,
        assignment: &database::ShipAssignment,
        fleet: &database::Fleet,
        reservation: &database::ReservedFund,
    ) -> Result<()> {
        let purchase_ship_response = self
            .context
            .api
            .purchase_ship(space_traders_client::models::PurchaseShipRequest {
                ship_type: shipyard_ship.ship_type,
                waypoint_symbol: shipyard_ship.waypoint_symbol.clone(),
            })
            .await?;

        self.context
            .budget_manager
            .set_current_funds(purchase_ship_response.data.agent.credits);

        self.context
            .budget_manager
            .use_reservation(
                &self.context.database_pool,
                reservation.id,
                purchase_ship_response.data.transaction.price as i64,
            )
            .await?;

        database::Agent::upsert(
            &self.context.database_pool,
            &database::Agent::from(*purchase_ship_response.data.agent),
        )
        .await?;

        let id = database::ShipyardTransaction::insert_new(
            &self.context.database_pool,
            &database::ShipyardTransaction::try_from(*purchase_ship_response.data.transaction)?,
        )
        .await?;

        ship::MyShip::update_info_db(
            (*purchase_ship_response.data.ship).clone(),
            &self.context.database_pool,
        )
        .await?;

        let mut ship_i = ship::MyShip::from_ship(
            *purchase_ship_response.data.ship,
            self.context.ship_manager.get_broadcaster(),
        );

        ship_i.purchase_id = Some(id);

        let ship_info = ship_i
            .apply_from_db_ship(self.context.database_pool.clone(), Some(assignment.id))
            .await?;

        ship_i.notify(true).await;

        ship::ShipManager::add_ship(&self.context.ship_manager, ship_i).await;

        {
            let mut ship_g = self.context.ship_manager.get_mut(&ship_info.symbol).await;
            let ship = ship_g
                .value_mut()
                .ok_or_else(|| crate::error::Error::General("Ship not found".into()))?;
            ship.notify(true).await;
        }

        self.context
            .budget_manager
            .complete_reservation(&self.context.database_pool, reservation.id)
            .await?;

        self.context.ship_tasks.start_ship(ship_info.clone()).await;

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::calculate_local_global_prices",
        skip(self),
        err(Debug)
    )]
    async fn calculate_local_global_prices<'a>(
        &mut self,
        fulfillable_assignments: &'a [database::ShipAssignment],
        all_shipyard_ships: &'a [(database::ShipyardShip, ShipCapabilities)],
        fleets: &'a HashMap<i32, database::Fleet>,
        waypoint_symbol: &'a str,
    ) -> Result<BTreeSet<AssignmentPriceComparison<'a>>> {
        let current_money = self.context.budget_manager.get_spendable_funds().await;

        let antimatter_price = { self.context.config.read().await.antimatter_price as i64 };

        let percentile = { self.context.config.read().await.ship_purchase_percentile };

        let jump_gate = self.get_jump_navigator().await?;

        let assignments = fulfillable_assignments
            .iter()
            .map(|assignment| {
                let shipyard_ships = all_shipyard_ships
                    .iter()
                    .filter(|(_shipyard_ship, capability)| capability.capable(assignment))
                    .map(|(shipyard_ship, _)| shipyard_ship)
                    .filter_map(|shipyard_ship| {
                        ShipWorth::new(
                            assignment,
                            shipyard_ship,
                            fleets.get(&assignment.fleet_id)?,
                            jump_gate,
                            antimatter_price,
                        )
                    })
                    .filter(|sh| {
                        sh.total_price < (sh.assignment.max_purchase_price as i64)
                            && current_money - sh.total_price
                                > (sh.assignment.credits_threshold as i64)
                    })
                    .collect::<BTreeSet<_>>();

                let purchasable_subset = shipyard_ships
                    .iter()
                    .take(((shipyard_ships.len() as f32) * (percentile / 100.0)).ceil() as usize)
                    .filter(|sh| sh.shipyard_ship.waypoint_symbol == waypoint_symbol)
                    .cloned()
                    .collect::<Vec<_>>();

                AssignmentPriceComparison {
                    assignment,
                    global_price: shipyard_ships,
                    purchasable_subset,
                    waypoint_symbol,
                }
            })
            // .sorted_by(|a, b| a.cmp(b))
            .collect::<BTreeSet<_>>();

        Ok(assignments)
    }

    async fn get_jump_navigator(&mut self) -> Result<&mut ship::autopilot::JumpGateRouterCache> {
        if self.jump_gate.is_none() || self.context.ship_procurement_manager.get_regen_jump_gates()
        {
            let connections =
                ship::autopilot::generate_all_connections(&self.context.database_pool).await?;
            let jump_gate_router = ship::autopilot::JumpGateRouterCache::new(
                ship::autopilot::JumpGateRouter::new(connections.0, connections.1),
            );

            self.jump_gate = Some(jump_gate_router);
            self.context
                .ship_procurement_manager
                .set_regen_jump_gates(false);
        }
        if let Some(navigator) = &mut self.jump_gate {
            Ok(navigator)
        } else {
            Err("No jump_gate after thing".into())
        }
    }
}

#[derive(Debug, Clone)]
struct AssignmentPriceComparison<'a> {
    assignment: &'a database::ShipAssignment,
    global_price: BTreeSet<ShipWorth<'a>>,
    purchasable_subset: Vec<ShipWorth<'a>>,
    waypoint_symbol: &'a str,
}

impl Eq for AssignmentPriceComparison<'_> {}

impl PartialEq for AssignmentPriceComparison<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.assignment.priority == other.assignment.priority
            && self.assignment.id == other.assignment.id
            && self.waypoint_symbol == other.waypoint_symbol
    }
}

impl Ord for AssignmentPriceComparison<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.assignment
            .priority
            .cmp(&other.assignment.priority)
            .then_with(|| self.assignment.id.cmp(&other.assignment.id))
            .then_with(|| self.waypoint_symbol.cmp(other.waypoint_symbol))
    }
}

impl PartialOrd for AssignmentPriceComparison<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Manager for ShipProcurementManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_ship_procurement_worker().await })
    }

    fn get_name(&self) -> &str {
        "ShipProcurementManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.slow_cancel_token
    }
}
