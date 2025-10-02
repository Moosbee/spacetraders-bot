use tracing::instrument;

use super::DatabaseConnector;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ShipInfo {
    pub symbol: String,
    pub display_name: String,
    pub role: ShipInfoRole,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "ship_info_role")]
pub enum ShipInfoRole {
    Transfer,
    Construction,
    TempTrader,
    Trader,
    Contract,
    Scraper,
    Mining,
    Charter,
    #[default]
    Manuel,
}

impl TryFrom<&str> for ShipInfoRole {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Construction" => Ok(ShipInfoRole::Construction),
            "Trader" => Ok(ShipInfoRole::Trader),
            "Contract" => Ok(ShipInfoRole::Contract),
            "Scraper" => Ok(ShipInfoRole::Scraper),
            "Mining" => Ok(ShipInfoRole::Mining),
            "Charter" => Ok(ShipInfoRole::Charter),
            "Manuel" => Ok(ShipInfoRole::Manuel),
            "Transfer" => Ok(ShipInfoRole::Transfer),
            _ => Err(crate::Error::InvalidShipInfoRole(value.to_string())),
        }
    }
}

// impl From<ship::ShipStatus> for ShipInfoRole {
//     fn from(role: ship::ShipStatus) -> Self {
//         match role {
//             ship::ShipStatus::Construction { .. } => Self::Construction,
//             ship::ShipStatus::Trader { .. } => Self::Trader,
//             ship::ShipStatus::Contract { .. } => Self::Contract,
//             ship::ShipStatus::Scraper { .. } => Self::Scraper,
//             ship::ShipStatus::Mining { .. } => Self::Mining,
//             ship::ShipStatus::Manuel => Self::Manuel,
//             ship::ShipStatus::Charting { .. } => Self::Charter,
//         }
//     }
// }

// impl From<ShipInfoRole> for ship::ShipStatus {
//     fn from(role: ShipInfoRole) -> Self {
//         match role {
//             ShipInfoRole::Construction => Self::Construction {
//                 cycle: None,
//                 shipment_id: None,
//                 shipping_status: None,
//                 waiting_for_manager: false,
//             },
//             ShipInfoRole::Trader => Self::Trader {
//                 cycle: None,
//                 shipment_id: None,
//                 shipping_status: None,
//                 waiting_for_manager: false,
//             },
//             ShipInfoRole::Contract => Self::Contract {
//                 contract_id: None,
//                 run_id: None,
//                 cycle: None,
//                 shipping_status: None,
//                 waiting_for_manager: false,
//             },
//             ShipInfoRole::Scraper => Self::Scraper {
//                 cycle: None,
//                 waiting_for_manager: false,
//                 waypoint_symbol: None,
//                 scrap_date: None,
//             },
//             ShipInfoRole::Mining => Self::Mining {
//                 assignment: MiningShipAssignment::Idle,
//             },
//             ShipInfoRole::Manuel => Self::Manuel,
//             ShipInfoRole::TempTrader => Self::Trader {
//                 cycle: None,
//                 shipment_id: None,
//                 shipping_status: None,
//                 waiting_for_manager: false,
//             },
//             ShipInfoRole::Charter => Self::Charting {
//                 cycle: None,
//                 waiting_for_manager: false,
//                 waypoint_symbol: None,
//             },
//         }
//     }
// }

impl ShipInfo {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> crate::Result<Option<ShipInfo>> {
        let erg = sqlx::query_as!(
            ShipInfo,
            r#"
        SELECT symbol, display_name, role as "role: ShipInfoRole", active
        FROM ship_info WHERE symbol = $1
        LIMIT 1
      "#,
            symbol
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;

        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_role(
        database_pool: &super::DbPool,
        symbol: &ShipInfoRole,
    ) -> crate::Result<Vec<ShipInfo>> {
        let erg = sqlx::query_as!(
            ShipInfo,
            r#"
        SELECT symbol, display_name, role as "role: ShipInfoRole", active
        FROM ship_info WHERE role = $1
      "#,
            symbol as &ShipInfoRole
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }
}

impl DatabaseConnector<ShipInfo> for ShipInfo {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &ShipInfo) -> crate::Result<()> {
        sqlx::query!(
            r#"
              INSERT INTO public.ship_info(
                symbol,
                display_name,
                role,
                active
                ) VALUES (
                 $1,
                 $2,
                 $3::ship_info_role,
                 $4
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                role = EXCLUDED.role,
                active = EXCLUDED.active;
            "#,
            &item.symbol,
            &item.display_name,
            &item.role as &ShipInfoRole,
            &item.active
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[ShipInfo]) -> crate::Result<()> {
        let (symbol_s, display_name_s, role_s, active_s): (
            Vec<String>,
            Vec<String>,
            Vec<ShipInfoRole>,
            Vec<bool>,
        ) = itertools::Itertools::multiunzip(
            items
                .iter()
                .cloned()
                .map(|s| (s.symbol.clone(), s.display_name.clone(), s.role, s.active)),
        );

        sqlx::query!(
            r#"
              INSERT INTO public.ship_info (
                symbol,
                display_name,
                role,
                active
                )
                SELECT * FROM UNNEST(
                  $1::character varying[],
                  $2::character varying[],
                  $3::ship_info_role[],
                  $4::boolean[]
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                role = EXCLUDED.role,
                active = EXCLUDED.active
            "#,
            &symbol_s as &[String],
            &display_name_s as &[String],
            &role_s as &[ShipInfoRole],
            &active_s as &[bool]
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<ShipInfo>> {
        let erg = sqlx::query_as! {
            ShipInfo,
            r#"
                SELECT 
                    symbol,
                    display_name,
                    role as "role: ShipInfoRole",
                    active
                FROM ship_info
            "#
        }
        .fetch_all(database_pool.get_cache_pool())
        .await?;

        Ok(erg)
    }
}
