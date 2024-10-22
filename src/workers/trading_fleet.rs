use std::time::Duration;

use log::info;
use tokio::time::sleep;
pub async fn trading_conductor() {
    info!("Starting trading workers");
    sleep(Duration::from_secs(10)).await;

    info!("Trading workers done");
}
