use database::{DatabaseConnector, ShipAssignment};
use itertools::Itertools;
use space_traders_client::models::{self};
use tracing::{debug, info};
use utils::WaypointCan;

use crate::utils::ConductorContext;

const DEFAULT_PRIORITY: i32 = 100;

#[derive(Debug)]
pub struct AssignmentResult {
    pub merged_assignments: Vec<ShipAssignment>, // will be kept and updated
    pub non_merged_old_assignments: Vec<ShipAssignment>, // will need to be deleted
    pub non_merged_new_assignments: Vec<ShipAssignment>, // will need to be created
}

pub async fn update_fleet_assignments(
    context: &ConductorContext,
    result: AssignmentResult,
) -> crate::error::Result<()> {
    for assignment in result.non_merged_old_assignments {
        database::ShipAssignment::delete_by_id(&context.database_pool, assignment.id).await?;
        info!(assignment_id = assignment.id, "Deleted old ship assignment");
    }

    for assignment in result.non_merged_new_assignments {
        let created_assignment =
            database::ShipAssignment::insert_new(&context.database_pool, &assignment).await?;
        info!(
            assignment_id = created_assignment,
            "Created new ship assignment"
        );
    }

    for assignment in result.merged_assignments {
        database::ShipAssignment::insert(&context.database_pool, &assignment).await?;
        info!(
            assignment_id = assignment.id,
            "Updated merged ship assignment"
        );
    }

    Ok(())
}

