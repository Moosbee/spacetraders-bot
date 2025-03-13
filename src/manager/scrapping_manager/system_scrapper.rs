use log::debug;

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
        &all_systems.iter().map(sql::System::from).collect(),
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
    sql::Waypoint::insert_bulk(
        database_pool,
        &waypoints
            .iter()
            .map(sql::Waypoint::from)
            .collect::<Vec<_>>(),
    )
    .await?;

    Ok(())
}
