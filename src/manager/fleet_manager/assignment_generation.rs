use std::collections::HashMap;

use database::ShipAssignment;
use space_traders_client::models::{self};
use utils::WaypointCan;

use crate::utils::ConductorContext;

const DEFAULT_PRIORITY: i32 = 100;

// TODO: get the minimum range to reach every waypoint in the system using
// cruise; see minimum spanning trees
const DEFAULT_MIN_RANGE: i32 = 300;

// TODO: get cargo according to the current trade volume and the contracts
// done in the system.
const DEFAULT_HAULER_CARGO: i32 = 40;

/// `-1` means no range requirement (infinite fuel capacity).
const INFINITE_RANGE: i32 = -1;

/// Fleets with an id below these cutoffs are still ramping up, so they get a
/// lower `credits_threshold`.
const EARLY_FLEET_MAX_ID: i32 = 10;
const CHARTING_EARLY_FLEET_MAX_ID: i32 = 20;

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
    match config {
        database::FleetConfig::Trading(trading_config) => {
            generate_trading_fleet_assignments(fleet, &trading_config, context).await
        }
        database::FleetConfig::Scraping(scraping_config) => {
            generate_scraping_fleet_assignments(fleet, &scraping_config, context).await
        }
        database::FleetConfig::Mining(mining_config) => {
            generate_mining_fleet_assignments(fleet, &mining_config, context).await
        }
        database::FleetConfig::Charting(charting_config) => {
            generate_charting_fleet_assignments(fleet, &charting_config, context).await
        }
        database::FleetConfig::Construction(construction_config) => {
            generate_construction_fleet_assignments(fleet, &construction_config)
        }
        database::FleetConfig::Contract(contract_config) => {
            generate_contract_fleet_assignments(fleet, &contract_config)
        }
        database::FleetConfig::Manuel(_) => Ok(vec![]),
    }
}

