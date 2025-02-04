use std::time::Duration;

use log::info;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::{self, DatabaseConnector},
};

pub struct AgentScrapper {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
}

impl AgentScrapper {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
    ) -> Self {
        Self {
            cancel_token,
            context,
        }
    }

    pub async fn run_scrapping_worker(&self) -> crate::error::Result<()> {
        info!("Starting agent scrapping workers");

        if !CONFIG.market.active {
            info!("Agent scrapping not active, exiting");

            return Ok(());
        }

        for i in 0..CONFIG.market.max_agent_scraps {
            if i != 0 {
                let erg = tokio::select! {
                _ = self.cancel_token.cancelled() => {
                  info!("Agent scrapping cancelled");
                  0},
                _ =  sleep(Duration::from_millis(CONFIG.market.agent_interval)) => {1}
                };
                if erg == 0 {
                    break;
                }
            }

            self.update_all_agents().await?;
        }

        info!("Agent scrapping workers done");

        Ok(())
    }

    async fn update_all_agents(&self) -> crate::error::Result<()> {
        let agents = self.context.api.get_all_agents(20).await?;
        let all_agents = agents.into_iter().map(sql::Agent::from).collect::<Vec<_>>();

        for agent in &all_agents {
            sql::Agent::insert(&self.context.database_pool, &agent).await?;
        }

        Ok(())
    }
}
