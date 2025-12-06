use core::panic;
use std::collections::HashSet;

use database::DatabaseConnector;
use space_traders_client::models;
use utils::WaypointCan;

struct SystemFleets {
    charting_fleet: Option<database::Fleet>, // exists if there are >1 uncharted waypoints
    scrapping_fleet: Option<database::Fleet>, // exists if there are >1 marketplaces
    trading_fleet: Option<database::Fleet>, // exists if there are >1 marketplaces, and there are no uncharted marketplace waypoints, if it's a system with an construction site there are more ships
    mining_fleet: Option<database::Fleet>,  // exists if a construction fleet exists
    construction_fleet: Option<database::Fleet>, // exists if it's the main system and in that system there is an unfinished construction site
    contract_fleet: Option<database::Fleet>, // exists if it's the main system and there is no other fleet present anywhere
}

/// adds or updates fleets in a system
pub async fn populate_system(
    context: &crate::utils::ConductorContext,
    system_symbol: &str,
) -> Result<(), crate::error::Error> {
    // let fleet_manager = &context.managers.fleet_manager;

    // fleet_manager
    //     .regenerate_system_assignments(system_symbol.to_string())
    //     .await?;

    let system_fleets = generate_system_fleets(context, system_symbol).await?;
    let current_fleets =
        database::Fleet::get_by_system(&context.database_pool, system_symbol).await?; // we can assume that there is a maximum of one fleet per type per system

    // Match scrapping fleet
    update_fleet(
        context,
        system_fleets.scrapping_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Scrapping)
            .cloned(),
    )
    .await?;

    // Match charting fleet
    update_fleet(
        context,
        system_fleets.charting_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Charting)
            .cloned(),
    )
    .await?;

    // Match mining fleet
    update_fleet(
        context,
        system_fleets.mining_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Mining)
            .cloned(),
    )
    .await?;

    // Match trading fleet
    update_fleet(
        context,
        system_fleets.trading_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Trading)
            .cloned(),
    )
    .await?;

    // Match construction fleet
    update_fleet(
        context,
        system_fleets.construction_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Construction)
            .cloned(),
    )
    .await?;

    // Match contract fleet
    update_fleet(
        context,
        system_fleets.contract_fleet,
        current_fleets
            .iter()
            .find(|f| f.fleet_type == database::FleetType::Contract)
            .cloned(),
    )
    .await?;

    let fleets = database::Fleet::get_by_system(&context.database_pool, system_symbol).await?;

    for fleet in fleets {
        let current_assignments =
            database::ShipAssignment::get_by_fleet_id(&context.database_pool, fleet.id).await?;
        let new_assignments =
            super::assignment_management::generate_fleet_assignments(&fleet, context).await?;

        let assignments = super::assignment_management::fix_fleet_assignments(
            current_assignments,
            new_assignments,
        )
        .await?;

        super::assignment_management::update_fleet_assignments(context, assignments).await?;
    }

    Ok(())
}

