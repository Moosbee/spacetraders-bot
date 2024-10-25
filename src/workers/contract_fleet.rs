use std::{clone, collections::VecDeque, sync::Arc, time::Duration};

use anyhow::Error;
use log::info;
use space_traders_client::models::{self, Contract};
use tokio::time::sleep;

use crate::{api, my_ship};
pub async fn contract_conductor(
    api: api::Api,
    database_pool: sqlx::PgPool,
    ship_roles: std::collections::HashMap<String, my_ship::Role>,
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    waypoints: Vec<models::Waypoint>,
) -> Result<(), Error> {
    info!("Starting contract workers");
    sleep(Duration::from_millis(500)).await;

    let contract_ships = ship_roles
        .iter()
        .filter(|ship| ship.1 == &my_ship::Role::Contract)
        .map(|ship| ship.0)
        .collect::<Vec<_>>();

    if contract_ships.len() == 0 {
        return Err(Error::msg("No ships assigned to me"));
    }

    let contracts = api.get_all_contracts(20).await?;

    let contract_que = VecDeque::from(
        contracts
            .iter()
            .filter(|f| !f.fulfilled)
            .collect::<Vec<Contract>>(),
    );

    while let Some(contract) = contract_que.pop_front() {}

    info!("Contract workers done");
    Ok(())
}

// fn get_next_contract(contracts: Vec<Contract>) -> Option<Contract> {
//     let mut unfinished_contracts = contracts
//         .iter()
//         .filter(|cont| !cont.fulfilled && cont.accepted)
//         .map(|f| f.clone())
//         .collect::<Vec<_>>();

//     let mut unaccepted_contracts = contracts
//         .iter()
//         .filter(|cont| !cont.fulfilled && !cont.accepted)
//         .map(|f| f.clone())
//         .collect::<Vec<_>>();

//     unfinished_contracts.append(&mut unaccepted_contracts);

//     unfinished_contracts.first().cloned()
// }
