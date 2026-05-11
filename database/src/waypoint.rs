use std::{collections::HashMap, str::FromStr, sync::Arc};

use space_traders_client::models;
use tracing::instrument;

use async_graphql::dataloader::Loader;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBWaypoint")]
pub struct Waypoint {
    pub symbol: String,
    pub system_symbol: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub x: i32,
    pub y: i32,
    pub waypoint_type: models::WaypointType,
    pub traits: Vec<models::WaypointTraitSymbol>,
    pub is_under_construction: bool,
    pub orbitals: Vec<String>,
    pub orbits: Option<String>,
    pub faction: Option<String>,
    pub modifiers: Vec<models::WaypointModifierSymbol>,
    pub charted_by: Option<String>,
    pub charted_on: Option<String>,
    pub unstable_since: Option<sqlx::types::chrono::DateTime<chrono::Utc>>,
    pub has_shipyard: bool,
    pub has_marketplace: bool,
}

impl From<Waypoint> for (i32, i32) {
    fn from(value: Waypoint) -> Self {
        (value.x, value.y)
    }
}
impl From<&Waypoint> for (i32, i32) {
    fn from(value: &Waypoint) -> Self {
        (value.x, value.y)
    }
}

impl From<&models::Waypoint> for Waypoint {
    fn from(value: &models::Waypoint) -> Self {
        Self {
            symbol: value.symbol.clone(),
            system_symbol: value.system_symbol.clone(),
            x: value.x,
            y: value.y,
            waypoint_type: value.r#type,
            traits: value.traits.iter().map(|t| t.symbol).collect(),
            is_under_construction: value.is_under_construction,
            orbitals: value.orbitals.iter().map(|o| o.symbol.clone()).collect(),
            orbits: value.orbits.clone(),
            faction: value.faction.as_ref().map(|f| f.symbol.to_string()),
            modifiers: value
                .modifiers
                .as_ref()
                .map(|m| m.iter().map(|t| t.symbol).collect::<Vec<_>>())
                .unwrap_or_default(),
            charted_by: value.chart.as_ref().map(|c| c.submitted_by.clone()),
            charted_on: value.chart.as_ref().map(|c| c.submitted_on.clone()),
            has_marketplace: value
                .traits
                .iter()
                .any(|t| t.symbol == models::WaypointTraitSymbol::Marketplace),
            has_shipyard: value
                .traits
                .iter()
                .any(|t| t.symbol == models::WaypointTraitSymbol::Shipyard),
            ..Default::default()
        }
    }
}

impl From<&Waypoint> for models::Waypoint {
    fn from(value: &Waypoint) -> Self {
        let chart = match (value.charted_by.as_ref(), value.charted_on.as_ref()) {
            (Some(charted_by), Some(charted_on)) => Some(models::Chart {
                submitted_by: charted_by.clone(),
                submitted_on: charted_on.clone(),
                waypoint_symbol: value.symbol.clone(),
            }),
            (None, Some(charted_on)) => Some(models::Chart {
                submitted_by: "".to_string(),
                submitted_on: charted_on.clone(),
                waypoint_symbol: value.symbol.clone(),
            }),
            (Some(charted_by), None) => Some(models::Chart {
                submitted_by: charted_by.clone(),
                submitted_on: "".to_string(),
                waypoint_symbol: value.symbol.clone(),
            }),
            _ => None,
        };

        let chart = chart.map(Box::new);

        let faction = value
            .faction
            .as_ref()
            .and_then(|f| models::FactionSymbol::from_str(f).ok())
            .map(|f| Box::new(models::WaypointFaction::new(f)));

        Self {
            symbol: value.symbol.clone(),
            system_symbol: value.system_symbol.clone(),
            x: value.x,
            y: value.y,
            r#type: value.waypoint_type,
            traits: value
                .traits
                .iter()
                .map(|t| models::WaypointTrait::new(*t, "".to_string(), "".to_string()))
                .collect(),
            is_under_construction: value.is_under_construction,
            orbitals: value
                .orbitals
                .iter()
                .map(|o| models::WaypointOrbital::new(o.clone()))
                .collect(),
            orbits: value.orbits.clone(),
            faction,
            modifiers: Some(
                value
                    .modifiers
                    .iter()
                    .map(|m| models::WaypointModifier::new(*m, "".to_string(), "".to_string()))
                    .collect::<Vec<_>>(),
            ),
            chart,
        }
    }
}

impl utils::WaypointCan for Waypoint {
    fn is_marketplace(&self) -> bool {
        self.traits
            .contains(&models::WaypointTraitSymbol::Marketplace)
            || self.has_marketplace
    }

    fn is_minable(&self) -> bool {
        self.waypoint_type == models::WaypointType::Asteroid
            || self.waypoint_type == models::WaypointType::AsteroidField
            || self.waypoint_type == models::WaypointType::EngineeredAsteroid
    }

    fn is_sipherable(&self) -> bool {
        self.waypoint_type == models::WaypointType::GasGiant
    }

