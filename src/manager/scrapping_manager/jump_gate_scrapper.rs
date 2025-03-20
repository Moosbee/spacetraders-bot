use std::time::Duration;

use log::{debug, info, warn};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    api,
    config::CONFIG,
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
};

use super::ScrappingManager;

pub struct JumpGateScrapper<'a> {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    scrapping_manager: &'a ScrappingManager,
}

impl<'a> JumpGateScrapper<'a> {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        scrapping_manager: &'a ScrappingManager,
    ) -> Self {
        Self {
            cancel_token,
            context,
            scrapping_manager,
        }
    }
    pub async fn run_scrapping_worker(&self) -> crate::error::Result<()> {
        info!("Starting JumpGate scrapping workers");

        if !CONFIG.market.active {
            info!("JumpGate scrapping not active, exiting");

            return Ok(());
        }

        for i in 0..CONFIG.market.max_scraps {
            if i != 0 {
                let erg = tokio::select! {
                _ = self.cancel_token.cancelled() => {
                  info!("JumpGate scrapping cancelled");
                  0},
                _ =  sleep(Duration::from_millis(CONFIG.market.scrap_interval)) => {1},
                };
                if erg == 0 {
                    break;
                }
            }

            let gates = self.get_jump_gates().await?;

            let jump_gates = self::get_all_jump_gates(&self.context.api, gates).await?;

            info!("JumpGates: {:?}", jump_gates);
            let erg = self::update_jump_gates(&self.context.database_pool, jump_gates).await;
            if erg.is_err() {
                warn!("JumpGate scrapping error: {}", erg.unwrap_err());
            }
        }

        info!("JumpGate scrapping workers done");

        Ok(())
    }

    async fn get_jump_gates(&self) -> crate::error::Result<Vec<(String, String)>> {
        let systems = self.scrapping_manager.get_system().await;
        let mut gates = vec![];
        for system in systems {
            let waypoints =
                sql::Waypoint::get_by_system(&self.context.database_pool, &system).await?;
            for waypoint in waypoints.iter().filter(|w| w.is_jump_gate()) {
                gates.push((waypoint.system_symbol.clone(), waypoint.symbol.clone()));
            }
        }
        Ok(gates)
    }
}

pub async fn get_all_jump_gates(
    api: &api::Api,
    gates: Vec<(String, String)>,
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
        update_jump_gate(&database_pool, jump_gate).await?;
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
            created_at: sqlx::types::chrono::NaiveDateTime::MIN,
            updated_at: sqlx::types::chrono::NaiveDateTime::MIN,
        })
        .collect::<Vec<sql::JumpGateConnection>>();

    sql::JumpGateConnection::insert_bulk(database_pool, &connections).await?;

    Ok(())
}
