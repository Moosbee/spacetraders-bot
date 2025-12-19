use std::collections::{HashMap, HashSet};

use database::DatabaseConnector;
use itertools::Itertools;
use tracing::{debug, warn};

use crate::{
    error::Result,
    manager::{
        fleet_manager::{ship_capabilities::ShipCapabilities, ship_worth::ShipWorth},
        Manager,
    },
    utils::ConductorContext,
};

use super::{message::FleetManagerMessage, messanger::FleetManagerMessanger};

pub struct FleetManager {
    cancel_token: tokio_util::sync::CancellationToken,
    receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    context: ConductorContext,
    jump_gate: Option<ship::autopilot::jump_gate_nav::JumpPathfinder>,
}

impl FleetManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<FleetManagerMessage>,
        FleetManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
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
        self.context.fleet_manager.set_busy(true);

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
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateAssignments {
                callback,
            } => {
                self.re_generate_assignments(RegenFleetBy::All).await?;
                callback.send(()).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateFleetAssignments {
                callback,
                fleet_id,
            } => {
                self.re_generate_assignments(RegenFleetBy::Fleet(fleet_id))
                    .await?;
                callback.send(()).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
            crate::manager::fleet_manager::message::FleetMessage::ReGenerateSystemAssignments {
                callback,
                system_symbol,
            } => {
                self.re_generate_assignments(RegenFleetBy::System(system_symbol))
                    .await?;
                callback.send(()).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
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
                callback.send(()).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
            crate::manager::fleet_manager::message::FleetMessage::PopulateFromJumpGate {
                callback,
                jump_gate_symbol,
            } => {
                self.handle_populate_from_jump_gate(&jump_gate_symbol)
                    .await?;
                callback.send(()).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
        }
        self.context.fleet_manager.set_busy(false);

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
        let stop = { self.context.config.read().await.ship_purchase_stop };
        if stop {
            return Ok(());
        }

        let ships_purchasable = database::ShipyardShip::get_last_by_waypoint(
            &self.context.database_pool,
            waypoint_symbol,
        )
        .await?;

        let open_assignments =
            database::ShipAssignment::get_open_assignments(&self.context.database_pool).await?;

        let assignments_count = open_assignments.len();

        // get all the open assignments that can be fulfilled from this shipyard

        let ship_frames = database::FrameInfo::get_all(&self.context.database_pool)
            .await?
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

                        let capable = ship_capabilities.capable(assignment);
                        // debug!(capable, ship_capabilities=?ship_capabilities,assignment=?assignment,"Ship Compatibility");
                        capable
                    } else {
                        false
                    }
                })
            })
            .collect::<Vec<_>>();

        debug!(
            "Got {} fulfillable assignments from {} open assignments",
            fulfillable_assignments.len(),
            assignments_count
        );

        // get for those assignments all other shipyard and shipyard_ships

        let all_shipyard_ships = database::ShipyardShip::get_last(&self.context.database_pool)
            .await?
            .into_iter()
            .filter_map(|shipyard_ship| {
                let ship_frame = ship_frames.get(&shipyard_ship.frame_type);
                if let Some(ship_frame) = ship_frame {
                    let ship_capabilities = ShipCapabilities::get_shipyard_ship_capabilities(
                        &shipyard_ship,
                        ship_frame,
                    );
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

        let current_money = self.context.budget_manager.get_spendable_funds().await;

        let antimatter_price = { self.context.config.read().await.antimatter_price as i64 };

        let percentile = { self.context.config.read().await.ship_purchase_percentile };

        let jump_gate = self.get_jump_navigator().await?;

        debug!(
            all_shipyard_ships_length = all_shipyard_ships.len(),
            current_money, "Information collected"
        );

        let assignments = fulfillable_assignments
            .iter()
            .map(|assignment| {
                let shipyard_ships = all_shipyard_ships
                    .iter()
                    .filter(|(_shipyard_ship, capability)| capability.capable(assignment))
                    .map(|(shipyard_ship, _)| shipyard_ship)
                    .filter_map(|shipyard_ship| {
                        Some(ShipWorth::new(
                            assignment,
                            shipyard_ship,
                            fleets.get(&assignment.fleet_id)?,
                            jump_gate,
                            antimatter_price as i64,
                        ))
                    })
                    .filter(|sh| {
                        sh.total_price < (sh.assignment.max_purchase_price as i64)
                            && current_money - sh.total_price
                                > (sh.assignment.credits_threshold as i64)
                    })
                    .sorted_by(|a, b| a.partial_cmp(b).unwrap())
                    .collect::<Vec<_>>();

                let purchasable_subset = shipyard_ships
                    .iter()
                    .take(((shipyard_ships.len() as f32) * (percentile / 100.0)).ceil() as usize)
                    .filter(|sh| sh.shipyard_ship.waypoint_symbol == waypoint_symbol)
                    .cloned()
                    .collect::<Vec<_>>();

                (assignment, shipyard_ships, purchasable_subset)
            })
            .sorted_by(|a, b| a.0.priority.cmp(&b.0.priority))
            .collect::<Vec<_>>();

        let assignments = assignments
            .iter()
            .filter(|(_assignment, _shipyard_ships, purchasable_subset)| {
                !purchasable_subset.is_empty()
            })
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

        for (assignment, _shipyard_ships, purchasable_subset) in assignments {
            if let Some(shipyard_ship_worth) = purchasable_subset.first() {
                if used_shipyard_ships.contains(&shipyard_ship_worth.shipyard_ship.ship_type) {
                    continue;
                }

                let reservation = self
                    .context
                    .budget_manager
                    .reserve_funds_with_remain(
                        &self.context.database_pool,
                        shipyard_ship_worth.total_price,
                        assignment.credits_threshold as i64,
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
                    reservation.unwrap()
                };

                used_shipyard_ships.insert(shipyard_ship_worth.shipyard_ship.ship_type);

                self.purchase_ship(
                    shipyard_ship_worth.shipyard_ship,
                    assignment,
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
        name = "spacetraders::manager::fleet_manager::purchase_ship",
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

        database::Agent::insert(
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

        debug!(conns=?conns,"Calculated all connections");

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

    async fn get_jump_navigator(
        &mut self,
    ) -> Result<&mut ship::autopilot::jump_gate_nav::JumpPathfinder> {
        if self.jump_gate.is_none() {
            let connections = ship::autopilot::jump_gate_nav::generate_all_connections(
                &self.context.database_pool,
            )
            .await?
            .into_iter()
            .filter(|c| !c.under_construction_a && !c.under_construction_b)
            .collect::<Vec<_>>();
            let jump_gate: ship::autopilot::jump_gate_nav::JumpPathfinder =
                ship::autopilot::jump_gate_nav::JumpPathfinder::new(connections);

            self.jump_gate = Some(jump_gate);
        }
        if let Some(navigator) = &mut self.jump_gate {
            Ok(navigator)
        } else {
            Err("No jump_gate after thing".into())
        }
    }

    async fn re_generate_assignments(&mut self, by: RegenFleetBy) -> Result<()> {
        let fleets = match by {
            RegenFleetBy::All => database::Fleet::get_all(&self.context.database_pool).await?,
            RegenFleetBy::System(system_symbol) => {
                database::Fleet::get_by_system(&self.context.database_pool, &system_symbol).await?
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
            let current_assignments =
                database::ShipAssignment::get_by_fleet_id(&self.context.database_pool, fleet.id)
                    .await?;
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
        let waypoint =
            database::Waypoint::get_by_symbol(&self.context.database_pool, jump_gate_symbol)
                .await?;

        if waypoint.is_none() {
            return Err(crate::error::Error::General(format!(
                "Waypoint {} not found in database",
                jump_gate_symbol
            )));
        }

        let system_symbol = utils::get_system_symbol(jump_gate_symbol);
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
            jump_gate_symbol,
        )
        .await?;

        for gate in connections.iter() {
            let waypoint =
                database::Waypoint::get_by_symbol(&self.context.database_pool, &gate.to).await?;

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
        &self.cancel_token
    }
}
