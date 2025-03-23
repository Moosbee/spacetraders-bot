use std::collections::HashMap;

use log::debug;
use space_traders_client::models;

use crate::{
    api::Api,
    sql::{self, DatabaseConnector},
};

pub async fn update_all_systems(
    database_pool: &sql::DbPool,
    api: &Api,
) -> crate::error::Result<()> {
    let all_systems = api.get_all_systems(20).await?;
    sql::System::insert_bulk(
        database_pool,
        &all_systems
            .iter()
            .map(sql::System::from)
            .collect::<Vec<_>>(),
    )
    .await?;

    for system in &all_systems {
        update_system(database_pool, api, &system.symbol, false).await?;
    }

    Ok(())
}

pub async fn update_system(
    database_pool: &sql::DbPool,
    api: &Api,
    system_symbol: &str,
    also_system: bool,
) -> crate::error::Result<()> {
    if also_system {
        let system = api.get_system(system_symbol).await?;
        sql::System::insert(database_pool, &sql::System::from(&*system.data)).await?;
    }

    let waypoints = loop {
        debug!("Getting waypoints for system {}", system_symbol);
        let waypoints = api.get_all_waypoints(system_symbol, 20).await;
        match waypoints {
            Ok(waypoints) => break waypoints,
            Err(e) => {
                log::error!("Error getting waypoints: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
    };

    let mut sql_waypoints = waypoints
        .iter()
        .map(sql::Waypoint::from)
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

    sql::Waypoint::insert_bulk(
        database_pool,
        &sql_waypoints
            .into_iter()
            .map(|(_, w)| w)
            .collect::<Vec<_>>(),
    )
    .await?;

    Ok(())
}
