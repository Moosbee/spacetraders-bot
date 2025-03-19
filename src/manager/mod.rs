use crate::error::Result;
use tokio_util::sync::CancellationToken;

pub mod chart_manager;
pub mod construction_manager;
pub mod contract_manager;
pub mod mining_manager;
pub mod scrapping_manager;
pub mod ship_task;
pub mod trade_manager;

pub trait Manager: Send + Sync {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;
    fn get_name(&self) -> &str;
    fn get_cancel_token(&self) -> &CancellationToken;
}
