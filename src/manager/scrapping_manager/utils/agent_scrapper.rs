use database::DatabaseConnector;

pub async fn update_all_agents(
    api: &space_traders_client::Api,
    database_pool: &database::DbPool,
) -> crate::error::Result<()> {
    let agents = api.get_all_agents(20).await?;
    let all_agents = agents
        .into_iter()
        .map(database::Agent::from)
        .collect::<Vec<_>>();

    database::Agent::insert_bulk(database_pool, &all_agents).await?;

    Ok(())
}
