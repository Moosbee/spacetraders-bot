use std::{collections::HashMap, str::FromStr};

use space_traders_client::models;

use crate::types::WaypointCan;

use super::{DatabaseConnector, DbPool};

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct Waypoint {
    pub symbol: String,
    pub system_symbol: String,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
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
    pub unstable_since: Option<sqlx::types::chrono::NaiveDateTime>,
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
                .map(|m| {
                    let mm = m.iter().map(|t| t.symbol).collect::<Vec<_>>();
                    mm
                })
                .unwrap_or_default(),
            charted_by: value.chart.as_ref().and_then(|c| c.submitted_by.clone()),
            charted_on: value.chart.as_ref().and_then(|c| c.submitted_on.clone()),
            ..Default::default()
        }
    }
}

impl From<&Waypoint> for models::Waypoint {
    fn from(value: &Waypoint) -> Self {
        let chart = match (value.charted_by.as_ref(), value.charted_on.as_ref()) {
            (Some(charted_by), Some(charted_on)) => Some(models::Chart {
                submitted_by: Some(charted_by.clone()),
                submitted_on: Some(charted_on.clone()),
                waypoint_symbol: Some(value.symbol.clone()),
            }),
            (None, Some(charted_on)) => Some(models::Chart {
                submitted_by: None,
                submitted_on: Some(charted_on.clone()),
                waypoint_symbol: Some(value.symbol.clone()),
            }),
            (Some(charted_by), None) => Some(models::Chart {
                submitted_by: Some(charted_by.clone()),
                submitted_on: None,
                waypoint_symbol: Some(value.symbol.clone()),
            }),
            _ => None,
        };

        let chart = chart.map(Box::new);

        let faction = value
            .faction
            .as_ref()
            .and_then(|f| models::FactionSymbol::from_str(f).ok())
            .map(|f| Box::new(models::WaypointFaction::new(f)));

        let erg = Self {
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
        };

        erg
    }
}

impl WaypointCan for Waypoint {
    fn is_marketplace(&self) -> bool {
        self.traits
            .contains(&models::WaypointTraitSymbol::Marketplace)
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
        self.traits.contains(&models::WaypointTraitSymbol::Shipyard)
    }

    fn is_jump_gate(&self) -> bool {
        self.waypoint_type == models::WaypointType::JumpGate
    }

    fn is_charted(&self) -> bool {
        self.charted_by.is_some() || self.charted_on.is_some()
    }
}

impl Waypoint {
    pub async fn get_hash_map(
        database_pool: &DbPool,
    ) -> sqlx::Result<HashMap<String, HashMap<String, Waypoint>>> {
        let erg = Waypoint::get_all(database_pool).await?;

        let mut map: HashMap<String, HashMap<String, Waypoint>> = HashMap::new();
        for waypoint in erg {
            map.entry(waypoint.system_symbol.clone())
                .or_default()
                .insert(waypoint.symbol.clone(), waypoint);
        }

        Ok(map)
    }

    pub async fn get_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
    ) -> sqlx::Result<Vec<Waypoint>> {
        sqlx::query_as!(
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
                  unstable_since
                FROM waypoint
                WHERE system_symbol = $1
            "#,
            system_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_symbol(
        database_pool: &DbPool,
        symbol: &str,
    ) -> sqlx::Result<Option<Waypoint>> {
        sqlx::query_as!(
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
                  unstable_since
                FROM waypoint
                WHERE symbol = $1
                LIMIT 1
            "#,
            symbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<Waypoint> for Waypoint {
    async fn insert(database_pool: &DbPool, item: &Waypoint) -> sqlx::Result<()> {
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
                  unstable_since
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
                        $14
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
                unstable_since = EXCLUDED.unstable_since;
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
            &item.unstable_since as &Option<sqlx::types::chrono::NaiveDateTime>,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[Waypoint]) -> sqlx::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }

        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<Waypoint>> {
        sqlx::query_as!(
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
                  unstable_since
                FROM waypoint
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
