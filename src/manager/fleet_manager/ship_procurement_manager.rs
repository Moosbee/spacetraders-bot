use std::{
    collections::{BTreeSet, HashMap, HashSet},
    num::NonZeroU32,
};

use database::DatabaseConnectorAsync;
use ship::autopilot::JumpGateRouter;
use tracing::{debug, warn};
use utils::get_system_symbol;

use crate::{
    error::{self, Result},
    manager::{
        Manager,
        fleet_manager::{ship_capabilities::ShipCapabilities, ship_worth::ShipWorth},
    },
    utils::ConductorContext,
};

pub struct ShipProcurementManager {
    fast_cancel_token: tokio_util::sync::CancellationToken,
    slow_cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    jump_gate: Option<ship::autopilot::JumpGateRouterCache>,
}

impl ShipProcurementManager {
    pub fn new(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
    ) -> Self {
        debug!("Creating new ShipProcurementManager");
        Self {
            fast_cancel_token,
            slow_cancel_token,
            context,
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
        let lim =
            governor::RateLimiter::direct(governor::Quota::per_second(NonZeroU32::new(1).unwrap()));

        while !self.slow_cancel_token.is_cancelled() {
            self.context
                .shipyard_ship_reservation
                .wait_for_ships()
                .await?;

            lim.until_ready().await;

            let shipyard_ship_reservation = self.context.shipyard_ship_reservation.clone();

            let purchase_reservations = self.handle_procurement(&shipyard_ship_reservation).await?;

            let mut error = None;

            for purchase_reservation in purchase_reservations {
                if error.is_some() {
                    self.context
                        .budget_manager
                        .cancel_reservation(
                            &self.context.database_pool,
                            purchase_reservation.reservation.id,
                        )
                        .await?;
                    continue;
                }
                let worked = self.purchase_ship(purchase_reservation).await;
                if let Err(e) = worked {
                    tracing::error!(error = %e, "Failed to purchase ship");
                    error = Some(e);
                }
            }

            if let Some(e) = error {
                return Err(e);
            }
        }

        Ok(())
    }

    async fn handle_procurement<'a>(
        &mut self,
        shipyard_ship_reservation: &'a super::ShipyardShipReservation,
    ) -> Result<Vec<ShipPurchaseReservation<'a>>> {
        let open_assignments = database::ShipAssignment::get_open_assignments(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let all_shipyard_ships = database::ShipyardShip::get_last_paginated(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let fleet_ids = open_assignments
            .iter()
            .map(|assignment| assignment.fleet_id)
            .collect::<HashSet<_>>();

        let fleets = database::Fleet::get_by_ids(&self.context.database_pool, fleet_ids).await?;

        let assignment_system_symbols: HashSet<String> = fleets
            .iter()
            .map(|f| f.1.system_symbol.clone())
            .collect::<HashSet<_>>();

        let shipyard_system_symbols: HashSet<String> = all_shipyard_ships
            .iter()
            .map(|shipyard_ship| shipyard_ship.waypoint_symbol.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|waypoint| get_system_symbol(&waypoint))
            .collect::<HashSet<_>>();

        let travel_costs: HashMap<(String, String), (i64, i32, f64)> = {
            let mut price_getter =
                ship::autopilot::TravelPriceCache::new(self.context.database_pool.clone());
            let jump_gate_nav = self.get_jump_navigator().await?;

            Self::calculate_travel_cost(
                &assignment_system_symbols,
                &shipyard_system_symbols,
                jump_gate_nav,
                &mut price_getter,
            )
            .await?
        };

        let ship_frames = database::FrameInfo::get_all(
            &self.context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
        .into_iter()
        .map(|f| (f.symbol, f))
        .collect::<HashMap<_, _>>();

        let all_shipyard_ships = all_shipyard_ships
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

        let ship_purchase_percentile =
            { self.context.config.read().await.ship_purchase_percentile };

        let reserved_waypoints = shipyard_ship_reservation.reserve_all().await?;

        let waypoints = reserved_waypoints
            .iter()
            .map(|(waypoint, _)| waypoint.clone())
            .collect::<HashSet<_>>();

        let ships_to_purchase = Self::get_ships_to_purchase(
            &open_assignments,
            &all_shipyard_ships,
            &fleets,
            &travel_costs,
            &waypoints,
            ship_purchase_percentile,
        );

        let mut reservations: Vec<ShipPurchaseReservation> = Vec::new();

        for ship_purchase_info in ships_to_purchase {
            let reservation = self
                .context
                .budget_manager
                .reserve_funds_with_remain(
                    &self.context.database_pool,
                    ship_purchase_info.total_price,
                    ship_purchase_info.assignment.credits_threshold as i64,
                )
                .await;
            if let Err(error) = reservation {
                if let crate::error::Error::NotEnoughFunds { .. } = error {
                    tracing::debug!("failed to reserve funds for ship: {}", error);
                    continue;
                } else {
                    return Err(error);
                }
            }

            let reservation = reservation.unwrap();

            let waypoint_reservation = reserved_waypoints
                .get(&ship_purchase_info.shipyard_ship.waypoint_symbol)
                .ok_or_else(|| error::Error::General("WaypointReservation".to_string()))?
                .clone();

            let ship_reservation = ShipPurchaseReservation {
                reservation,
                shipyard_ship: ship_purchase_info.shipyard_ship.clone(),
                assignment: ship_purchase_info.assignment.clone(),
                waypoint_reservation,
            };

            reservations.push(ship_reservation);
        }

        Ok(reservations)
    }

    async fn calculate_travel_cost(
        assignment_system_symbols: &HashSet<String>,
        shipyard_system_symbols: &HashSet<String>, // is smaller
        jump_gate_nav: &mut ship::autopilot::JumpGateRouterCache,
        price_getter: &mut impl ship::autopilot::TravelPriceCalc,
    ) -> Result<HashMap<(String, String), (i64, i32, f64)>> {
        let mut travel_costs: HashMap<(String, String), (i64, i32, f64)> = HashMap::new();

        for shipyard_system in shipyard_system_symbols {
            let tree = jump_gate_nav.generate_jump_tree(&shipyard_system, true);

            for assignment_system in assignment_system_symbols {
                let route = JumpGateRouter::get_route(&tree, &assignment_system, &shipyard_system);
                let mut total_cost = 0;
                let mut total_distance = 0.0;
                let mut total_jumps = 0;
                for conn in route {
                    let cost: i32 = price_getter
                        .get_antimatter_price(&conn.start_system)
                        .await?;
                    total_cost += cost as i64;
                    total_distance += conn.conn.distance;
                    total_jumps += 1;
                }

                travel_costs.insert(
                    (shipyard_system.clone(), assignment_system.clone()),
                    (total_cost, total_jumps, total_distance),
                );
            }
        }

        Ok(travel_costs)
    }

    /// will fail if no ship is at the shipyard
    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::ship_procurement_manager::purchase_ship",
        skip(self),
        err(Debug)
    )]
    async fn purchase_ship(&self, ship_reservation: ShipPurchaseReservation<'_>) -> Result<()> {
        let shipyard_ship = ship_reservation.shipyard_ship;
        let assignment = ship_reservation.assignment;
        let reservation = ship_reservation.reservation;
        let waypoint_reservation = ship_reservation.waypoint_reservation;

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

        drop(waypoint_reservation);

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
        name = "spacetraders::manager::fleet_manager::calculate_global_prices"
    )]
    fn calculate_global_prices<'a>(
        assignments: &'a [database::ShipAssignment],
        all_shipyard_ships: &'a [(database::ShipyardShip, ShipCapabilities)],
        fleets: &'a HashMap<i32, database::Fleet>,
        travel_cost: &HashMap<(String, String), (i64, i32, f64)>,
    ) -> BTreeSet<AssignmentPriceComparison<'a>> {
        let assignments = assignments
            .iter()
            .map(|assignment| {
                Self::calculate_assignment_prices(
                    assignment,
                    all_shipyard_ships,
                    fleets,
                    travel_cost,
                )
            })
            // .sorted_by(|a, b| a.cmp(b))
            .collect::<BTreeSet<_>>();

        assignments
    }

    fn calculate_assignment_prices<'a>(
        assignment: &'a database::ShipAssignment,
        all_shipyard_ships: &'a [(database::ShipyardShip, ShipCapabilities)],
        fleets: &'a HashMap<i32, database::Fleet>,
        travel_cost: &HashMap<(String, String), (i64, i32, f64)>,
    ) -> AssignmentPriceComparison<'a> {
        let shipyard_ships = all_shipyard_ships
            .iter()
            .filter(|(_shipyard_ship, capability)| capability.capable(assignment))
            .map(|(shipyard_ship, _)| shipyard_ship)
            .filter_map(|shipyard_ship| {
                ShipWorth::new(
                    assignment,
                    shipyard_ship,
                    fleets.get(&assignment.fleet_id)?,
                    travel_cost,
                )
            })
            .collect::<BTreeSet<_>>();

        AssignmentPriceComparison {
            assignment,
            global_price: shipyard_ships,
        }
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

    fn get_ships_to_purchase<'a>(
        open_assignments: &'a [database::ShipAssignment],
        all_shipyard_ships: &'a [(database::ShipyardShip, ShipCapabilities)],
        fleets: &'a HashMap<i32, database::Fleet>,
        travel_costs: &'_ HashMap<(String, String), (i64, i32, f64)>,
        waypoints: &'_ HashSet<String>,
        ship_purchase_percentile: f64,
    ) -> Vec<ShipWorth<'a>> {
        let mut available_ships = all_shipyard_ships
            .iter()
            .filter(|(shipyard_ship, _)| waypoints.contains(&shipyard_ship.waypoint_symbol))
            .map(|(shipyard_ship, _)| {
                (
                    shipyard_ship.waypoint_symbol.clone(),
                    shipyard_ship.ship_type.clone(),
                )
            })
            .collect::<HashSet<_>>();

        let global_prices = Self::calculate_global_prices(
            &open_assignments,
            &all_shipyard_ships,
            &fleets,
            &travel_costs,
        );

        let mut ships_to_purchase = Vec::new();

        for assignment in global_prices {
            let ship_count = assignment.global_price.len();
            let purchasable = assignment
                .global_price
                .into_iter()
                .take(((ship_count as f64) * (ship_purchase_percentile / 100.0)).ceil() as usize)
                .filter(|sh| sh.total_price < (sh.assignment.max_purchase_price as i64))
                // .filter()
                .find(|sh| {
                    available_ships.contains(&(
                        sh.shipyard_ship.waypoint_symbol.clone(),
                        sh.shipyard_ship.ship_type.clone(),
                    ))
                });

            if let Some(purchasable) = purchasable {
                available_ships.remove(&(
                    purchasable.shipyard_ship.waypoint_symbol.clone(),
                    purchasable.shipyard_ship.ship_type.clone(),
                ));
                ships_to_purchase.push(purchasable);
                if available_ships.is_empty() {
                    break;
                }
            }
        }

        ships_to_purchase
    }
}

#[derive(Debug)]
struct ShipPurchaseReservation<'a> {
    shipyard_ship: database::ShipyardShip,
    assignment: database::ShipAssignment,
    reservation: database::ReservedFund,
    waypoint_reservation: super::WaypointReservation<'a>,
}

#[derive(Debug, Clone)]
struct AssignmentPriceComparison<'a> {
    assignment: &'a database::ShipAssignment,
    global_price: BTreeSet<ShipWorth<'a>>,
}

impl Eq for AssignmentPriceComparison<'_> {}

impl PartialEq for AssignmentPriceComparison<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.assignment.priority == other.assignment.priority
            && self.assignment.id == other.assignment.id
    }
}

impl Ord for AssignmentPriceComparison<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.assignment
            .priority
            .cmp(&other.assignment.priority)
            .then_with(|| self.assignment.id.cmp(&other.assignment.id))
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
