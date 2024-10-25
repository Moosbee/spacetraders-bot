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

    // Create a connection database_pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let database_pool = PgPoolOptions::new()
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
    //         .fetch_all(&database_pool)
    //         .await?;

    // info!("Row: {:?}", row);

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

    insert_waypoint(&database_pool, &waypoints).await;

    let ship_roles: std::collections::HashMap<String, my_ship::Role> = vec![
        ("MOOSBEE-1".to_string(), my_ship::Role::Manuel),
        ("MOOSBEE-2".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-3".to_string(), my_ship::Role::Trader),
        ("MOOSBEE-4".to_string(), my_ship::Role::Trader),
        ("MOOSBEE-5".to_string(), my_ship::Role::Manuel),
        ("MOOSBEE-6".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-7".to_string(), my_ship::Role::Trader),
        ("MOOSBEE-8".to_string(), my_ship::Role::Trader),
        ("MOOSBEE-9".to_string(), my_ship::Role::Trader),
        ("MOOSBEE-A".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-B".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-C".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-D".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-E".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-F".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-10".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-11".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-12".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-13".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-14".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-15".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-16".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-17".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-18".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-19".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-1A".to_string(), my_ship::Role::Scraper),
        ("MOOSBEE-1B".to_string(), my_ship::Role::Scraper),
    ]
    .clone()
    .into_iter()
    .collect();

    let my_ships = Arc::new(
        ships
            .iter()
            .map(|s| {
                let mut shipi = MyShip::from_ship(s.clone());
                shipi.role = ship_roles
                    .get(&s.symbol)
                    .unwrap_or(&my_ship::Role::Manuel)
                    .clone();

                (s.symbol.clone(), shipi)
            })
            .collect::<dashmap::DashMap<String, MyShip>>(),
    );

    info!("My ship: {:?}", my_ships);

    let construction = tokio::spawn(async move {
        workers::construction_fleet::construction_conductor().await;
    });

    let pool_2 = database_pool.clone();
    let api_2 = api.clone();
    let waypoints_2 = waypoints.clone();
    let ship_roles_2 = ship_roles.clone();
    let my_ships_2: Arc<dashmap::DashMap<String, MyShip>> = my_ships.clone();
    let contract = tokio::spawn(async move {
        workers::contract_fleet::contract_conductor(
            api_2,
            pool_2,
            ship_roles_2,
            my_ships_2,
            waypoints_2,
        )
        .await
    });
    let pool_3 = database_pool.clone();
    let scrapping = tokio::spawn(async move {
        workers::market_scrapers::scrapping_conductor(api, pool_3, waypoints).await;
    });
    let mining = tokio::spawn(async move {
        workers::mining_fleet::mining_conductor().await;
    });
    let trading = tokio::spawn(async move {
        workers::trading_fleet::trading_conductor(database_pool).await;
    });

    construction.await?;
    let _ = contract.await?;
    scrapping.await?;
    mining.await?;
    trading.await?;

    Ok(())
}