    fn is_shipyard(&self) -> bool {
        self.traits.contains(&models::WaypointTraitSymbol::Shipyard) || self.has_shipyard
    }

    fn is_jump_gate(&self) -> bool {
        self.waypoint_type == models::WaypointType::JumpGate
    }

    fn is_charted(&self) -> bool {
        self.charted_by.is_some() || self.charted_on.is_some()
    }
}

pub struct WaypointSystemLoader(DbPool);

impl WaypointSystemLoader {
    pub fn new(database_pool: DbPool) -> Self {
        Self(database_pool)
    }
}

impl Loader<String> for WaypointSystemLoader {
    type Value = Vec<Waypoint>;
    type Error = Arc<crate::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut map: HashMap<String, Vec<Waypoint>> = HashMap::new();

        let waypoints = Waypoint::get_by_systems(
            &self.0,
            &keys.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
        )
        .await?;

        for waypoint in waypoints {
            map.entry(waypoint.system_symbol.clone())
                .or_default()
                .push(waypoint);
        }

        Ok(map)
    }
}

pub struct WaypointLoader(DbPool);

impl WaypointLoader {
    pub fn new(database_pool: DbPool) -> Self {
        Self(database_pool)
    }
}

impl Loader<String> for WaypointLoader {
    type Value = Waypoint;
    type Error = Arc<crate::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut map: HashMap<String, Waypoint> = HashMap::new();

        let waypoints = Waypoint::get_by_symbols(
            &self.0,
            &keys.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
        )
        .await?;

        for waypoint in waypoints {
            map.insert(waypoint.symbol.clone(), waypoint);
        }

        Ok(map)
    }
}

