mod api;
mod my_ship;

mod sql;
mod workers;

mod tests;

use std::{env, sync::Arc};

use env_logger::{Env, Target};
use my_ship::MyShip;
use sql::insert_waypoint;

use crate::api::Api;
use log::info;

use std::num::NonZeroU32;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
        .target(Target::Stdout)
        .init();

    let access_token = env::var("ACCESS_TOKEN").ok();

    let api: Api = Api::new(access_token, 550, NonZeroU32::new(2).unwrap());

    // Create a connection pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            // format!(
            //     "postgres://{}:{}@{}/{}",
            //     env::var("POSTGRES_USER").unwrap(),
            //     env::var("POSTGRES_PASSWORD").unwrap(),
            //     "localhost",
            //     env::var("POSTGRES_DB").unwrap()
            // )
            // .as_str(),
            env::var("DATABASE_URL").unwrap().as_str(),
        )
        .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    // let row = sqlx::query_as!(sql::MarketTradeGood, r#"SELECT created_at, created, waypoint_symbol, symbol as "symbol: TradeSymbol", "type" as "type: models::market_trade_good::Type", trade_volume, supply as "supply: models::SupplyLevel", activity as "activity: models::ActivityLevel", purchase_price, sell_price FROM public.market_trade_good"#)
    //         .fetch_all(&pool)
    //         .await?;

    // info!("Row: {:?}", row);

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    // let insert = sqlx::query(
    //     "INSERT INTO waypoint (symbol, system_symbol) VALUES ($1, $2) ON CONFLICT (symbol) DO NOTHING;",
    // )
    // .bind(waypoint.symbol.clone())
    // .bind(waypoint.system_symbol.clone())
    // .execute(&pool)
    // .await?;

    // info!("Insert: {:?}", insert);

    let my_agent = api.get_my_agent().await?;
    info!("My agent: {:?}", my_agent);

    // let my_ships = api.get_my_ships().await?;

    // let systems_json = api.get_all_systems_json().await?;
    // info!("Systems json: {:?}", systems_json.len());

    // let systems = api.get_all_systems(20).await?;
    // info!(
    //     "Systems json: {:?} Systems: {:?}",
    //     systems_json.len(),
    //     systems.len()
    // );

    let ships = api.get_all_my_ships(20).await?;
    info!("Ships: {:?}", ships.len());

    let waypoints = api
        .get_all_waypoints(&ships[0].nav.system_symbol, 20)
        .await?;
    info!("Waypoints: {:?}", waypoints.len());

    insert_waypoint(&pool, &waypoints).await;

    let contracts = api.get_all_contracts(20).await?;
    info!("Contracts: {:?}", contracts.len());

    let my_ships = Arc::new(
        ships
            .iter()
            .map(|s| (s.symbol.clone(), MyShip::from_ship(s.clone())))
            .collect::<dashmap::DashMap<String, MyShip>>(),
    );

    info!("My ship: {:?}", my_ships.get("MOOSBEE-1").unwrap().value());

    let construction = tokio::spawn(async move {
        workers::construction_fleet::construction_conductor().await;
    });
    let contract = tokio::spawn(async move {
        workers::contract_fleet::contract_conductor().await;
    });
    let peel = pool.clone();
    let scrapping = tokio::spawn(async move {
        workers::market_scrapers::scrapping_conductor(&api, peel, waypoints).await;
    });
    let mining = tokio::spawn(async move {
        workers::mining_fleet::mining_conductor().await;
    });
    let trading = tokio::spawn(async move {
        workers::trading_fleet::trading_conductor(pool).await;
    });

    construction.await?;
    contract.await?;
    scrapping.await?;
    mining.await?;
    trading.await?;

    Ok(())
}
