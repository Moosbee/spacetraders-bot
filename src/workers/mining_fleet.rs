use std::time::Duration;

use log::info;
use tokio::time::sleep;
pub async fn mining_conductor() {
    info!("Starting mining workers");
    sleep(Duration::from_secs(10)).await;

    info!("Mining workers done");
}
