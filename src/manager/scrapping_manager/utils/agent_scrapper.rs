use crate::sql::{self, DatabaseConnector, DbPool};

pub async fn update_all_agents(
    api: &crate::api::Api,
    database_pool: &DbPool,
) -> crate::error::Result<()> {
    let agents = api.get_all_agents(20).await?;
    let all_agents = agents.into_iter().map(sql::Agent::from).collect::<Vec<_>>();

    sql::Agent::insert_bulk(database_pool, &all_agents).await?;

    Ok(())
}
