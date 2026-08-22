use std::collections::{HashMap, HashSet};

use database::DatabaseConnectorAsync;
use tracing::{debug, warn};

use crate::{
    error::Result,
    manager::{Manager, fleet_manager::ship_capabilities::ShipCapabilities},
    utils::ConductorContext,
};

use super::{message::FleetManagerMessage, messanger::FleetManagerMessanger};

pub type FleetManagerReceiver = tokio::sync::mpsc::Receiver<FleetManagerMessage>;

pub struct FleetManager {
    fast_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    receiver: FleetManagerReceiver,
    context: ConductorContext,
    jump_gate: Option<ship::autopilot::JumpGateRouterCache>,
}

impl FleetManager {
    pub fn create() -> (FleetManagerReceiver, FleetManagerMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created FleetManager channel");

        (receiver, FleetManagerMessanger::new(sender))
    }

    pub fn new(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: FleetManagerReceiver,
    ) -> Self {
        debug!("Creating new FleetManager");
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
        name = "spacetraders::manager::fleet_manager::fleet_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_fleet_worker(&mut self) -> std::result::Result<(), crate::error::Error> {
        let fast_cancel_token = self.fast_cancel_token.clone();

        tokio::select! {
            _ = fast_cancel_token.cancelled() => {
                tracing::info!("FleetManager fast cancel token triggered");
                Ok(())
            },
            erg = self.run_fleet_worker_loop() => erg,
        }?;

        Ok(())
    }