async fn generate_system_fleets(
    context: &crate::utils::ConductorContext,
    system_symbol: &str,
) -> Result<SystemFleets, crate::error::Error> {
    let waypoints =
        database::Waypoint::get_by_system(&context.database_pool, system_symbol).await?;

    let headquarters_waypoint = { context.run_info.read().await.headquarters.clone() };

    let is_headquarters_system = waypoints.iter().any(|f| f.symbol == headquarters_waypoint);

    let has_uncharted_waypoints = waypoints.iter().any(|w| !w.is_charted());

    let has_uncharted_marketplace_waypoints = waypoints
        .iter()
        .any(|w| w.is_marketplace() && !w.is_charted());

    let constructions =
        database::ConstructionMaterial::get_by_system(&context.database_pool, system_symbol)
            .await?;

    let open_construction_site = constructions
        .iter()
        .filter(|c| c.required != c.fulfilled)
        .collect::<Vec<_>>();

    let open_construction_waypoints = open_construction_site
        .iter()
        .map(|c| c.waypoint_symbol.clone())
        .collect::<HashSet<_>>();

    if open_construction_waypoints.len() > 1 {
        log::error!(
            "System {} has multiple open construction sites: {:?}",
            system_symbol,
            open_construction_waypoints
        );
        panic!("System has multiple open construction sites");
    }

    let construction_waypoint = open_construction_waypoints.into_iter().next();

    let has_open_construction_site = !open_construction_site.is_empty();

    let market_count = waypoints.iter().filter(|w| w.is_marketplace()).count();

    let _shipyard_count = waypoints.iter().filter(|w| w.is_shipyard()).count();

    let mut system_fleets = SystemFleets {
        charting_fleet: None,
        scrapping_fleet: None,
        trading_fleet: None,
        mining_fleet: None,
        construction_fleet: None,
        contract_fleet: None,
    };

    // charting fleet
    if has_uncharted_waypoints {
        system_fleets.charting_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Charting(database::ChartingFleetConfig {
                    charting_probe_count: 1,
                }),
            ),
        );
    }

    // construction fleet
    if is_headquarters_system && has_open_construction_site && construction_waypoint.is_some() {
        system_fleets.construction_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Construction(database::ConstructionFleetConfig {
                    construction_ship_count: 1,
                    construction_waypoint: construction_waypoint.unwrap(),
                }),
            ),
        );
    }

    // scrapping fleet
    if market_count > 1 {
        system_fleets.scrapping_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Scraping(database::ScrapingFleetConfig {
                    allowed_requests: 1, // todo do right calculations
                    notify_on_shipyard: true,
                    ship_market_ratio: 1.0,
                }),
            ),
        );
    }

    // trading fleet
    if market_count > 1 && !has_uncharted_marketplace_waypoints {
        let ship_market_ratio = if system_fleets.construction_fleet.is_some() {
            0.2
        } else {
            0.1
        };

        let trade_mode = if system_fleets.construction_fleet.is_some() {
            database::TradeMode::ProfitPerHour
        } else {
            database::TradeMode::ProfitPerTrip
        };

        let trade_profit_threshold = if system_fleets.construction_fleet.is_some() {
            200
        } else {
            2000
        };

        let market_blacklist = if system_fleets.construction_fleet.is_some() {
            let goods = open_construction_site
                .iter()
                .map(|f| f.trade_symbol)
                .collect::<HashSet<_>>();
            goods.into_iter().collect()
        } else {
            vec![]
        };

        system_fleets.trading_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Trading(database::TradingFleetConfig {
                    market_blacklist,
                    market_prefer_list: vec![], // todo calculate based on construction needs
                    purchase_multiplier: 2.0,
                    ship_market_ratio,
                    min_cargo_space: 40, // todo calculate based on markets
                    trade_mode,
                    trade_profit_threshold,
                }),
            ),
        );
    }

    // mining fleet
    if system_fleets.construction_fleet.is_some() {
        let mining_prefer_list = vec![
            // todo calculate based on construction needs
            models::TradeSymbol::SiliconCrystals,
            models::TradeSymbol::CopperOre,
            models::TradeSymbol::IronOre,
            models::TradeSymbol::QuartzSand,
        ];

        let mining_eject_list = models::extraction_yield::EXTRACTABLE
            .iter()
            .filter(|s| !mining_prefer_list.contains(s))
            .cloned()
            .collect::<Vec<_>>();

        system_fleets.mining_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Mining(database::MiningFleetConfig {
                    mining_eject_list,
                    mining_prefer_list,
                    ignore_engineered_asteroids: false,
                    stop_all_unstable: true,
                    unstable_since_timeout: 10800,
                    mining_waypoints: 1,
                    syphon_waypoints: 1,
                    miners_per_waypoint: 16,
                    siphoners_per_waypoint: 6,
                    surveyers_per_waypoint: 1,
                    mining_transporters_per_waypoint: 3,
                    min_transporter_cargo_space: 80,
                    min_mining_cargo_space: 1,
                    min_siphon_cargo_space: 1,
                }),
            ),
        );
    }

    // contract fleet
    if is_headquarters_system {
        system_fleets.contract_fleet = Some(
            database::Fleet::new(system_symbol.to_string(), true).with_config(
                database::FleetConfig::Contract(database::ContractFleetConfig {
                    contract_ship_count: 1,
                }),
            ),
        );
    }

    Ok(system_fleets)
}

async fn update_fleet(
    context: &crate::utils::ConductorContext,
    // if both are Some, update old with new, if only new is Some, insert new, if only old is Some, deactivate old
    new_fleet: Option<database::Fleet>,
    old_fleet: Option<database::Fleet>,
) -> Result<(), crate::error::Error> {
    match (new_fleet, old_fleet) {
        (Some(mut new), Some(old)) => {
            // update old with new
            new.id = old.id;
            database::Fleet::insert(&context.database_pool, &new).await?;
        }
        (Some(new), None) => {
            // insert new
            database::Fleet::insert_new(&context.database_pool, &new).await?;
        }
        (None, Some(mut old)) => {
            // deactivate old
            old.active = false;
            database::Fleet::insert(&context.database_pool, &old).await?;
        }
        (None, None) => {
            // do nothing
        }
    }

    Ok(())
}

pub async fn is_system_populated(
    database_pool: &database::DbPool,
    system_symbol: &str,
) -> Result<bool, crate::error::Error> {
    let fleets = database::Fleet::get_by_system(database_pool, system_symbol).await?;
    let length = 1;
    Ok(fleets.len() >= length) // if there are less than 5 fleets, the system is not fully populated
}
