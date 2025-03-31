use std::time::Duration;

use chrono::DateTime;
use log::{debug, warn};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    api,
    sql::{self, DatabaseConnector},
};

pub async fn get_all_jump_gates(
    api: &api::Api,
    gates: Vec<(String, String, bool)>,
) -> crate::error::Result<Vec<models::JumpGate>> {
    let mut jump_gate_handles = tokio::task::JoinSet::new();

    for waypoint in gates {
        let api = api.clone();
        let waypoint = waypoint.clone();
        jump_gate_handles.spawn(async move {
            debug!("JumpGate: {}", waypoint.1);
            let mut err_count = 2;
            loop {
                let jump_gate = api.get_jump_gate(&waypoint.0, &waypoint.1).await;
                match jump_gate {
                    Ok(jump_gate) => {
                        break Some(*jump_gate.data);
                    }
                    Err(space_traders_client::apis::Error::ResponseError(e)) => {
                        if
                            e.entity
                                .as_ref()
                                .map(
                                    |f|
                                        f.error.code ==
                                        space_traders_client::models::error_codes::WAYPOINT_NO_ACCESS_ERROR
                                )
                                .unwrap_or(false)
                        {
                            break None;
                        }
                        err_count -= 1;
                        warn!("JumpGate: {} Error: {:?}", waypoint.1, e);
                        if err_count <= 0 {
                            break None;
                        }
                        sleep(Duration::from_millis(1000)).await;
                    }
                    Err(e) => {
                        err_count -= 1;
                        warn!("JumpGate: {} Error: {} {:?}", waypoint.1, e, e);
                        if err_count <= 0 {
                            break None;
                        }
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
        });
    }

    let mut jump_gates = Vec::new();
    while let Some(jump_gate_data) = jump_gate_handles.join_next().await {
        debug!(
            "JumpGate: {} {}",
            jump_gate_data.is_ok(),
            jump_gate_data
                .as_ref()
                .map(|m| m
                    .as_ref()
                    .map(|j| j.symbol.clone())
                    .unwrap_or("Errorr".to_string()))
                .unwrap_or("Error".to_string())
        );
        match jump_gate_data {
            Ok(jump_gate) => {
                if let Some(jump_gate) = jump_gate {
                    jump_gates.push(jump_gate);
                }
            }
            Err(e) => {
                warn!("JumpGate: Error: {}, {:?}", e, e);
            }
        }
    }

    Ok(jump_gates)
}

pub async fn update_jump_gates(
    database_pool: &crate::sql::DbPool,
    jump_gates: Vec<models::JumpGate>,
) -> Result<(), crate::error::Error> {
    for jump_gate in jump_gates {
        update_jump_gate(database_pool, jump_gate).await?;
    }

    Ok(())
}

pub async fn update_jump_gate(
    database_pool: &crate::sql::DbPool,
    jump_gate: models::JumpGate,
) -> Result<(), crate::error::Error> {
    let connections = jump_gate
        .connections
        .iter()
        .map(|c| sql::JumpGateConnection {
            id: 0,
            from: jump_gate.symbol.clone(),
            to: c.clone(),
            created_at: DateTime::<chrono::Utc>::MIN_UTC,
            updated_at: DateTime::<chrono::Utc>::MIN_UTC,
        })
        .collect::<Vec<sql::JumpGateConnection>>();

    sql::JumpGateConnection::insert_bulk(database_pool, &connections).await?;

    Ok(())
}
