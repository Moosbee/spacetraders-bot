use std::collections::{HashMap, HashSet};

use async_graphql::Object;
use serde::{Deserialize, Serialize};
use space_traders_client::models;
use tracing::instrument;

use crate::{DatabaseConnector, DbPool, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fleet {
    pub id: i32,
    pub system_symbol: String,
    pub fleet_type: FleetType,
    pub active: bool, // if false, ships should not be assigned or purchased for this fleet, but already assigned ships remain but are paused
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // add the config...
    // trading config
    market_blacklist: Option<Vec<models::TradeSymbol>>,
    market_prefer_list: Option<Vec<models::TradeSymbol>>,
    purchase_multiplier: Option<f64>,
    ship_market_ratio: Option<f64>,
    min_cargo_space: Option<i32>,
    trade_mode: Option<TradeMode>,
    trade_profit_threshold: Option<i32>,

    // scrapping config
    allowed_requests: Option<i32>,
    notify_on_shipyard: Option<bool>,

    // mining config
    mining_eject_list: Option<Vec<models::TradeSymbol>>,
    mining_prefer_list: Option<Vec<models::TradeSymbol>>,
    ignore_engineered_asteroids: Option<bool>,
    stop_all_unstable: Option<bool>,
    mining_waypoints: Option<i32>,
    unstable_since_timeout: Option<i32>,
    syphon_waypoints: Option<i32>,
    miners_per_waypoint: Option<i32>,
    siphoners_per_waypoint: Option<i32>,
    surveyors_per_waypoint: Option<i32>,
    mining_transporters_per_waypoint: Option<i32>,
    min_transporter_cargo_space: Option<i32>,
    min_mining_cargo_space: Option<i32>,
    min_siphon_cargo_space: Option<i32>,

    // charting config
    charting_probe_count: Option<i32>,

    // construction config
    construction_ship_count: Option<i32>,
    construction_waypoint: Option<String>,

    // contract config
    contract_ship_count: Option<i32>,
}

#[Object(name = "Fleet")]
impl Fleet {
    async fn id(&self) -> i32 {
        self.id
    }
    async fn system_symbol(&self) -> String {
        self.system_symbol.clone()
    }

    async fn fleet_type(&self) -> FleetType {
        self.fleet_type
    }

    async fn active(&self) -> bool {
        self.active
    }

    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    async fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }

    async fn config(&self) -> Result<FleetConfig> {
        self.get_config()
    }

    async fn system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<crate::System>> {
        let database_pool = ctx.data::<DbPool>().unwrap();
        crate::System::get_by_symbol(database_pool, &self.system_symbol).await
    }

    async fn assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<crate::ShipAssignment>> {
        let database_pool = ctx.data::<DbPool>().unwrap();
        crate::ShipAssignment::get_by_fleet_id(database_pool, self.id).await
    }
}

impl Default for Fleet {
    fn default() -> Self {
        Fleet {
            id: 0,
            system_symbol: String::new(),
            fleet_type: FleetType::Manuel,
            active: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            market_blacklist: None,
            market_prefer_list: None,
            purchase_multiplier: None,
            ship_market_ratio: None,
            min_cargo_space: None,
            trade_mode: None,
            trade_profit_threshold: None,
            allowed_requests: None,
            notify_on_shipyard: None,
            mining_eject_list: None,
            mining_prefer_list: None,
            ignore_engineered_asteroids: None,
            stop_all_unstable: None,
            mining_waypoints: None,
            unstable_since_timeout: None,
            syphon_waypoints: None,
            miners_per_waypoint: None,
            siphoners_per_waypoint: None,
            surveyors_per_waypoint: None,
            mining_transporters_per_waypoint: None,
            min_transporter_cargo_space: None,
            min_mining_cargo_space: None,
            min_siphon_cargo_space: None,
            charting_probe_count: None,
            construction_ship_count: None,
            construction_waypoint: None,
            contract_ship_count: None,
        }
    }
}

