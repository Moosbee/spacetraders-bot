use space_traders_client::{ShipSymbol, SystemSymbol};
use tracing::instrument;

use crate::{DatabaseConnectorAsync, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Default, Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBShipTransfer")]
pub struct ShipTransfer {
    pub id: i64,
    pub ship_symbol: ShipSymbol,

    pub state: ShipTransferState,

    pub fleet_id: i32,
    pub assignment_id: i64,
    pub start_system_symbol: SystemSymbol,
    pub end_system_symbol: SystemSymbol,
    pub reservation_id: i64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, async_graphql::Enum, sqlx::Type)]
#[sqlx(type_name = "ship_transfer_state")]
pub enum ShipTransferState {
    #[graphql(name = "ON_GOING")]
    #[sqlx(rename = "ON_GOING")]
    #[default]
    OnGoing,
    #[graphql(name = "COMPLETED")]
    #[sqlx(rename = "COMPLETED")]
    Completed,
}

impl DatabaseConnectorAsync for ShipTransfer {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipTransfer,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO ship_transfers (
                    ship_symbol,
                    state,
                    fleet_id,
                    assignment_id,
                    start_system_symbol,
                    end_system_symbol,
                    reservation_id
                )
                VALUES ($1, $2::ship_transfer_state, $3, $4, $5, $6, $7)
                RETURNING id
            "#,
            item.ship_symbol.as_str(),
            &item.state as &ShipTransferState,
            item.fleet_id,
            item.assignment_id,
            item.start_system_symbol.as_str(),
            item.end_system_symbol.as_str(),
            item.reservation_id
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &ShipTransfer) -> crate::Result<()> {
        if item.id == 0 {
            let _ = ShipTransfer::insert_new(database_pool, item).await?;
            return Ok(());
        }

        sqlx::query!(
            r#"
                INSERT INTO ship_transfers (
                    id,
                    ship_symbol,
                    state,
                    fleet_id,
                    assignment_id,
                    start_system_symbol,
                    end_system_symbol,
                    reservation_id
                )
                VALUES ($1, $2, $3::ship_transfer_state, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE SET
                    ship_symbol = EXCLUDED.ship_symbol,
                    state = EXCLUDED.state,
                    fleet_id = EXCLUDED.fleet_id,
                    assignment_id = EXCLUDED.assignment_id,
                    start_system_symbol = EXCLUDED.start_system_symbol,
                    end_system_symbol = EXCLUDED.end_system_symbol,
                    reservation_id = EXCLUDED.reservation_id,
                    updated_at = NOW()
            "#,
            item.id,
            item.ship_symbol.as_str(),
            &item.state as &ShipTransferState,
            item.fleet_id,
            item.assignment_id,
            item.start_system_symbol.as_str(),
            item.end_system_symbol.as_str(),
            item.reservation_id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &ShipTransfer) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE ship_transfers
                SET
                    ship_symbol = $1,
                    state = $2::ship_transfer_state,
                    fleet_id = $3,
                    assignment_id = $4,
                    start_system_symbol = $5,
                    end_system_symbol = $6,
                    reservation_id = $7,
                    updated_at = NOW()
                WHERE id = $8
            "#,
            item.ship_symbol.as_str(),
            &item.state as &ShipTransferState,
            item.fleet_id,
            item.assignment_id,
            item.start_system_symbol.as_str(),
            item.end_system_symbol.as_str(),
            item.reservation_id,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipTransfer],
    ) -> crate::Result<()> {
        let (
            ship_symbols,
            states,
            fleet_ids,
            assignment_ids,
            start_system_symbols,
            end_system_symbols,
            reservation_ids,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(items.iter().map(|t| {
                (
                    t.ship_symbol.clone().into_inner(),
                    t.state,
                    t.fleet_id,
                    t.assignment_id,
                    t.start_system_symbol.clone().into_inner(),
                    t.end_system_symbol.clone().into_inner(),
                    t.reservation_id,
                )
            }));

        sqlx::query!(
            r#"
                INSERT INTO ship_transfers (
                    ship_symbol,
                    state,
                    fleet_id,
                    assignment_id,
                    start_system_symbol,
                    end_system_symbol,
                    reservation_id
                )
                SELECT * FROM UNNEST(
                    $1::character varying[],
                    $2::ship_transfer_state[],
                    $3::integer[],
                    $4::bigint[],
                    $5::character varying[],
                    $6::character varying[],
                    $7::bigint[]
                )
            "#,
            &ship_symbols,
            &states as &[ShipTransferState],
            &fleet_ids,
            &assignment_ids,
            &start_system_symbols,
            &end_system_symbols,
            &reservation_ids
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipTransfer>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipTransfer,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            state as "state: ShipTransferState",
                            fleet_id,
                            assignment_id,
                            start_system_symbol,
                            end_system_symbol,
                            reservation_id,
                            created_at,
                            updated_at
                        FROM ship_transfers
                        ORDER BY id ASC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipTransfer,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            state as "state: ShipTransferState",
                            fleet_id,
                            assignment_id,
                            start_system_symbol,
                            end_system_symbol,
                            reservation_id,
                            created_at,
                            updated_at
                        FROM ship_transfers
                        ORDER BY id ASC
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_transfers
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            ShipTransfer,
            r#"
                SELECT
                    id,
                    ship_symbol,
                    state as "state: ShipTransferState",
                    fleet_id,
                    assignment_id,
                    start_system_symbol,
                    end_system_symbol,
                    reservation_id,
                    created_at,
                    updated_at
                FROM ship_transfers
                WHERE id = $1
                LIMIT 1
            "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &super::DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM ship_transfers
                WHERE id = $1
            "#,
            *id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
