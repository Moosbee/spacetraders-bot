use std::collections::HashMap;

use database::ShipAssignment;
use space_traders_client::models::{self};
use utils::WaypointCan;

use crate::utils::ConductorContext;

const DEFAULT_PRIORITY: i32 = 100;

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
    let market_trades = database::MarketTrade::get_last_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items;

    let market_opportunities = calculate_market_opportunities(&market_trades);

    let ship_counts = calculate_trade_ship_counts(
        market_opportunities.strict_market_opportunities as f64,
        trading_config.ship_market_ratio,
    );

    let min_range = 300; // todo get minimum range to get to every waypoint in the system using cruse, see percolation theory and minimum spanning tree and Kruskal's algorithm

    let min_cargo = trading_config.min_cargo_space; // todo get cargo according to the current trade volume

    let is_early = fleet.id < 10;

    let ships = (0..ship_counts)
        .map(|i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: if is_early { 100_000 } else { 1_000_000 },
            priority: if i == 0 {
                DEFAULT_PRIORITY - 5 - (ship_counts as i32) * 2 // first ship has higher priority, ensure at least one ship is always active
            } else {
                DEFAULT_PRIORITY - (ship_counts as i32) * 2
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

    let waypoints = database::Waypoint::get_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items;

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
    let (market_count, shipyard_count) = database::Waypoint::get_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items
    .into_iter()
    .fold(
        (0u32, 0u32),
        |(mut market_count, mut shipyard_count), wp| {
            if wp.is_marketplace() {
                market_count += 1;
            }
            if wp.is_shipyard() {
                shipyard_count += 1;
            }
            (market_count, shipyard_count)
        },
    );

    let market_trades = database::MarketTrade::get_last_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items;

    let market_opportunities = calculate_market_opportunities(&market_trades);

    let trading_ship_counts = calculate_trade_ship_counts(
        market_opportunities.strict_market_opportunities as f64,
        0.01,
    );

    let max_trading_ship_counts = calculate_trade_ship_counts(
        market_opportunities.strict_market_opportunities as f64,
        0.05,
    );

    let ship_counts = if trading_ship_counts > 0 {
        (market_count as f64 * scraping_config.ship_market_ratio).floor() as u32
    } else {
        1.min(market_count) + shipyard_count
    };
    let quarter_ships = (ship_counts as f64 / 4.0).floor() as u32;

    let is_early = fleet.id < 10;

    let ships = (0..ship_counts)
        .map(|i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: if is_early { 50_000 } else { 1_000_000 },
            priority: if i < quarter_ships {
                DEFAULT_PRIORITY - 5 - (max_trading_ship_counts as i32) * 2 // first quarter ships have higher priority
            } else {
                DEFAULT_PRIORITY - (max_trading_ship_counts as i32) * 2
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
    let uncharted_waypoints = database::Waypoint::get_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items
    .into_iter()
    .filter(|wp| !wp.is_charted())
    .count();

    let ship_count = (uncharted_waypoints as i32).min(charting_config.charting_probe_count);

    let priority = if charting_config.chart_only_jump_gates {
        DEFAULT_PRIORITY - 30
    } else {
        DEFAULT_PRIORITY - 20
    };

    let is_early = fleet.id < 20;

    let ships = (0..ship_count)
        .map(|_i| ShipAssignment {
            id: 0, // This should be set appropriately
            fleet_id: fleet.id,
            max_purchase_price: 1_000_000,
            credits_threshold: if is_early { 100_000 } else { 500_000 },
            priority,
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
    _context: &ConductorContext,
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
    _context: &ConductorContext,
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

#[derive(Debug, Clone, Default)]
pub(crate) struct MarketOpportunities {
    pub total_market_opportunities: i32,
    pub total_without_fuel_market_opportunities: i32,
    pub strict_market_opportunities: i32, // only opportunities from export to import
}

pub(crate) fn calculate_market_opportunities(
    market_trades: &[database::MarketTrade],
) -> MarketOpportunities {
    let market_trade_symbols: HashMap<models::TradeSymbol, (i32, i32, i32, i32)> =
        market_trades.iter().fold(HashMap::new(), |mut map, tg| {
            let entry = map.entry(tg.symbol).or_insert((0, 0, 0, 0));
            entry.0 += 1;
            match tg.r#type {
                models::market_trade_good::Type::Export => entry.1 += 1,
                models::market_trade_good::Type::Exchange => entry.2 += 1,
                models::market_trade_good::Type::Import => entry.3 += 1,
            }
            map
        });

    let market_opportunities =
        market_trade_symbols
            .iter()
            .fold(MarketOpportunities::default(), |mut acc, (tg, count)| {
                acc.strict_market_opportunities += count.1 * count.2;
                acc.total_market_opportunities += ((count.0) * ((count.0) - 1)) / 2;
                if *tg != models::TradeSymbol::Fuel {
                    acc.total_without_fuel_market_opportunities +=
                        ((count.0) * ((count.0) - 1)) / 2;
                }
                acc
            });

    market_opportunities
}

pub(crate) fn calculate_trade_ship_counts(
    strict_market_opportunities: f64,
    linear_factor: f64, /* default 0.01 */
) -> u32 {
    (((strict_market_opportunities + 22.0).log2() - 5.0
        + (linear_factor * strict_market_opportunities))
        .ceil())
    .max(0.0) as u32
}
