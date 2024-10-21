mod api;

use std::env;

use env_logger::{Env, Target};

use crate::api::Api;
use log::info;

use std::num::NonZeroU32;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
        .target(Target::Stdout)
        .init();

    let access_token = match env::var("ACCESS_TOKEN") {
        Ok(v) => v,
        Err(_) => "".to_string(),
    };

    let api: Api = Api::new(access_token, 550, NonZeroU32::new(2).unwrap());

    let my_agent = api.get_my_agent().await?;
    info!("My agent: {:?}", my_agent);

    // let my_ships = api.get_my_ships().await?;

    Ok(())
}
