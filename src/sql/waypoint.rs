use std::collections::HashMap;

use space_traders_client::models;

use crate::types::WaypointCan;

use super::{
    sql_models::{DatabaseConnector, Waypoint},
    DbPool,
};

impl From<&models::Waypoint> for super::sql_models::Waypoint {
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
            charted_by: value
                .chart
                .as_ref()
                .map(|c| c.submitted_by.clone())
                .flatten(),
            charted_on: value
                .chart
                .as_ref()
                .map(|c| c.submitted_on.clone())
                .flatten(),
            ..Default::default()
        }
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
}

impl Default for Waypoint {
    fn default() -> Self {
        Self {
            symbol: Default::default(),
            system_symbol: Default::default(),
            created_at: Default::default(),
            x: Default::default(),
            y: Default::default(),
            waypoint_type: Default::default(),
            traits: Default::default(),
            is_under_construction: Default::default(),
            orbitals: Default::default(),
            orbits: Default::default(),
            faction: Default::default(),
            modifiers: Default::default(),
            charted_by: Default::default(),
            charted_on: Default::default(),
        }
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
                  charted_on
                FROM waypoint
                WHERE system_symbol = $1
            "#,
            system_symbol
        )
        .fetch_all(&database_pool.database_pool)
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
                  charted_on
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
                        $13
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
                charted_on = EXCLUDED.charted_on;
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
            &item.charted_on as &Option<String>
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<Waypoint>) -> sqlx::Result<()> {
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
                  charted_on
                FROM waypoint
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
