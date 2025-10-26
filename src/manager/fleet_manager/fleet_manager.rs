use std::collections::{HashMap, HashSet};

use database::DatabaseConnector;
use tracing::debug;

use crate::{
    error::Result,
    manager::{fleet_manager::ship_capabilities::ShipCapabilities, Manager},
    utils::ConductorContext,
};

use super::{message::FleetManagerMessage, messanger::FleetManagerMessanger};

pub struct FleetManager {
    cancel_token: tokio_util::sync::CancellationToken,
    receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    context: ConductorContext,
}

impl FleetManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<FleetManagerMessage>,
        FleetManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(24);
        debug!("Created FleetManager channel");

        (receiver, FleetManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    ) -> Self {
        debug!("Creating new FleetManager");
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::fleet_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_fleet_worker(&mut self) -> std::result::Result<(), crate::error::Error> {
        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
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
        match message {
            crate::manager::fleet_manager::message::FleetMessage::ScrapperAtShipyard {
                waypoint_symbol,
                ship_symbol,
                callback,
            } => {
                let erg = self
                    .handle_scrapper_at_shipyard(&waypoint_symbol, &ship_symbol)
                    .await;
                callback.send(ship_symbol).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
                erg?;
            }
            crate::manager::fleet_manager::message::FleetMessage::GetNewAssignments {
                callback,
                ship_clone,
                temp,
            } => {
                let erg = self.get_new_assignment(&ship_clone, temp).await?;
                callback.send(erg).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_scrapper_at_shipyard",
        skip(self)
    )]
    async fn handle_scrapper_at_shipyard(
        &mut self,
        waypoint_symbol: &str,
        _ship_symbol: &str,
    ) -> Result<()> {
        return Ok(());
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::get_new_assignment",
        skip(self, ship_clone)
    )]
    async fn get_new_assignment(
        &mut self,
        ship_clone: &ship::MyShip,
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

        let open_assignments =
            database::ShipAssignment::get_open_assignments(&self.context.database_pool).await?;

        // filter assignments based on ship capabilities (e.g. required cargo space, required fuel, required equipment, ...)

        let mut open_possible_assignments = open_assignments
            .into_iter()
            .filter(|assignment| ShipCapabilities::can_assign(ship_clone, assignment))
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

        let connections =
            ship::autopilot::jump_gate_nav::generate_all_connections(&self.context.database_pool)
                .await?
                .into_iter()
                .filter(|c| !c.under_construction_a && !c.under_construction_b)
                .collect::<Vec<_>>();
        let jump_gate = ship::autopilot::jump_gate_nav::JumpPathfinder::new(connections);

        let target_systems = fleets
            .iter()
            .map(|f| f.1.system_symbol.clone())
            .collect::<HashSet<_>>();

        let start_system: &str = &ship_clone.nav.system_symbol;

        let conns = target_systems
            .iter()
            .map(|end_system| (end_system, jump_gate.find_route(start_system, end_system)))
            .map(|f| (f.0.clone(), f.1.iter().map(|conn| conn.cost).sum::<f64>()))
            .collect::<HashMap<_, _>>();

        // sort them based on priority, distance to system and "fitness" i.e. a ship with 130 cargo should be better assigned the one which needs 100 cargo than the one which needs 40 cargo

        open_possible_assignments.sort_by(|a, b| {
            let fleet_a = &fleets.get(&a.fleet_id);
            let fleet_b = &fleets.get(&b.fleet_id);
            let priority_a = a.priority;
            let priority_b = b.priority;
            let distance_a = conns.get(&fleet_a.unwrap().system_symbol).unwrap();
            let distance_b = conns.get(&fleet_b.unwrap().system_symbol).unwrap();
            priority_a.cmp(&priority_b).then_with(|| {
                distance_a
                    .partial_cmp(distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // pick the best assignment

        let best_assignment = open_possible_assignments.first().cloned();

        // assign it to the ship
        if let Some(best_assignment) = best_assignment {
            // assign it to the ship
            let mut ship_info =
                database::ShipInfo::get_by_symbol(&self.context.database_pool, &ship_clone.symbol)
                    .await?
                    .ok_or(crate::error::Error::General(
                        "No ship info found".to_string(),
                    ))?;

            if temp {
                ship_info.temp_assignment_id = Some(best_assignment.id);
            } else {
                ship_info.assignment_id = Some(best_assignment.id);
            }

            database::ShipInfo::insert(&self.context.database_pool, &ship_info).await?;

            Ok(Some(best_assignment.id))
        } else {
            Ok(None)
        }
    }
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
        &self.cancel_token
    }
}
