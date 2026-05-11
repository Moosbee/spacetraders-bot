use tracing::instrument;

use super::{
    run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipInfo")]
pub struct ShipInfo {
    pub symbol: String,
    pub display_name: String,
    pub active: bool, // if false ship is paused, does not request new assignments, but holds current assignment
    pub assignment_id: Option<i64>,
    pub temp_assignment_id: Option<i64>,
    pub purchase_id: Option<i64>,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, sqlx::Type)]
// #[sqlx(type_name = "ship_info_role")]
// pub enum ShipInfoRole {
//     Transfer,
//     Construction,
//     TempTrader,
//     Trader,
//     Contract,
//     Scraper,
//     Mining,
//     Charter,
//     #[default]
//     Manuel,
// }

// impl TryFrom<&str> for ShipInfoRole {
//     type Error = crate::Error;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "Construction" => Ok(ShipInfoRole::Construction),
//             "Trader" => Ok(ShipInfoRole::Trader),
//             "Contract" => Ok(ShipInfoRole::Contract),
//             "Scraper" => Ok(ShipInfoRole::Scraper),
//             "Mining" => Ok(ShipInfoRole::Mining),
//             "Charter" => Ok(ShipInfoRole::Charter),
//             "Manuel" => Ok(ShipInfoRole::Manuel),
//             "Transfer" => Ok(ShipInfoRole::Transfer),
//             _ => Err(crate::Error::InvalidShipInfoRole(value.to_string())),
//         }
//     }
// }

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
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn unassign_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
          UPDATE ship_info SET assignment_id = NULL WHERE symbol = $1
        "#,
            ship_symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn unassign_temp_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
          UPDATE ship_info SET temp_assignment_id = NULL WHERE symbol = $1
        "#,
            ship_symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }
}

impl DatabaseConnectorAsync for ShipInfo {
    type ID = String;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &super::DbPool, item: &ShipInfo) -> crate::Result<Self::ID> {
        sqlx::query!(
            r#"
              INSERT INTO public.ship_info(
                symbol,
                display_name,
                active,
                assignment_id,
                temp_assignment_id,
                purchase_id
                ) VALUES (
                 $1,
                 $2,
                 $3,
                 $4,
                 $5,
                 $6
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                active = EXCLUDED.active,
                assignment_id = EXCLUDED.assignment_id,
                temp_assignment_id = EXCLUDED.temp_assignment_id,
                purchase_id = EXCLUDED.purchase_id;
            "#,
            &item.symbol,
            &item.display_name,
            &item.active,
            &item.assignment_id as &Option<i64>,
            &item.temp_assignment_id as &Option<i64>,
            &item.purchase_id as &Option<i64>
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(item.symbol.clone())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &ShipInfo) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &ShipInfo) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[ShipInfo]) -> crate::Result<()> {
        let (symbol_s, display_name_s, active_s, assignment_id_s, purchase_id_s): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::Itertools::multiunzip(items.iter().cloned().map(|s| {
            (
                s.symbol.clone(),
                s.display_name.clone(),
                s.active,
                s.assignment_id,
                s.purchase_id,
            )
        }));

        sqlx::query!(
            r#"
              INSERT INTO public.ship_info (
                symbol,
                display_name,
                active,
                assignment_id,
                purchase_id
                )
                SELECT * FROM UNNEST(
                  $1::character varying[],
                  $2::character varying[],
                  $3::boolean[],
                  $4::bigint[],
                  $5::bigint[]
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                active = EXCLUDED.active,
                assignment_id = EXCLUDED.assignment_id,
                purchase_id = EXCLUDED.purchase_id;
            "#,
            &symbol_s as &[String],
            &display_name_s as &[String],
            &active_s as &[bool],
            &assignment_id_s as &[Option<i64>],
            &purchase_id_s as &[Option<i64>]
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipInfo>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as! {
                    ShipInfo,
                    r#"
                        SELECT 
                            symbol,
                            display_name,
                            active,
                            assignment_id,
                            temp_assignment_id,
                            purchase_id
                        FROM ship_info
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                }
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let items = sqlx::query_as! {
                    ShipInfo,
                    r#"
                        SELECT 
                            symbol,
                            display_name,
                            active,
                            assignment_id,
                            temp_assignment_id,
                            purchase_id
                        FROM ship_info
                    "#
                }
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ship_info
                    "#
                )
                .fetch_one(&database_pool.database_pool)
                .await?;

                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(database_pool: &super::DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            ShipInfo,
            r#"
        SELECT symbol, display_name, active, assignment_id, temp_assignment_id, purchase_id
        FROM ship_info WHERE symbol = $1
        LIMIT 1
      "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &super::DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM ship_info
            WHERE symbol = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
