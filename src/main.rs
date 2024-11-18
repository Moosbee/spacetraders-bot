mod api;
mod ship;

mod sql;
mod workers;

mod tests;

use std::{collections::HashMap, env, sync::Arc};

use dashmap::DashMap;
use env_logger::{Env, Target};
use space_traders_client::models::waypoint;
use sql::DatabaseConnector;
use workers::types::Conductor;

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

    sql::Waypoint::insert_bulk(
        &database_pool,
        &waypoints
            .iter()
            .map(sql::Waypoint::from)
            .collect::<Vec<_>>(),
    )
    .await
    .unwrap();

    let ship_roles: std::collections::HashMap<String, ship::models::Role> = vec![
        ("MOOSBEE-1".to_string(), ship::models::Role::Contract),
        ("MOOSBEE-2".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-3".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-4".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-5".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-6".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-7".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-8".to_string(), ship::models::Role::Trader),
        ("MOOSBEE-9".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-A".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-B".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-C".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-D".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-E".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-F".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-10".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-11".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-12".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-13".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-14".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-15".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-16".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-17".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-18".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-19".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1A".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1B".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1C".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1D".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1E".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-1F".to_string(), ship::models::Role::Scraper),
        ("MOOSBEE-20".to_string(), ship::models::Role::Scraper),
    ]
    .clone()
    .into_iter()
    .collect();

    let my_ships: Arc<dashmap::DashMap<String, ship::MyShip>> = Arc::new(
        ships
            .iter()
            .map(|s| {
                let mut shipi = ship::MyShip::from_ship(s.clone());
                shipi.role = *ship_roles
                    .get(&s.symbol)
                    .unwrap_or(&ship::models::Role::Manuel);

                (s.symbol.clone(), shipi)
            })
            .collect(),
    );

    let all_waypoints: Arc<dashmap::DashMap<String, HashMap<String, waypoint::Waypoint>>> =
        Arc::new(DashMap::new());

    {
        let mut a_wps = all_waypoints
            .entry(waypoints[0].system_symbol.clone())
            .or_default();
        for wp in waypoints.iter() {
            a_wps.insert(wp.symbol.clone(), wp.clone());
        }
    }

    // info!("My ship: {:?}", my_ships);

    let context = workers::types::ConductorContext {
        api: api.clone(),
        database_pool: database_pool.clone(),
        my_ships: my_ships.clone(),
        all_waypoints: all_waypoints.clone(),
        ship_roles: ship_roles.clone(),
    };

    let conductors: Vec<Box<dyn Conductor>> = vec![
        // workers::construction_fleet::ConstructionFleet::new_box(context.clone()),
        // workers::contract_fleet::ContractFleet::new_box(context.clone()),
        workers::market_scrapers::MarketScraper::new_box(context.clone()),
        // workers::mining_fleet::MiningFleet::new_box(context.clone()),
        // workers::trading_fleet::TradingFleet::new_box(context.clone()),
    ];

    let conductor_join_handles = conductors
        .into_iter()
        .map(|c| {
            (
                c.get_name(),
                tokio::task::spawn(async move { c.run().await }),
            )
        })
        .collect::<Vec<_>>();

    for handle in conductor_join_handles {
        let name = handle.0;
        let erg = handle.1.await;
        println!("{}: {:?}", name, erg);
        if let Err(errror) = erg {
            println!("{} error: {} {:?}", name, errror, errror);
        } else if let Ok(r_erg) = erg {
            if let Err(errror) = r_erg {
                println!(
                    "{} error: {} {:?} {:?} {:?}",
                    name,
                    errror,
                    errror.backtrace(),
                    errror.source(),
                    errror.root_cause()
                );
            } else if let Ok(_res) = r_erg {
            }
        }
    }

    Ok(())
}

pub trait IsMarketplace {
    fn is_marketplace(&self) -> bool;
}

impl IsMarketplace for space_traders_client::models::Waypoint {
    fn is_marketplace(&self) -> bool {
        self.traits
            .iter()
            .any(|t| t.symbol == space_traders_client::models::WaypointTraitSymbol::Marketplace)
    }
}
