use std::{collections::HashMap, fmt::Debug, sync::Arc};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

use crate::{ship::ShipManager, sql::DbPool};

#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub api: crate::api::Api,
    pub database_pool: DbPool,
    pub ship_manager: Arc<ShipManager>,
    pub all_waypoints:
        Arc<DashMap<String, HashMap<String, space_traders_client::models::Waypoint>>>,
}

pub trait Conductor: Send + Sync {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
    fn get_name(&self) -> String;
    fn get_cancel_token(&self) -> CancellationToken;
    fn is_independent(&self) -> bool {
        true
    }
}
