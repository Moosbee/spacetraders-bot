use std::time::Duration;

use log::info;
use tokio::time::sleep;
pub async fn contract_conductor() {
    info!("Starting contract workers");
    sleep(Duration::from_secs(10)).await;

    info!("Contract workers done");
}
