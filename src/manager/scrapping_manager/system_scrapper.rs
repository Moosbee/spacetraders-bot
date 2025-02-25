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
            .map(|s| -> sql::System { Into::into(s.clone()) })
            .collect(),
    )
    .await?;
    for system in &all_systems {
        let waypoints = api.get_all_waypoints(&system.symbol, 20).await?;
        sql::Waypoint::insert_bulk(
            &database_pool,
            &waypoints
                .iter()
                .map(sql::Waypoint::from)
                .collect::<Vec<_>>(),
        )
        .await?;
    }

    Ok(())
}
