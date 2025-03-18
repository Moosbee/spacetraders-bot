use std::time::Duration;

use log::{debug, info, warn};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
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

            let jump_gates = self.get_all_jump_gates().await?;

            info!("JumpGates: {:?}", jump_gates);
            let erg = self.update_jump_gates(jump_gates).await;
            if erg.is_err() {
                warn!("JumpGate scrapping error: {}", erg.unwrap_err());
            }
        }

        info!("JumpGate scrapping workers done");

        Ok(())
    }

    async fn get_all_jump_gates(&self) -> crate::error::Result<Vec<models::JumpGate>> {
        let systems = self.scrapping_manager.get_system().await;
        let mut jump_gate_handles = tokio::task::JoinSet::new();

        for system in systems {
            let waypoints =
                sql::Waypoint::get_by_system(&self.context.database_pool, &system).await?;
            for waypoint in waypoints.iter().filter(|w| w.is_jump_gate()) {
                let api = self.context.api.clone();
                let waypoint = waypoint.clone();
                jump_gate_handles.spawn(async move {
                    debug!("JumpGate: {}", waypoint.symbol);
                    loop {
                        let jump_gate = api
                            .get_jump_gate(&waypoint.system_symbol, &waypoint.symbol)
                            .await;
                        match jump_gate {
                            Ok(jump_gate) => {
                                break *jump_gate.data;
                            }
                            Err(e) => {
                                warn!("JumpGate: {} Error: {}", waypoint.symbol, e);
                                sleep(Duration::from_millis(500)).await;
                            }
                        }
                    }
                });
            }
        }

        let mut jump_gates = Vec::new();
        while let Some(jump_gate_data) = jump_gate_handles.join_next().await {
            debug!(
                "JumpGate: {} {}",
                jump_gate_data.is_ok(),
                jump_gate_data
                    .as_ref()
                    .map(|m| m.symbol.clone())
                    .unwrap_or("Error".to_string())
            );
            match jump_gate_data {
                Ok(jump_gate) => {
                    jump_gates.push(jump_gate);
                }
                Err(e) => {
                    warn!("JumpGate: Error: {}", e);
                }
            }
        }

        Ok(jump_gates)
    }

    async fn update_jump_gates(
        &self,
        jump_gates: Vec<models::JumpGate>,
    ) -> Result<(), crate::error::Error> {
        for jump_gate in jump_gates {
            update_jump_gate(&self.context.database_pool, jump_gate).await?;
        }

        Ok(())
    }
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
