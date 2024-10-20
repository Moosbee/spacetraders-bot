

mod api;

use std::env;

use env_logger::{Env, Target};

use crate::api::Api;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  println!("Hello, world!");

  dotenvy::dotenv()?;

    let env = Env::default()
        .filter_or("RUST_LOG", "spacetraders_rs=trace")
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
        .target(Target::Stdout)
        .init();

    let access_token = match env::var("ACCESS_TOKEN") {
        Ok(v) => v,
        Err(_) => "".to_string(),
    };

    let api = Api::new(access_token);

    // let register_response = Api::register(ACCOUNT_SYMBOL.to_string(), Faction::Cosmic).await?;
    // info!("Register response: {:?}", register_response);

    let my_agent = api.get_my_agent().await?;
    info!("My agent: {:?}", my_agent);
    let my_contracts = api.get_my_contracts().await?;
    info!("My contracts {:?}", my_contracts);

    Ok(())
}