impl Waypoint {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_hash_map(
        database_pool: &DbPool,
    ) -> crate::Result<HashMap<String, HashMap<String, Waypoint>>> {
        let erg = Waypoint::get_all(database_pool, PaginatedQuery::unpaged())
            .await?
            .items;

        let mut map: HashMap<String, HashMap<String, Waypoint>> = HashMap::new();
        for waypoint in erg {
            map.entry(waypoint.system_symbol.clone())
                .or_default()
                .insert(waypoint.symbol.clone(), waypoint);
        }

        Ok(map)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_systems(
        database_pool: &DbPool,
        system_symbols: &[&str],
    ) -> crate::Result<Vec<Waypoint>> {
        let erg = sqlx::query_as!(
            Waypoint,
            r#"
                SELECT 
                  symbol,
                  system_symbol,
                  created_at,
                  x,
                  y,
                  type as "waypoint_type: models::WaypointType",
                  traits as "traits: Vec<models::WaypointTraitSymbol>",
                  is_under_construction,
                  orbitals,
                  orbits,
                  faction,
                  modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                  charted_by,
                  charted_on,
                  unstable_since,
                  has_shipyard,
                  has_marketplace
                FROM waypoint
                WHERE system_symbol = ANY($1)
            "#,
            system_symbols as &[&str]
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Waypoint>> {
        let system_symbol_page = system_symbol.to_string();
        let system_symbol_all = system_symbol.to_string();
        let system_symbol_count = system_symbol.to_string();

        run_paginated_query(
            query,
            move |limit, offset| async move {
                let erg = sqlx::query_as!(
                    Waypoint,
                    r#"
                        SELECT 
                          symbol,
                          system_symbol,
                          created_at,
                          x,
                          y,
                          type as "waypoint_type: models::WaypointType",
                          traits as "traits: Vec<models::WaypointTraitSymbol>",
                          is_under_construction,
                          orbitals,
                          orbits,
                          faction,
                          modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                          charted_by,
                          charted_on,
                          unstable_since,
                          has_shipyard,
                          has_marketplace
                        FROM waypoint
                        WHERE system_symbol = $1
                        ORDER BY symbol
                        LIMIT $2 OFFSET $3
                    "#,
                    &system_symbol_page,
                    limit,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(erg)
            },
            move || async move {
                let erg = sqlx::query_as!(
                    Waypoint,
                    r#"
                        SELECT 
                          symbol,
                          system_symbol,
                          created_at,
                          x,
                          y,
                          type as "waypoint_type: models::WaypointType",
                          traits as "traits: Vec<models::WaypointTraitSymbol>",
                          is_under_construction,
                          orbitals,
                          orbits,
                          faction,
                          modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                          charted_by,
                          charted_on,
                          unstable_since,
                          has_shipyard,
                          has_marketplace
                        FROM waypoint
                        WHERE system_symbol = $1
                        ORDER BY symbol
                    "#,
                    &system_symbol_all,
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(erg)
            },
            move || async move {
                let erg = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as count
                        FROM waypoint
                        WHERE system_symbol = $1
                    "#,
                    &system_symbol_count,
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(erg.count.unwrap_or(0))
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_symbols(
        database_pool: &DbPool,
        symbols: &[&str],
    ) -> crate::Result<Vec<Waypoint>> {
        let erg = sqlx::query_as!(
            Waypoint,
            r#"
                SELECT 
                  symbol,
                  system_symbol,
                  created_at,
                  x,
                  y,
                  type as "waypoint_type: models::WaypointType",
                  traits as "traits: Vec<models::WaypointTraitSymbol>",
                  is_under_construction,
                  orbitals,
                  orbits,
                  faction,
                  modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                  charted_by,
                  charted_on,
                  unstable_since,
                  has_shipyard,
                  has_marketplace
                FROM waypoint
                WHERE symbol = ANY($1)
            "#,
            symbols as &[&str]
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnectorAsync for Waypoint {
    type ID = String;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &Waypoint) -> crate::Result<Self::ID> {
        sqlx::query!(
            r#"
                INSERT INTO waypoint (
	                symbol,
                  system_symbol,
                  x,
                  y,
                  type,
                  traits,
                  is_under_construction,
                  orbitals,
                  orbits,
                  faction,
                  modifiers,
                  charted_by,
                  charted_on,
                  unstable_since,
                  has_shipyard,
                  has_marketplace
                )
                VALUES ($1,
                        $2,
                        $3,
                        $4,
                        $5::waypoint_type,
                        $6::waypoint_trait_symbol[],
                        $7,
                        $8,
                        $9,
                        $10,
                        $11::waypoint_modifier_symbol[],
                        $12,
                        $13,
                        $14,
                        $15,
                        $16
                        )
                ON CONFLICT (symbol) DO UPDATE SET 
                system_symbol = EXCLUDED.system_symbol,
                x = EXCLUDED.x,
                y = EXCLUDED.y,
                type = EXCLUDED.type,
                traits = EXCLUDED.traits,
                is_under_construction = EXCLUDED.is_under_construction,
                orbitals = EXCLUDED.orbitals,
                orbits = EXCLUDED.orbits,
                faction = EXCLUDED.faction,
                modifiers = EXCLUDED.modifiers,
                charted_by = EXCLUDED.charted_by,
                charted_on = EXCLUDED.charted_on,
                unstable_since = EXCLUDED.unstable_since,
                has_shipyard = EXCLUDED.has_shipyard,
                has_marketplace = EXCLUDED.has_marketplace;
            "#,
            &item.symbol,
            &item.system_symbol,
            &item.x,
            &item.y,
            &item.waypoint_type as &models::WaypointType,
            &item.traits as &[models::WaypointTraitSymbol],
            &item.is_under_construction,
            &item.orbitals,
            &item.orbits as &Option<String>,
            &item.faction as &Option<String>,
            &item.modifiers as &[models::WaypointModifierSymbol],
            &item.charted_by as &Option<String>,
            &item.charted_on as &Option<String>,
            &item.unstable_since as &Option<sqlx::types::chrono::DateTime<chrono::Utc>>,
            &item.has_shipyard,
            &item.has_marketplace
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(item.symbol.clone())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &Waypoint) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &Waypoint) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Waypoint]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Waypoint>> {
        run_paginated_query(
            query,
            |limit, offset| async move {
                let erg = sqlx::query_as!(
                    Waypoint,
                    r#"
                        SELECT 
                          symbol,
                          system_symbol,
                          created_at,
                          x,
                          y,
                          type as "waypoint_type: models::WaypointType",
                          traits as "traits: Vec<models::WaypointTraitSymbol>",
                          is_under_construction,
                          orbitals,
                          orbits,
                          faction,
                          modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                          charted_by,
                          charted_on,
                          unstable_since,
                          has_shipyard,
                          has_marketplace
                        FROM waypoint
                        ORDER BY symbol
                        LIMIT $1 OFFSET $2
                    "#,
                    limit,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(erg)
            },
            || async move {
                let erg = sqlx::query_as!(
                    Waypoint,
                    r#"
                        SELECT 
                          symbol,
                          system_symbol,
                          created_at,
                          x,
                          y,
                          type as "waypoint_type: models::WaypointType",
                          traits as "traits: Vec<models::WaypointTraitSymbol>",
                          is_under_construction,
                          orbitals,
                          orbits,
                          faction,
                          modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                          charted_by,
                          charted_on,
                          unstable_since,
                          has_shipyard,
                          has_marketplace
                        FROM waypoint
                        ORDER BY symbol
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(erg)
            },
            || async move {
                let erg = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as count
                        FROM waypoint
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(erg.count.unwrap_or(0))
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            Waypoint,
            r#"
                SELECT 
                  symbol,
                  system_symbol,
                  created_at,
                  x,
                  y,
                  type as "waypoint_type: models::WaypointType",
                  traits as "traits: Vec<models::WaypointTraitSymbol>",
                  is_under_construction,
                  orbitals,
                  orbits,
                  faction,
                  modifiers as "modifiers: Vec<models::WaypointModifierSymbol>",
                  charted_by,
                  charted_on,
                  unstable_since,
                  has_shipyard,
                  has_marketplace
                FROM waypoint
                WHERE symbol = $1
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM waypoint
                WHERE symbol = $1
            "#,
            id,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