async fn generate_trading_fleet_assignments(
    fleet: &database::Fleet,
    trading_config: &database::TradingFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let market_opportunities = system_market_opportunities(fleet, context).await?;

    let ship_counts = calculate_trade_ship_counts(
        market_opportunities.strict_market_opportunities as f64,
        trading_config.ship_market_ratio,
    );

    let is_early = fleet.id < EARLY_FLEET_MAX_ID;
    let credits_threshold = if is_early { 100_000 } else { 1_000_000 };
    let trading_priority_base = DEFAULT_PRIORITY - (ship_counts as i32) * 2;

    let ships = (0..ship_counts)
        .map(|i| {
            ShipAssignment::builder(fleet.id)
                .credits_threshold(credits_threshold)
                .priority(if i == 0 {
                    // first ship has higher priority, ensure at least one ship is always active
                    trading_priority_base - 5
                } else {
                    trading_priority_base
                })
                .range_min(DEFAULT_MIN_RANGE)
                .cargo_min(trading_config.min_cargo_space)
                .build()
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

    let waypoints = system_waypoints(fleet, context).await?;

    let waypoint_count = if mining_config.ignore_engineered_asteroids {
        count_where(&waypoints, |wp| {
            wp.is_minable() && wp.waypoint_type != models::WaypointType::EngineeredAsteroid
        })
    } else {
        count_where(&waypoints, |wp| wp.is_minable())
    } as i32;

    let mining_waypoint = mining_config.mining_waypoints.min(waypoint_count);

    let siphon_waypoints_count = count_where(&waypoints, |wp| wp.is_sipherable()) as i32;

    let siphon_waypoint = mining_config.syphon_waypoints.min(siphon_waypoints_count);

    let mining_ships_count = mining_waypoint * mining_config.miners_per_waypoint;
    let survey_ships_count = mining_waypoint * mining_config.surveyers_per_waypoint;
    let siphon_ships_count = siphon_waypoint * mining_config.siphoners_per_waypoint;
    let transporter_ships_count =
        (mining_waypoint + siphon_waypoint) * mining_config.mining_transporters_per_waypoint;

    let mining_ships = copies(
        ShipAssignment::builder(fleet.id)
            .cargo_min(mining_config.min_mining_cargo_space)
            .extractor(true)
            .build(),
        mining_ships_count,
    );

    let survey_ships = copies(
        ShipAssignment::builder(fleet.id).survey(true).build(),
        survey_ships_count,
    );

    let siphon_ships = copies(
        ShipAssignment::builder(fleet.id)
            .cargo_min(mining_config.min_siphon_cargo_space)
            .siphon(true)
            .build(),
        siphon_ships_count,
    );

    let transporter_ships = copies(
        ShipAssignment::builder(fleet.id)
            .priority(DEFAULT_PRIORITY - 1) // transporters have higher priority
            .cargo_min(mining_config.min_transporter_cargo_space)
            .build(),
        transporter_ships_count,
    );

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
    let waypoints = system_waypoints(fleet, context).await?;
    let market_count = count_where(&waypoints, |wp| wp.is_marketplace());
    let shipyard_count = count_where(&waypoints, |wp| wp.is_shipyard());

    let market_opportunities = system_market_opportunities(fleet, context).await?;

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
        (1.min(market_count) + shipyard_count) as u32
    };
    let quarter_ships = (ship_counts as f64 / 4.0).floor() as u32;

    let is_early = fleet.id < EARLY_FLEET_MAX_ID;
    let credits_threshold = if is_early { 90_000 } else { 1_000_000 };
    let trading_priority_base = DEFAULT_PRIORITY - (max_trading_ship_counts as i32) * 2;

    let ships = (0..ship_counts)
        .map(|i| {
            ShipAssignment::builder(fleet.id)
                .credits_threshold(credits_threshold)
                .priority(if i < quarter_ships {
                    // first quarter ships have higher priority
                    trading_priority_base - 5
                } else {
                    trading_priority_base
                })
                .build()
        })
        .collect();

    Ok(ships)
}

async fn generate_charting_fleet_assignments(
    fleet: &database::Fleet,
    charting_config: &database::ChartingFleetConfig,
    context: &ConductorContext,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let waypoints = system_waypoints(fleet, context).await?;
    let uncharted_waypoints = count_where(&waypoints, |wp| !wp.is_charted());

    let ship_count = (uncharted_waypoints as i32).min(charting_config.charting_probe_count);

    let priority = if charting_config.chart_only_jump_gates {
        DEFAULT_PRIORITY - 30
    } else {
        DEFAULT_PRIORITY - 20
    };

    let is_early = fleet.id < CHARTING_EARLY_FLEET_MAX_ID;
    let credits_threshold = if is_early { 100_000 } else { 500_000 };

    let ships = copies(
        ShipAssignment::builder(fleet.id)
            .credits_threshold(credits_threshold)
            .priority(priority)
            .range_min(INFINITE_RANGE)
            .build(),
        ship_count,
    )
    .collect();

    Ok(ships)
}

fn generate_construction_fleet_assignments(
    fleet: &database::Fleet,
    construction_config: &database::ConstructionFleetConfig,
) -> crate::error::Result<Vec<ShipAssignment>> {
    let ships = copies(
        ShipAssignment::builder(fleet.id)
            .credits_threshold(500_000)
            .priority(DEFAULT_PRIORITY + 10) // lower priority than other assignments
            .range_min(DEFAULT_MIN_RANGE)
            .cargo_min(DEFAULT_HAULER_CARGO)
            .build(),
        construction_config.construction_ship_count,
    )
    .collect();
    Ok(ships)
}

fn generate_contract_fleet_assignments(
    fleet: &database::Fleet,
    contract_config: &database::ContractFleetConfig,
) -> crate::error::Result<Vec<ShipAssignment>> {
    // Placeholder for contract fleet assignment generation logic

    let ships = copies(
        ShipAssignment::builder(fleet.id)
            .credits_threshold(50_000)
            .priority(DEFAULT_PRIORITY - 15) // higher priority than other assignments
            .range_min(DEFAULT_MIN_RANGE)
            .cargo_min(DEFAULT_HAULER_CARGO)
            .build(),
        contract_config.contract_ship_count,
    )
    .collect();
    Ok(ships)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MarketOpportunities {
    pub total_market_opportunities: i32,
    pub total_without_fuel_market_opportunities: i32,
    pub strict_market_opportunities: i32, // only opportunities from export to import
}

#[derive(Debug, Clone, Copy, Default)]
struct SymbolCounts {
    total: i32,
    exports: i32,
    imports: i32,
}

impl SymbolCounts {
    /// Unordered pairs of markets trading this symbol: `C(total, 2)`.
    fn pairs(self) -> i32 {
        self.total * (self.total - 1) / 2
    }

    /// Export/import pairs, i.e. a strict buy-low/sell-high opportunity.
    fn strict_pairs(self) -> i32 {
        self.exports * self.imports
    }
}

pub(crate) fn calculate_market_opportunities(
    market_trades: &[database::MarketTrade],
) -> MarketOpportunities {
    let symbol_counts: HashMap<models::TradeSymbol, SymbolCounts> =
        market_trades.iter().fold(HashMap::new(), |mut map, tg| {
            let entry = map.entry(tg.symbol).or_default();
            entry.total += 1;
            match tg.r#type {
                models::market_trade_good::Type::Export => entry.exports += 1,
                models::market_trade_good::Type::Import => entry.imports += 1,
                models::market_trade_good::Type::Exchange => {}
            }
            map
        });

    symbol_counts
        .iter()
        .fold(MarketOpportunities::default(), |mut acc, (tg, counts)| {
            acc.strict_market_opportunities += counts.strict_pairs();
            acc.total_market_opportunities += counts.pairs();
            if *tg != models::TradeSymbol::Fuel {
                acc.total_without_fuel_market_opportunities += counts.pairs();
            }
            acc
        })
}

pub(crate) fn calculate_trade_ship_counts(
    strict_market_opportunities: f64,
    linear_factor: f64, /* default 0.01 */
) -> u32 {
    let logarithmic_term = (strict_market_opportunities + 22.0).log2() - 5.0;
    let linear_term = linear_factor * strict_market_opportunities;
    (logarithmic_term + linear_term).ceil().max(0.0) as u32
}

/// All waypoints in the fleet's system.
async fn system_waypoints(
    fleet: &database::Fleet,
    context: &ConductorContext,
) -> crate::error::Result<Vec<database::Waypoint>> {
    let page = database::Waypoint::get_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?;
    Ok(page.items)
}

/// Market opportunities derived from the latest market trades in the fleet's
/// system.
async fn system_market_opportunities(
    fleet: &database::Fleet,
    context: &ConductorContext,
) -> crate::error::Result<MarketOpportunities> {
    let market_trades = database::MarketTrade::get_last_by_system(
        &context.database_pool,
        &fleet.system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items;
    Ok(calculate_market_opportunities(&market_trades))
}

/// Number of `items` matching `predicate`.
fn count_where<T>(items: &[T], predicate: impl Fn(&T) -> bool) -> usize {
    items.iter().filter(|item| predicate(item)).count()
}

/// `count` copies of `template`; negative counts yield nothing.
fn copies(template: ShipAssignment, count: i32) -> impl Iterator<Item = ShipAssignment> {
    let count = usize::try_from(count).unwrap_or(0);
    std::iter::repeat_n(template, count)
}