pub async fn fix_fleet_assignments(
    current_assignments: Vec<ShipAssignment>,
    new_assignments: Vec<ShipAssignment>,
) -> crate::error::Result<AssignmentResult> {
    // https://en.wikipedia.org/wiki/Bipartite_graph
    // https://en.wikipedia.org/wiki/Maximum_weight_matching
    // https://en.wikipedia.org/wiki/Hungarian_algorithm

    // if current_assignments.is_empty() {
    //     return Ok(AssignmentResult {
    //         merged_assignments: vec![],
    //         non_merged_old_assignments: vec![],
    //         non_merged_new_assignments: new_assignments,
    //     });
    // }

    // if current_assignments.len() == 1
    //     && new_assignments.len() == 1
    //     && current_assignments[0].can_merge(&new_assignments[0])
    // {
    //     let mut merged = current_assignments[0].clone();
    //     merged.merge_into(&new_assignments[0]);
    //     return Ok(AssignmentResult {
    //         merged_assignments: vec![merged],
    //         non_merged_old_assignments: vec![],
    //         non_merged_new_assignments: vec![],
    //     });
    // }

    let max_assignments = current_assignments.len().max(new_assignments.len());

    let weights = pathfinding::matrix::Matrix::from_rows(
        current_assignments
            .iter()
            .map(Some)
            .pad_using(max_assignments, |_| None)
            .map(|ca| {
                new_assignments
                    .iter()
                    .map(Some)
                    .pad_using(max_assignments, |_| None)
                    .map(|na| {
                        if na.is_none() || ca.is_none() {
                            return ordered_float::OrderedFloat(1_000_001.0);
                        }
                        let ca = ca.unwrap();
                        let na = na.unwrap();

                        let can_merge = ca.can_merge(na);
                        let score = if can_merge {
                            ca.merge_score(na)
                        } else {
                            1_000_000.0
                        };
                        // AssignmentMatch {
                        //     current_assignment_index: ca,
                        //     new_assignment_index: na,
                        //     score,
                        // }
                        ordered_float::OrderedFloat(score)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<Vec<_>>>(),
    )
    .unwrap();

    // TODO will panic if more rows than columns, so more current assignments than new assignments
    let (cash_flow, assignments) = pathfinding::prelude::kuhn_munkres_min(&weights);

    debug!(
        ?cash_flow,
        ?assignments,
        ?current_assignments,
        ?new_assignments,
        ?weights,
        "Calculated optimal assignment matching"
    );

    let mut merged_assignments = vec![];
    let mut non_merged_old_assignments = vec![];
    let mut new_assignments = new_assignments
        .into_iter()
        .map(|f| (f, false))
        .collect::<Vec<_>>();

    for (assignment, new_assignment_index) in current_assignments.into_iter().zip(assignments) {
        let new_assignment = new_assignments.get_mut(new_assignment_index);
        if let Some((new_assignment, merged)) = new_assignment {
            if assignment.can_merge(new_assignment) {
                *merged = true;
                let mut merged = assignment.clone();
                merged.merge_into(new_assignment);
                merged_assignments.push(merged);
            } else {
                merged_assignments.push(assignment);
            }
        } else {
            non_merged_old_assignments.push(assignment);
        }
    }

    let non_merged_new_assignments = new_assignments
        .into_iter()
        .filter_map(|(assignment, merged)| if !merged { Some(assignment) } else { None })
        .collect::<Vec<_>>();

    Ok(AssignmentResult {
        merged_assignments,
        non_merged_old_assignments,
        non_merged_new_assignments,
    })
}

#[tracing::instrument(
    level = "debug",
    name = "spacetraders::manager::fleet_manager::assignment_management::generate_fleet_assignments",
    err(Debug)
)]
pub async fn generate_fleet_assignments(
    fleet: &database::Fleet,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    if !fleet.active {
        return Ok(vec![]);
    }

    let config = fleet.get_config()?;
    let ships = match config {
        database::FleetConfig::Trading(trading_config) => {
            generate_trading_fleet_assignments(fleet, &trading_config, context).await?
        }
        database::FleetConfig::Scraping(scraping_config) => {
            generate_scraping_fleet_assignments(fleet, &scraping_config, context).await?
        }
        database::FleetConfig::Mining(mining_config) => {
            generate_mining_fleet_assignments(fleet, &mining_config, context).await?
        }
        database::FleetConfig::Charting(charting_config) => {
            generate_charting_fleet_assignments(fleet, &charting_config, context).await?
        }
        database::FleetConfig::Construction(construction_config) => {
            generate_construction_fleet_assignments(fleet, &construction_config, context).await?
        }
        database::FleetConfig::Contract(contract_config) => {
            generate_contract_fleet_assignments(fleet, &contract_config, context).await?
        }
        database::FleetConfig::Manuel(_) => vec![],
    };
    Ok(ships)
}

async fn generate_trading_fleet_assignments(
    fleet: &database::Fleet,
    trading_config: &database::TradingFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let waypoints = database::Waypoint::get_by_system(&context.database_pool, &fleet.system_symbol)
        .await?
        .into_iter()
        .filter(|wp| wp.is_marketplace() || wp.is_shipyard())
        .count();

    // let market_trades =
    //     database::MarketTradeGood::get_last_by_system(&context.database_pool, &fleet.system_symbol)
    //         .await?;

    let ship_counts = (waypoints as f64 * trading_config.ship_market_ratio).floor() as u32;

    let min_range = 300; // todo get minimum range to get to every waypoint in the system using cruse, see percolation theory and minimum spanning tree and Kruskal's algorithm

    let min_cargo = trading_config.min_cargo_space; // todo get cargo according to the current trade volume

    let ships = (0..ship_counts)
        .map(|i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: 100_000,
            priority: if i == 0 {
                DEFAULT_PRIORITY - 5 // first ship has higher priority, ensure at least one ship is always active
            } else {
                DEFAULT_PRIORITY
            },
            disabled: false,
            range_min: min_range,
            cargo_min: min_cargo,
            survey: false,
            extractor: false,
            siphon: false,
            warp_drive: false,
        })
        .collect();

    Ok(ships)
}

async fn generate_mining_fleet_assignments(
    fleet: &database::Fleet,
    mining_config: &database::MiningFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    // Placeholder for mining fleet assignment generation logic

    let waypoints =
        database::Waypoint::get_by_system(&context.database_pool, &fleet.system_symbol).await?;

    let mining_waypoints_count = waypoints.iter().filter(|wp| wp.is_minable()).count();

    let waypoint_count = if mining_config.ignore_engineered_asteroids {
        let engineered_mining_waypoints = waypoints
            .iter()
            .filter(|wp| wp.waypoint_type == models::WaypointType::EngineeredAsteroid)
            .count();
        mining_waypoints_count - engineered_mining_waypoints
    } else {
        mining_waypoints_count
    } as i32;

    let mining_waypoint = mining_config.mining_waypoints.min(waypoint_count);

    let siphon_waypoints_count = waypoints.iter().filter(|wp| wp.is_sipherable()).count() as i32;

    let siphon_waypoint = mining_config.syphon_waypoints.min(siphon_waypoints_count);

    let mining_ships_count = mining_waypoint * mining_config.miners_per_waypoint;
    let survey_ships_count = mining_waypoint * mining_config.surveyers_per_waypoint;
    let siphon_ships_count = siphon_waypoint * mining_config.siphoners_per_waypoint;
    let transporter_ships_count =
        (mining_waypoint + siphon_waypoint) * mining_config.mining_transporters_per_waypoint;

    let mining_ships = (0..mining_ships_count).map(|_i| ShipAssignment {
        id: 0, // This should be set appropriately
        fleet_id: fleet.id,
        priority: DEFAULT_PRIORITY,
        max_purchase_price: 1_000_000,
        credits_threshold: 100_000,
        disabled: false,
        range_min: 0,
        cargo_min: mining_config.min_mining_cargo_space,
        survey: false,
        extractor: true,
        siphon: false,
        warp_drive: false,
    });

    let survey_ships = (0..survey_ships_count).map(|_i| ShipAssignment {
        id: 0, // This should be set appropriately
        fleet_id: fleet.id,
        priority: DEFAULT_PRIORITY,
        max_purchase_price: 1_000_000,
        credits_threshold: 100_000,
        disabled: false,
        range_min: 0,
        cargo_min: 0,
        survey: true,
        extractor: false,
        siphon: false,
        warp_drive: false,
    });

    let siphon_ships = (0..siphon_ships_count).map(|_i| ShipAssignment {
        id: 0, // This should be set appropriately
        fleet_id: fleet.id,
        priority: DEFAULT_PRIORITY,
        max_purchase_price: 1_000_000,
        credits_threshold: 100_000,
        disabled: false,
        range_min: 0,
        cargo_min: mining_config.min_siphon_cargo_space,
        survey: false,
        extractor: false,
        siphon: true,
        warp_drive: false,
    });

    let transporter_ships = (0..transporter_ships_count).map(|_i| ShipAssignment {
        id: 0, // This should be set appropriately
        fleet_id: fleet.id,
        max_purchase_price: 1_000_000,
        credits_threshold: 100_000,
        priority: DEFAULT_PRIORITY - 1, // transporters have higher priority
        disabled: false,
        range_min: 0,
        cargo_min: mining_config.min_transporter_cargo_space,
        survey: false,
        extractor: false,
        siphon: false,
        warp_drive: false,
    });

    let ships = mining_ships
        .chain(survey_ships)
        .chain(siphon_ships)
        .chain(transporter_ships)
        .collect();

    Ok(ships)
}

async fn generate_scraping_fleet_assignments(
    fleet: &database::Fleet,
    scraping_config: &database::ScrapingFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let waypoint_counts =
        database::Waypoint::get_by_system(&context.database_pool, &fleet.system_symbol)
            .await?
            .into_iter()
            .filter(|wp| wp.is_marketplace() || wp.is_shipyard())
            .count();

    let ship_counts = (waypoint_counts as f64 * scraping_config.ship_market_ratio).floor() as u32;
    let quarter_ships = (ship_counts as f64 / 4.0).floor() as u32;

    let ships = (0..ship_counts)
        .map(|i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: 50_000,
            priority: if i < quarter_ships {
                DEFAULT_PRIORITY - 5 // first quarter ships have higher priority
            } else {
                DEFAULT_PRIORITY
            },

            disabled: false,
            range_min: 0,
            cargo_min: 0,
            survey: false,
            extractor: false,
            siphon: false,
            warp_drive: false,
        })
        .collect();

    Ok(ships)
}