impl DatabaseConnector<Fleet> for Fleet {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &Fleet) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO fleet (
                  id,
                  system_symbol,
                  fleet_type,
                  active,
                  created_at,
                  updated_at,
                  market_blacklist,
                  market_prefer_list,
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode,
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list,
                  mining_prefer_list,
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                )
                VALUES (
                  $1, $2, $3::fleet_type, $4, NOW(), NOW(),
                  $5, $6, $7, $8, $9, $10::trade_mode, $11,
                  $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                  $25, $26, $27, $28, $29, $30, $31
                )
                ON CONFLICT (id) DO UPDATE SET
                  system_symbol = EXCLUDED.system_symbol,
                  fleet_type = EXCLUDED.fleet_type,
                  active = EXCLUDED.active,
                  updated_at = NOW(),
                  market_blacklist = EXCLUDED.market_blacklist,
                  market_prefer_list = EXCLUDED.market_prefer_list,
                  purchase_multiplier = EXCLUDED.purchase_multiplier,
                  ship_market_ratio = EXCLUDED.ship_market_ratio,
                  min_cargo_space = EXCLUDED.min_cargo_space,
                  trade_mode = EXCLUDED.trade_mode,
                  trade_profit_threshold = EXCLUDED.trade_profit_threshold,
                  allowed_requests = EXCLUDED.allowed_requests,
                  notify_on_shipyard = EXCLUDED.notify_on_shipyard,
                  mining_eject_list = EXCLUDED.mining_eject_list,
                  mining_prefer_list = EXCLUDED.mining_prefer_list,
                  ignore_engineered_asteroids = EXCLUDED.ignore_engineered_asteroids,
                  stop_all_unstable = EXCLUDED.stop_all_unstable,
                  mining_waypoints = EXCLUDED.mining_waypoints,
                  unstable_since_timeout = EXCLUDED.unstable_since_timeout,
                  syphon_waypoints = EXCLUDED.syphon_waypoints,
                  miners_per_waypoint = EXCLUDED.miners_per_waypoint,
                  siphoners_per_waypoint = EXCLUDED.siphoners_per_waypoint,
                  surveyors_per_waypoint = EXCLUDED.surveyors_per_waypoint,
                  mining_transporters_per_waypoint = EXCLUDED.mining_transporters_per_waypoint,
                  min_transporter_cargo_space = EXCLUDED.min_transporter_cargo_space,
                  min_mining_cargo_space = EXCLUDED.min_mining_cargo_space,
                  min_siphon_cargo_space = EXCLUDED.min_siphon_cargo_space,
                  charting_probe_count = EXCLUDED.charting_probe_count,
                  construction_ship_count = EXCLUDED.construction_ship_count,
                  construction_waypoint = EXCLUDED.construction_waypoint,
                  contract_ship_count = EXCLUDED.contract_ship_count;
            "#,
            &item.id,
            &item.system_symbol,
            &item.fleet_type as &FleetType,
            &item.active,
            &item.market_blacklist as &Option<Vec<models::TradeSymbol>>,
            &item.market_prefer_list as &Option<Vec<models::TradeSymbol>>,
            &item.purchase_multiplier as &Option<f64>,
            &item.ship_market_ratio as &Option<f64>,
            &item.min_cargo_space as &Option<i32>,
            &item.trade_mode as &Option<TradeMode>,
            &item.trade_profit_threshold as &Option<i32>,
            &item.allowed_requests as &Option<i32>,
            &item.notify_on_shipyard as &Option<bool>,
            &item.mining_eject_list as &Option<Vec<models::TradeSymbol>>,
            &item.mining_prefer_list as &Option<Vec<models::TradeSymbol>>,
            &item.ignore_engineered_asteroids as &Option<bool>,
            &item.stop_all_unstable as &Option<bool>,
            &item.mining_waypoints as &Option<i32>,
            &item.unstable_since_timeout as &Option<i32>,
            &item.syphon_waypoints as &Option<i32>,
            &item.miners_per_waypoint as &Option<i32>,
            &item.siphoners_per_waypoint as &Option<i32>,
            &item.surveyors_per_waypoint as &Option<i32>,
            &item.mining_transporters_per_waypoint as &Option<i32>,
            &item.min_transporter_cargo_space as &Option<i32>,
            &item.min_mining_cargo_space as &Option<i32>,
            &item.min_siphon_cargo_space as &Option<i32>,
            &item.charting_probe_count as &Option<i32>,
            &item.construction_ship_count as &Option<i32>,
            &item.construction_waypoint as &Option<String>,
            &item.contract_ship_count as &Option<i32>,
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Fleet]) -> crate::Result<()> {
        for item in items {
            Fleet::insert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<Fleet>> {
        let result = sqlx::query_as!(
            Fleet,
            r#"
                SELECT
                  id,
                  system_symbol,
                  fleet_type as "fleet_type: FleetType",
                  active,
                  created_at,
                  updated_at,
                  market_blacklist as "market_blacklist: Vec<models::TradeSymbol>",
                  market_prefer_list as "market_prefer_list: Vec<models::TradeSymbol>",
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode as "trade_mode: TradeMode",
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list as "mining_eject_list: Vec<models::TradeSymbol>",
                  mining_prefer_list as "mining_prefer_list: Vec<models::TradeSymbol>",
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                FROM fleet
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(result)
    }
}

impl Fleet {
    pub fn new(system_symbol: String, active: bool) -> Self {
        Fleet {
            id: 0,
            system_symbol,
            fleet_type: FleetType::Manuel,
            active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ..Default::default()
        }
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn insert_new(database_pool: &DbPool, item: &Fleet) -> crate::Result<i32> {
        let erg = sqlx::query!(
            r#"
                INSERT INTO fleet (
                  system_symbol,
                  fleet_type,
                  active,
                  created_at,
                  updated_at,
                  market_blacklist,
                  market_prefer_list,
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode,
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list,
                  mining_prefer_list,
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                )
                VALUES (
                  $1, $2::fleet_type, $3, NOW(), NOW(),
                  $4, $5, $6, $7, $8, $9::trade_mode, $10,
                  $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                  $25, $26, $27, $28, $29, $30
                )
                RETURNING id;
            "#,
            &item.system_symbol,
            &item.fleet_type as &FleetType,
            &item.active,
            &item.market_blacklist as &Option<Vec<models::TradeSymbol>>,
            &item.market_prefer_list as &Option<Vec<models::TradeSymbol>>,
            &item.purchase_multiplier as &Option<f64>,
            &item.ship_market_ratio as &Option<f64>,
            &item.min_cargo_space as &Option<i32>,
            &item.trade_mode as &Option<TradeMode>,
            &item.trade_profit_threshold as &Option<i32>,
            &item.allowed_requests as &Option<i32>,
            &item.notify_on_shipyard as &Option<bool>,
            &item.mining_eject_list as &Option<Vec<models::TradeSymbol>>,
            &item.mining_prefer_list as &Option<Vec<models::TradeSymbol>>,
            &item.ignore_engineered_asteroids as &Option<bool>,
            &item.stop_all_unstable as &Option<bool>,
            &item.mining_waypoints as &Option<i32>,
            &item.unstable_since_timeout as &Option<i32>,
            &item.syphon_waypoints as &Option<i32>,
            &item.miners_per_waypoint as &Option<i32>,
            &item.siphoners_per_waypoint as &Option<i32>,
            &item.surveyors_per_waypoint as &Option<i32>,
            &item.mining_transporters_per_waypoint as &Option<i32>,
            &item.min_transporter_cargo_space as &Option<i32>,
            &item.min_mining_cargo_space as &Option<i32>,
            &item.min_siphon_cargo_space as &Option<i32>,
            &item.charting_probe_count as &Option<i32>,
            &item.construction_ship_count as &Option<i32>,
            &item.construction_waypoint as &Option<String>,
            &item.contract_ship_count as &Option<i32>,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(erg.id)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_ids(
        database_pool: &DbPool,
        ids: HashSet<i32>,
    ) -> crate::Result<HashMap<i32, Fleet>> {
        let ids = ids.into_iter().collect::<Vec<i32>>();
        let resp = sqlx::query_as!(
            Fleet,
            r#"
              SELECT
                id,
                system_symbol,
                fleet_type as "fleet_type: FleetType",
                active,
                created_at,
                updated_at,
                market_blacklist as "market_blacklist: Vec<models::TradeSymbol>",
                market_prefer_list as "market_prefer_list: Vec<models::TradeSymbol>",
                purchase_multiplier,
                ship_market_ratio,
                min_cargo_space,
                trade_mode as "trade_mode: TradeMode",
                trade_profit_threshold,
                allowed_requests,
                notify_on_shipyard,
                mining_eject_list as "mining_eject_list: Vec<models::TradeSymbol>",
                mining_prefer_list as "mining_prefer_list: Vec<models::TradeSymbol>",
                ignore_engineered_asteroids,
                stop_all_unstable,
                mining_waypoints,
                unstable_since_timeout,
                syphon_waypoints,
                miners_per_waypoint,
                siphoners_per_waypoint,
                surveyors_per_waypoint,
                mining_transporters_per_waypoint,
                min_transporter_cargo_space,
                min_mining_cargo_space,
                min_siphon_cargo_space,
                charting_probe_count,
                construction_ship_count,
                construction_waypoint,
                contract_ship_count
              FROM fleet
              WHERE id = ANY($1)
          "#,
            &ids
        )
        .fetch_all(&database_pool.database_pool)
        .await?
        .into_iter()
        .map(|fleet| (fleet.id, fleet))
        .collect();

        Ok(resp)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_system(database_pool: &DbPool, system: &str) -> crate::Result<Vec<Fleet>> {
        let resp = sqlx::query_as!(
            Fleet,
            r#"
                SELECT
                  id,
                  system_symbol,
                  fleet_type as "fleet_type: FleetType",
                  active,
                  created_at,
                  updated_at,
                  market_blacklist as "market_blacklist: Vec<models::TradeSymbol>",
                  market_prefer_list as "market_prefer_list: Vec<models::TradeSymbol>",
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode as "trade_mode: TradeMode",
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list as "mining_eject_list: Vec<models::TradeSymbol>",
                  mining_prefer_list as "mining_prefer_list: Vec<models::TradeSymbol>",
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                FROM fleet
                WHERE system_symbol = $1
            "#,
            &system
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(resp)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_type(
        database_pool: &DbPool,
        fleet_type: FleetType,
    ) -> crate::Result<Vec<Fleet>> {
        let resp = sqlx::query_as!(
            Fleet,
            r#"
                SELECT
                  id,
                  system_symbol,
                  fleet_type as "fleet_type: FleetType",
                  active,
                  created_at,
                  updated_at,
                  market_blacklist as "market_blacklist: Vec<models::TradeSymbol>",
                  market_prefer_list as "market_prefer_list: Vec<models::TradeSymbol>",
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode as "trade_mode: TradeMode",
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list as "mining_eject_list: Vec<models::TradeSymbol>",
                  mining_prefer_list as "mining_prefer_list: Vec<models::TradeSymbol>",
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                FROM fleet
                WHERE fleet_type = $1
            "#,
            fleet_type as FleetType
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(resp)
    }

    pub async fn get_by_id(database_pool: &DbPool, id: i32) -> crate::Result<Option<Fleet>> {
        let resp = sqlx::query_as!(
            Fleet,
            r#"
                SELECT
                  id,
                  system_symbol,
                  fleet_type as "fleet_type: FleetType",
                  active,
                  created_at,
                  updated_at,
                  market_blacklist as "market_blacklist: Vec<models::TradeSymbol>",
                  market_prefer_list as "market_prefer_list: Vec<models::TradeSymbol>",
                  purchase_multiplier,
                  ship_market_ratio,
                  min_cargo_space,
                  trade_mode as "trade_mode: TradeMode",
                  trade_profit_threshold,
                  allowed_requests,
                  notify_on_shipyard,
                  mining_eject_list as "mining_eject_list: Vec<models::TradeSymbol>",
                  mining_prefer_list as "mining_prefer_list: Vec<models::TradeSymbol>",
                  ignore_engineered_asteroids,
                  stop_all_unstable,
                  mining_waypoints,
                  unstable_since_timeout,
                  syphon_waypoints,
                  miners_per_waypoint,
                  siphoners_per_waypoint,
                  surveyors_per_waypoint,
                  mining_transporters_per_waypoint,
                  min_transporter_cargo_space,
                  min_mining_cargo_space,
                  min_siphon_cargo_space,
                  charting_probe_count,
                  construction_ship_count,
                  construction_waypoint,
                  contract_ship_count
                FROM fleet
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(resp)
    }

    pub fn with_config(mut self, config: FleetConfig) -> Self {
        self.set_config(config);
        self
    }

    pub fn set_config(&mut self, config: FleetConfig) {
        match config {
            FleetConfig::Trading(cfg) => {
                self.fleet_type = FleetType::Trading;
                self.market_blacklist = Some(cfg.market_blacklist);
                self.market_prefer_list = Some(cfg.market_prefer_list);
                self.purchase_multiplier = Some(cfg.purchase_multiplier);
                self.ship_market_ratio = Some(cfg.ship_market_ratio);
                self.min_cargo_space = Some(cfg.min_cargo_space);
                self.trade_mode = Some(cfg.trade_mode);
                self.trade_profit_threshold = Some(cfg.trade_profit_threshold);
            }
            FleetConfig::Scraping(cfg) => {
                self.fleet_type = FleetType::Scrapping;
                self.ship_market_ratio = Some(cfg.ship_market_ratio);
                self.allowed_requests = Some(cfg.allowed_requests);
                self.notify_on_shipyard = Some(cfg.notify_on_shipyard);
            }
            FleetConfig::Mining(cfg) => {
                self.fleet_type = FleetType::Mining;
                self.mining_eject_list = Some(cfg.mining_eject_list);
                self.mining_prefer_list = Some(cfg.mining_prefer_list);
                self.ignore_engineered_asteroids = Some(cfg.ignore_engineered_asteroids);
                self.stop_all_unstable = Some(cfg.stop_all_unstable);
                self.mining_waypoints = Some(cfg.mining_waypoints);
                self.unstable_since_timeout = Some(cfg.unstable_since_timeout);
                self.syphon_waypoints = Some(cfg.syphon_waypoints);
                self.miners_per_waypoint = Some(cfg.miners_per_waypoint);
                self.siphoners_per_waypoint = Some(cfg.siphoners_per_waypoint);
                self.surveyors_per_waypoint = Some(cfg.surveyers_per_waypoint);
                self.mining_transporters_per_waypoint = Some(cfg.mining_transporters_per_waypoint);
                self.min_transporter_cargo_space = Some(cfg.min_transporter_cargo_space);
                self.min_mining_cargo_space = Some(cfg.min_mining_cargo_space);
                self.min_siphon_cargo_space = Some(cfg.min_siphon_cargo_space);
            }
            FleetConfig::Charting(cfg) => {
                self.fleet_type = FleetType::Charting;
                self.charting_probe_count = Some(cfg.charting_probe_count);
            }
            FleetConfig::Construction(cfg) => {
                self.fleet_type = FleetType::Construction;
                self.construction_ship_count = Some(cfg.construction_ship_count);
                self.construction_waypoint = Some(cfg.construction_waypoint);
            }
            FleetConfig::Contract(cfg) => {
                self.fleet_type = FleetType::Contract;
                self.contract_ship_count = Some(cfg.contract_ship_count);
            }
            FleetConfig::Manuel(_) => {
                self.fleet_type = FleetType::Manuel;
            }
        }
    }

    pub fn get_config(&self) -> Result<FleetConfig> {
        match self.fleet_type {
            FleetType::Trading => Ok(FleetConfig::Trading(
                self.get_trading_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Scrapping => Ok(FleetConfig::Scraping(
                self.get_scraping_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Mining => Ok(FleetConfig::Mining(
                self.get_mining_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Charting => Ok(FleetConfig::Charting(
                self.get_charting_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Construction => Ok(FleetConfig::Construction(
                self.get_construction_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Contract => Ok(FleetConfig::Contract(
                self.get_contract_config()
                    .ok_or(crate::Error::IncompleteFleetConfig { fleet_id: self.id })?,
            )),
            FleetType::Manuel => Ok(FleetConfig::Manuel(ManuelConfig::default())),
        }
    }

    pub fn get_trading_config(&self) -> Option<TradingConfig> {
        Some(TradingConfig {
            market_blacklist: self.market_blacklist.clone()?,
            market_prefer_list: self.market_prefer_list.clone()?,
            purchase_multiplier: self.purchase_multiplier?,
            ship_market_ratio: self.ship_market_ratio?,
            min_cargo_space: self.min_cargo_space?,
            trade_mode: self.trade_mode?,
            trade_profit_threshold: self.trade_profit_threshold?,
        })
    }

    pub fn get_scraping_config(&self) -> Option<ScrapingConfig> {
        Some(ScrapingConfig {
            ship_market_ratio: self.ship_market_ratio?,
            allowed_requests: self.allowed_requests?,
            notify_on_shipyard: self.notify_on_shipyard?,
        })
    }

    pub fn get_mining_config(&self) -> Option<MiningConfig> {
        Some(MiningConfig {
            mining_eject_list: self.mining_eject_list.clone()?,
            mining_prefer_list: self.mining_prefer_list.clone()?,
            ignore_engineered_asteroids: self.ignore_engineered_asteroids?,
            stop_all_unstable: self.stop_all_unstable?,
            mining_waypoints: self.mining_waypoints?,
            unstable_since_timeout: self.unstable_since_timeout?,
            syphon_waypoints: self.syphon_waypoints?,
            miners_per_waypoint: self.miners_per_waypoint?,
            siphoners_per_waypoint: self.siphoners_per_waypoint?,
            surveyers_per_waypoint: self.surveyors_per_waypoint?,
            mining_transporters_per_waypoint: self.mining_transporters_per_waypoint?,
            min_transporter_cargo_space: self.min_transporter_cargo_space?,
            min_mining_cargo_space: self.min_mining_cargo_space?,
            min_siphon_cargo_space: self.min_siphon_cargo_space?,
        })
    }

    pub fn get_charting_config(&self) -> Option<ChartingConfig> {
        Some(ChartingConfig {
            charting_probe_count: self.charting_probe_count?,
        })
    }

    pub fn get_construction_config(&self) -> Option<ConstructionConfig> {
        Some(ConstructionConfig {
            construction_ship_count: self.construction_ship_count?,
            construction_waypoint: self.construction_waypoint.clone()?,
        })
    }

    pub fn get_contract_config(&self) -> Option<ContractConfig> {
        Some(ContractConfig {
            contract_ship_count: self.contract_ship_count?,
        })
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    async_graphql::Enum,
)]
#[sqlx(type_name = "trade_mode")]
#[allow(clippy::enum_variant_names)]
pub enum TradeMode {
    ProfitPerHour,
    ProfitPerAPIRequest,
    ProfitPerTrip,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Default,
    sqlx::Type,
    PartialEq,
    Eq,
    async_graphql::Enum,
)]
#[sqlx(type_name = "fleet_type")]
pub enum FleetType {
    Trading,
    Scrapping,
    Mining,
    Charting,
    Construction,
    #[default]
    Manuel,
    Contract,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::Union)]
pub enum FleetConfig {
    Trading(TradingConfig),
    Scraping(ScrapingConfig),
    Mining(MiningConfig),
    Charting(ChartingConfig),
    Construction(ConstructionConfig),
    Contract(ContractConfig),
    Manuel(ManuelConfig),
}

impl Default for FleetConfig {
    fn default() -> Self {
        FleetConfig::Manuel(ManuelConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct TradingConfig {
    pub market_blacklist: Vec<models::TradeSymbol>,
    pub market_prefer_list: Vec<models::TradeSymbol>,
    pub purchase_multiplier: f64,
    pub ship_market_ratio: f64,
    pub min_cargo_space: i32,
    pub trade_mode: TradeMode,
    pub trade_profit_threshold: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct ScrapingConfig {
    pub ship_market_ratio: f64,
    pub allowed_requests: i32,
    pub notify_on_shipyard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct MiningConfig {
    pub mining_eject_list: Vec<models::TradeSymbol>,
    pub mining_prefer_list: Vec<models::TradeSymbol>,
    pub ignore_engineered_asteroids: bool,
    pub stop_all_unstable: bool,
    pub unstable_since_timeout: i32,
    pub mining_waypoints: i32,
    pub syphon_waypoints: i32,
    pub miners_per_waypoint: i32,
    pub siphoners_per_waypoint: i32,
    pub surveyers_per_waypoint: i32,
    pub mining_transporters_per_waypoint: i32,
    pub min_transporter_cargo_space: i32,
    pub min_mining_cargo_space: i32,
    pub min_siphon_cargo_space: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct ChartingConfig {
    pub charting_probe_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct ConstructionConfig {
    pub construction_ship_count: i32,
    pub construction_waypoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct ContractConfig {
    pub contract_ship_count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct ManuelConfig {
    pub config: String,
}
