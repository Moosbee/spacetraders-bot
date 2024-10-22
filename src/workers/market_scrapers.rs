use std::time::Duration;

use log::info;
use tokio::time::sleep;
pub async fn scrapping_conductor() {
    info!("Starting market scrapping workers");
    sleep(Duration::from_secs(10)).await;

    info!("Market scrapping workers done");
}