async fn generate_charting_fleet_assignments(
    fleet: &database::Fleet,
    charting_config: &database::ChartingFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let uncharted_waypoints =
        database::Waypoint::get_by_system(&context.database_pool, &fleet.system_symbol)
            .await?
            .into_iter()
            .filter(|wp| !wp.is_charted())
            .count();

    let ship_count = (uncharted_waypoints as i32).min(charting_config.charting_probe_count);

    let ships = (0..ship_count)
        .map(|_i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: 100_000,
            priority: DEFAULT_PRIORITY - 10,
            disabled: false,
            range_min: -1, // need infinite range for charting
            cargo_min: 0,  // to not need cargo for charting
            survey: false,
            extractor: false,
            siphon: false,
            warp_drive: false,
        })
        .collect();

    Ok(ships)
}

async fn generate_construction_fleet_assignments(
    fleet: &database::Fleet,
    construction_config: &database::ConstructionFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    // https://en.wikipedia.org/wiki/Percolation_theory
    let min_range = 300; // todo get minimum range to get to every waypoint in the system using cruse, see percolation theory

    let min_cargo = 40; // todo get cargo according to the contracts done in the system.

    let ships = (0..construction_config.construction_ship_count)
        .map(|_i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: 500_000,
            priority: DEFAULT_PRIORITY + 10, // lower priority than other assignments
            disabled: false,
            range_min: min_range,
            cargo_min: min_cargo,
            survey: false,
            extractor: false,
            siphon: false,
            warp_drive: false,
        })
        .collect();
    Ok(ships)
}

async fn generate_contract_fleet_assignments(
    fleet: &database::Fleet,
    contract_config: &database::ContractFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    // Placeholder for contract fleet assignment generation logic

    // https://en.wikipedia.org/wiki/Percolation_theory
    let min_range = 300; // todo get minimum range to get to every waypoint in the system using cruse, see percolation theory

    let min_cargo = 40; // todo get cargo according to the contracts done in the system.

    let ships = (0..contract_config.contract_ship_count)
        .map(|_i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: 50_000,
            priority: DEFAULT_PRIORITY - 10, // higher priority than other assignments
            disabled: false,
            range_min: min_range,
            cargo_min: min_cargo,
            survey: false,
            extractor: false,
            siphon: false,
            warp_drive: false,
        })
        .collect();
    Ok(ships)
}