    async fn run_fleet_worker_loop(&mut self) -> std::result::Result<(), crate::error::Error> {
        while !self.slow_cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.slow_cancel_token.cancelled() => {
                    tracing::info!("FleetManager slow cancel token triggered");
                    None
                }
            };
            debug!("Received FleetManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_fleet_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_fleet_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_fleet_message(&mut self, message: super::message::FleetMessage) -> Result<()> {
        self.context.fleet_manager.set_busy(true);

        match message {
            crate::manager::fleet_manager::message::FleetMessage::GetNewAssignments {
                callback,
                ship_clone,
                temp,
            } => {
                let erg = self.get_new_assignment(&ship_clone, temp).await?;
                if let Err(send_err) = callback.send(erg) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateAssignments {
                callback,
            } => {
                self.re_generate_assignments(RegenFleetBy::All).await?;
                if let Err(send_err) = callback.send(()) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateFleetAssignments {
                callback,
                fleet_id,
            } => {
                self.re_generate_assignments(RegenFleetBy::Fleet(fleet_id))
                    .await?;
                if let Err(send_err) = callback.send(()) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateSystemAssignments {
                callback,
                system_symbol,
            } => {
                self.re_generate_assignments(RegenFleetBy::System(system_symbol))
                    .await?;
                if let Err(send_err) = callback.send(()) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
            crate::manager::fleet_manager::message::FleetMessage::PopulateSystem {
                callback,
                system_symbol,
            } => {
                crate::manager::fleet_manager::fleet_population::populate_system(
                    &self.context,
                    &system_symbol,
                )
                .await?;
                self.re_generate_assignments(RegenFleetBy::System(system_symbol))
                    .await?;
                if let Err(send_err) = callback.send(()) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
            crate::manager::fleet_manager::message::FleetMessage::PopulateFromJumpGate {
                callback,
                jump_gate_symbol,
            } => {
                self.handle_populate_from_jump_gate(&jump_gate_symbol)
                    .await?;
                if let Err(send_err) = callback.send(()) {
                    warn!(send_err =? send_err, "Failed to send message");
                }
            }
        }
        self.context.fleet_manager.set_busy(false);

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::get_new_assignment",
        skip(self, ship_clone)
    )]
    async fn get_new_assignment(
        &mut self,
        ship_clone: &ship::MyShipCopy,
        temp: bool,
    ) -> Result<Option<i64>> {
        // remove current assignment

        if temp {
            database::ShipInfo::unassign_temp_ship(&self.context.database_pool, &ship_clone.symbol)
                .await?;
        } else {
            database::ShipInfo::unassign_ship(&self.context.database_pool, &ship_clone.symbol)
                .await?;
        }

        // get all "open" assignments from the database, i.e. assignments that are not yet assigned to a ship, that are not disabled and where the fleet is activated

        let open_assignments = database::ShipAssignment::get_open_assignments(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        // filter assignments based on ship capabilities (e.g. required cargo space, required fuel, required equipment, ...)

        let mut open_possible_assignments = open_assignments
            .into_iter()
            .filter(|assignment| ShipCapabilities::can_assign_ship(ship_clone, assignment))
            .collect::<Vec<_>>();

        // get fleets from the database and calculate the distance from the ship_system to the fleet system

        let fleets = database::Fleet::get_by_ids(
            &self.context.database_pool,
            open_possible_assignments
                .iter()
                .map(|assignment| assignment.fleet_id)
                .collect::<HashSet<_>>(),
        )
        .await?;

        debug!(open_possible_assignments=?open_possible_assignments,"possible assignments");

        let jump_gate = self.get_jump_navigator().await?;

        debug!(fleets=?fleets,"fleets and created jump gate navigator");

        let target_systems = fleets
            .iter()
            .map(|f| f.1.system_symbol.clone())
            .collect::<HashSet<_>>();

        let start_system: &str = &ship_clone.nav.system_symbol;

        let conns = target_systems
            .iter()
            .map(|end_system| {
                (
                    end_system.clone(),
                    jump_gate
                        .find_jump_route(start_system, end_system, true)
                        .map(|c| c.iter().map(|conn| conn.cost).sum::<f64>())
                        .unwrap_or(f64::MAX),
                )
            })
            .collect::<HashMap<_, _>>();

        debug!(conns=?conns,"Calculated all connections");

        // sort them based on priority, distance to system and "fitness" i.e. a ship with 130 cargo should be better assigned the one which needs 100 cargo than the one which needs 40 cargo

        open_possible_assignments.sort_by(|a, b| {
            let fleet_a = &fleets.get(&a.fleet_id);
            let fleet_b = &fleets.get(&b.fleet_id);
            let priority_a = a.priority;
            let priority_b = b.priority;
            let distance_a = *conns
                .get(&fleet_a.unwrap().system_symbol)
                .unwrap_or(&f64::MAX);
            let distance_b = *conns
                .get(&fleet_b.unwrap().system_symbol)
                .unwrap_or(&f64::MAX);
            priority_a.cmp(&priority_b).then_with(|| {
                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // pick the best assignment

        let best_assignment = open_possible_assignments.first().cloned();

        // assign it to the ship
        if let Some(best_assignment) = best_assignment {
            // assign it to the ship
            let mut ship_info =
                database::ShipInfo::get_by_id(&self.context.database_pool, &ship_clone.symbol)
                    .await?
                    .ok_or(crate::error::Error::General(
                        "No ship info found".to_string(),
                    ))?;

            if temp {
                ship_info.temp_assignment_id = Some(best_assignment.id);
            } else {
                ship_info.assignment_id = Some(best_assignment.id);
            }

            database::ShipInfo::upsert(&self.context.database_pool, &ship_info).await?;

            Ok(Some(best_assignment.id))
        } else {
            Ok(None)
        }
    }

    async fn get_jump_navigator(&mut self) -> Result<&mut ship::autopilot::JumpGateRouterCache> {
        if self.jump_gate.is_none() {
            let connections =
                ship::autopilot::generate_all_connections(&self.context.database_pool).await?;
            let jump_gate_router = ship::autopilot::JumpGateRouterCache::new(
                ship::autopilot::JumpGateRouter::new(connections.0, connections.1),
            );

            self.jump_gate = Some(jump_gate_router);
        }
        if let Some(navigator) = &mut self.jump_gate {
            Ok(navigator)
        } else {
            Err("No jump_gate after thing".into())
        }
    }

    async fn re_generate_assignments(&mut self, by: RegenFleetBy) -> Result<()> {
        let fleets = match by {
            RegenFleetBy::All => {
                database::Fleet::get_all(
                    &self.context.database_pool,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
            RegenFleetBy::System(system_symbol) => {
                database::Fleet::get_by_system(
                    &self.context.database_pool,
                    &system_symbol,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
            RegenFleetBy::Fleet(fleet_id) => {
                let fleet =
                    database::Fleet::get_by_id(&self.context.database_pool, fleet_id).await?;
                match fleet {
                    Some(fleet) => vec![fleet],
                    None => vec![],
                }
            }
        };

        for fleet in fleets {
            let current_assignments = database::ShipAssignment::get_by_fleet_id(
                &self.context.database_pool,
                fleet.id,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;
            let new_assignments =
                super::assignment_management::generate_fleet_assignments(&fleet, &self.context)
                    .await?;

            let assignments = super::assignment_management::fix_fleet_assignments(
                current_assignments,
                new_assignments,
            )
            .await?;

            super::assignment_management::update_fleet_assignments(&self.context, assignments)
                .await?;
        }

        Ok(())
    }

    async fn handle_populate_from_jump_gate(&mut self, jump_gate_symbol: &str) -> Result<()> {
        self.jump_gate = None;
        self.context
            .ship_procurement_manager
            .set_regen_jump_gates(true);

        let jump_gate_symbol = jump_gate_symbol.to_string();
        let waypoint =
            database::Waypoint::get_by_id(&self.context.database_pool, &jump_gate_symbol).await?;

        if waypoint.is_none() {
            return Err(crate::error::Error::General(format!(
                "Waypoint {} not found in database",
                jump_gate_symbol
            )));
        }

        let system_symbol = utils::get_system_symbol(&jump_gate_symbol);
        crate::manager::fleet_manager::fleet_population::populate_system(
            &self.context,
            &system_symbol,
        )
        .await?;

        if waypoint.unwrap().is_under_construction {
            return Ok(());
        }

        let connections = database::JumpGateConnection::get_all_from(
            &self.context.database_pool,
            &jump_gate_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        for gate in connections.iter() {
            let waypoint =
                database::Waypoint::get_by_id(&self.context.database_pool, &gate.to).await?;

            if waypoint.map(|f| f.is_under_construction).unwrap_or(true) {
                continue;
            }

            let system_symbol = utils::get_system_symbol(&gate.to);
            crate::manager::fleet_manager::fleet_population::populate_system(
                &self.context,
                &system_symbol,
            )
            .await?;
        }

        Ok(())
    }
}

enum RegenFleetBy {
    System(String),
    Fleet(i32),
    All,
}

impl Manager for FleetManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_fleet_worker().await })
    }

    fn get_name(&self) -> &str {
        "FleetManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.slow_cancel_token
    }
}
