mod api;
mod ship;

mod sql;
mod workers;

mod config;
mod tests;

mod control_api;
mod types;

use std::{collections::HashMap, env, sync::Arc};

use chrono::{DateTime, Utc};
use config::CONFIG;
use dashmap::DashMap;
use env_logger::{Env, Target};
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models::waypoint;
use sql::DatabaseConnector;
use tokio::sync::broadcast;
use workers::{mining::m_types::MiningShipAssignment, types::Conductor};

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

    let client = AsyncSntpClient::new();
    let result = client.synchronize("pool.ntp.org").await.unwrap();
    let local_time: DateTime<Utc> =
        DateTime::from(result.datetime().into_chrono_datetime().unwrap());
    let time_diff = (local_time - Utc::now()).abs();

    info!(
        "The local time is: {} and it should be: {} and the time diff is: {:?}",
        Utc::now(),
        local_time,
        time_diff.to_std().unwrap()
    );

    if time_diff > chrono::Duration::milliseconds(1000) {
        panic!("The time is not correct");
    }

    let access_token = env::var("ACCESS_TOKEN").ok();

    let api: Api = Api::new(access_token, 550, NonZeroU32::new(2).unwrap());

    // Create a connection database_pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let database_connector = PgPoolOptions::new()
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

    let database_pool = sql::DbPool::new(database_connector);

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

    info!("{:?}", CONFIG.clone());

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

    let ship_roles: std::collections::HashMap<String, ship::Role> = vec![
        ("MOOSBEE-1".to_string(), ship::Role::Contract(None)),
        // ("MOOSBEE-1".to_string(), ship::Role::Trader(None)),
        ("MOOSBEE-2".to_string(), ship::Role::Scraper),
        ("MOOSBEE-3".to_string(), ship::Role::Scraper),
        ("MOOSBEE-4".to_string(), ship::Role::Scraper),
        ("MOOSBEE-5".to_string(), ship::Role::Scraper),
        ("MOOSBEE-6".to_string(), ship::Role::Scraper),
        ("MOOSBEE-7".to_string(), ship::Role::Scraper),
        ("MOOSBEE-8".to_string(), ship::Role::Scraper),
        ("MOOSBEE-9".to_string(), ship::Role::Scraper),
        ("MOOSBEE-A".to_string(), ship::Role::Scraper),
        ("MOOSBEE-B".to_string(), ship::Role::Scraper),
        ("MOOSBEE-C".to_string(), ship::Role::Scraper),
        ("MOOSBEE-D".to_string(), ship::Role::Scraper),
        ("MOOSBEE-E".to_string(), ship::Role::Scraper),
        ("MOOSBEE-F".to_string(), ship::Role::Scraper),
        ("MOOSBEE-10".to_string(), ship::Role::Scraper),
        ("MOOSBEE-11".to_string(), ship::Role::Scraper),
        ("MOOSBEE-12".to_string(), ship::Role::Scraper),
        ("MOOSBEE-13".to_string(), ship::Role::Scraper),
        ("MOOSBEE-14".to_string(), ship::Role::Scraper),
        ("MOOSBEE-15".to_string(), ship::Role::Scraper),
        ("MOOSBEE-16".to_string(), ship::Role::Scraper),
        ("MOOSBEE-17".to_string(), ship::Role::Scraper),
        ("MOOSBEE-18".to_string(), ship::Role::Scraper),
        ("MOOSBEE-19".to_string(), ship::Role::Scraper),
        ("MOOSBEE-1A".to_string(), ship::Role::Scraper),
        ("MOOSBEE-1B".to_string(), ship::Role::Trader(None)),
        (
            "MOOSBEE-1C".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-1D".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-1E".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-1F".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-20".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-21".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-22".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-23".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-24".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-25".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-26".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-27".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-28".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-29".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-2A".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-2B".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-2C".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-2D".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        (
            "MOOSBEE-2E".to_string(),
            ship::Role::Mining(MiningShipAssignment::Idle),
        ),
        ("MOOSBEE-2F".to_string(), ship::Role::Trader(None)),
    ]
    .clone()
    .into_iter()
    .collect();

    let ship_manager = Arc::new(ship::ShipManager::new()); // ship::ShipManager::new();

    let (sender, receiver) = broadcast::channel(1024);

    let broadcaster = ship::my_ship_update::InterShipBroadcaster { sender, receiver };

    for ship in ships {
        let mut ship_i = ship::MyShip::from_ship(ship.clone(), broadcaster.clone());
        ship_i.role = ship_roles
            .get(&ship.symbol)
            .unwrap_or(&ship::Role::Manuel)
            .clone();

        ShipManager::add_ship(&ship_manager, ship_i).await;
    }

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
        database_pool,
        ship_manager,
        all_waypoints: all_waypoints.clone(),
        ship_roles: ship_roles.clone(),
    };

    let mut conductors: Vec<Box<dyn Conductor>> = vec![
        workers::construction_fleet::ConstructionFleet::new_box(context.clone()),
        workers::contract_fleet::ContractFleet::new_box(context.clone()),
        workers::mining::mining_fleet::MiningFleet::new_box(context.clone()),
        workers::trading::trading_fleet::TradingFleet::new_box(context.clone()),
        workers::market_scrapers::MarketScraper::new_box(context.clone()),
    ];
    conductors.push(control_api::server::ControlApiServer::new_box(
        context.clone(),
        context.ship_manager.get_rx(),
        conductors
            .iter()
            .map(|c| (c.get_name(), c.is_independent(), c.get_cancel_token()))
            .collect(),
    ));

    let conductor_join_handles = conductors
        .into_iter()
        .map(|mut c| {
            (
                c.get_name(),
                c.is_independent(),
                c.get_cancel_token(),
                tokio::task::spawn(async move { c.run().await }),
            )
        })
        .collect::<Vec<_>>();

    for handle in conductor_join_handles {
        let name = handle.0;
        if !handle.1 {
            handle.2.cancel();
        }
        let erg = handle.3.await;
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
