use std::time::Duration;

use log::info;
use tokio::time::sleep;

pub async fn construction_conductor() {
    info!("Starting construction workers");
    sleep(Duration::from_secs(1)).await;

    info!("Construction workers done");
}
