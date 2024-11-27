use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub api: crate::api::Api,
    pub database_pool: sqlx::PgPool,
    pub ship_roles: HashMap<String, crate::ship::Role>,
    pub my_ships: Arc<DashMap<String, crate::ship::MyShip>>,
    pub all_waypoints:
        Arc<DashMap<String, HashMap<String, space_traders_client::models::Waypoint>>>,
}

pub trait Conductor: Send + Sync {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
    fn get_name(&self) -> String;
    fn get_cancel_token(&self) -> CancellationToken;
}
