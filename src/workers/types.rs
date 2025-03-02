use tokio_util::sync::CancellationToken;

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
