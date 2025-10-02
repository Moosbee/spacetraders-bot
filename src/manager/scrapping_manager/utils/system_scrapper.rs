use std::collections::HashMap;

use database::DatabaseConnector;
use space_traders_client::{models, Api};
use tracing::debug;

pub async fn update_all_systems(
    database_pool: &database::DbPool,
    api: &Api,
) -> crate::error::Result<()> {
    let all_systems = api.get_all_systems(20).await?;
    database::System::insert_bulk(
        database_pool,
        &all_systems
            .iter()
            .map(database::System::from)
            .collect::<Vec<_>>(),
    )
    .await?;

    debug!("Updating {} systems", all_systems.len());

    for system in &all_systems {
        let erg = update_system(database_pool, api, &system.symbol, false).await;
        if let Err(e) = erg {
            tracing::error!("Error updating system {}: {}", system.symbol, e);
        }
    }

    Ok(())
}

pub async fn update_system(
    database_pool: &database::DbPool,
    api: &Api,
    system_symbol: &str,
    also_system: bool,
) -> crate::error::Result<()> {
    if also_system {
        let system = api.get_system(system_symbol).await?;
        database::System::insert(database_pool, &database::System::from(&*system.data)).await?;
    }

    let waypoints = loop {
        debug!("Getting waypoints for system {}", system_symbol);
        let waypoints = api.get_all_waypoints(system_symbol, 20).await;
        match waypoints {
            Ok(waypoints) => break waypoints,
            Err(e) => {
                tracing::error!("Error getting waypoints: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
    };

    let mut sql_waypoints = waypoints
        .iter()
        .map(database::Waypoint::from)
        .map(|w| (w.symbol.clone(), w))
        .collect::<HashMap<_, _>>();

    if waypoints.iter().any(|w| {
        w.traits
            .iter()
            .any(|t| t.symbol == models::WaypointTraitSymbol::Uncharted)
    }) {
        debug!("System {} has uncharted waypoints", system_symbol);
        let shipyards = api
            .get_all_waypoints_with_traits(
                system_symbol,
                Some(
                    models::GetSystemWaypointsTraitsParameter::WaypointTraitSymbol(
                        models::WaypointTraitSymbol::Shipyard,
                    ),
                ),
                20,
            )
            .await?;

        for sh in shipyards.into_iter() {
            let wp = sql_waypoints.get_mut(&sh.symbol).unwrap();
            wp.has_shipyard = true;
        }

        let markets = api
            .get_all_waypoints_with_traits(
                system_symbol,
                Some(
                    models::GetSystemWaypointsTraitsParameter::WaypointTraitSymbol(
                        models::WaypointTraitSymbol::Marketplace,
                    ),
                ),
                20,
            )
            .await?;

        for mk in markets.into_iter() {
            let wp = sql_waypoints.get_mut(&mk.symbol).unwrap();
            wp.has_marketplace = true;
        }
    }

    database::Waypoint::insert_bulk(
        database_pool,
        &sql_waypoints.into_values().collect::<Vec<_>>(),
    )
    .await?;

    Ok(())
}
