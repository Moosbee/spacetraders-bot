use tracing::instrument;

use super::{DatabaseConnectorAsync, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipTransferRequestSQL")]
pub struct ShipTransferRequest {
    pub id: i32,
    pub ship_symbol: String,
    pub reserved_fund: i64,
    pub finished: bool,
    pub fleet_id: i32,
    pub assignment_id: i64,
}

impl ShipTransferRequest {
    pub async fn get_active_by_ship_fleet_and_assignment(
        database_pool: &super::DbPool,
        ship_symbol: &str,
        fleet_id: i32,
        assignment_id: i64,
    ) -> crate::Result<Vec<ShipTransferRequest>> {
        let items = sqlx::query_as!(
            ShipTransferRequest,
            r#"
                SELECT
                    id,
                    ship_symbol,
                    reserved_fund,
                    finished,
                    fleet_id,
                    assignment_id
                FROM ship_transfer_request
                WHERE ship_symbol = $1 AND fleet_id = $2 AND assignment_id = $3
            "#,
            ship_symbol,
            fleet_id,
            assignment_id,
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(items)
    }
}

impl DatabaseConnectorAsync for ShipTransferRequest {
    type ID = i32;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipTransferRequest,
    ) -> crate::Result<Self::ID> {
        let id = sqlx::query!(
            r#"
                INSERT INTO public.ship_transfer_request (
                    ship_symbol,
                    reserved_fund,
                    finished,
                    fleet_id,
                    assignment_id
                ) VALUES (
                    $1, $2, $3, $4, $5
                )
                RETURNING id
            "#,
            &item.ship_symbol,
            &item.reserved_fund,
            &item.finished,
            &item.fleet_id,
            &item.assignment_id,
        )
        .fetch_one(&database_pool.database_pool)
        .await?
        .id;

        Ok(id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(
        database_pool: &super::DbPool,
        item: &ShipTransferRequest,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO public.ship_transfer_request (
                    id,
                    ship_symbol,
                    reserved_fund,
                    finished,
                    fleet_id,
                    assignment_id
                ) VALUES (
                    $1, $2, $3, $4, $5, $6
                )
                ON CONFLICT (id) DO UPDATE SET
                    ship_symbol = EXCLUDED.ship_symbol,
                    reserved_fund = EXCLUDED.reserved_fund,
                    finished = EXCLUDED.finished,
                    fleet_id = EXCLUDED.fleet_id,
                    assignment_id = EXCLUDED.assignment_id
            "#,
            &item.id,
            &item.ship_symbol,
            &item.reserved_fund,
            &item.finished,
            &item.fleet_id,
            &item.assignment_id,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(
        database_pool: &super::DbPool,
        item: &ShipTransferRequest,
    ) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipTransferRequest],
    ) -> crate::Result<()> {
        let (ship_symbol_s, reserved_fund_s, finished_s, fleet_id_s, assignment_id_s): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::Itertools::multiunzip(items.iter().map(|s| {
            (
                s.ship_symbol.clone(),
                s.reserved_fund,
                s.finished,
                s.fleet_id,
                s.assignment_id,
            )
        }));

        sqlx::query!(
            r#"
                INSERT INTO public.ship_transfer_request (
                    ship_symbol,
                    reserved_fund,
                    finished,
                    fleet_id,
                    assignment_id
                )
                SELECT * FROM UNNEST(
                    $1::character varying[],
                    $2::bigint[],
                    $3::boolean[],
                    $4::bigint[],
                    $5::bigint[]
                )
            "#,
            &ship_symbol_s as &[String],
            &reserved_fund_s as &[i64],
            &finished_s as &[bool],
            &fleet_id_s as &[i32],
            &assignment_id_s as &[i64],
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipTransferRequest>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipTransferRequest,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            reserved_fund,
                            finished,
                            fleet_id,
                            assignment_id
                        FROM ship_transfer_request
                        ORDER BY id ASC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipTransferRequest,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            reserved_fund,
                            finished,
                            fleet_id,
                            assignment_id
                        FROM ship_transfer_request
                        ORDER BY id ASC
                    "#
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_transfer_request
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
    async fn get_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            ShipTransferRequest,
            r#"
                SELECT id, ship_symbol, reserved_fund, finished, fleet_id, assignment_id
                FROM ship_transfer_request
                WHERE id = $1
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
                DELETE FROM ship_transfer_request
                WHERE id = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
