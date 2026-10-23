use tracing::instrument;

use crate::{DatabaseConnectorAsync, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBShipPurchaseReservation")]
pub struct ShipPurchaseReservation {
    pub id: i64,
    pub shipyard_ship_id: i64,
    pub assignment_id: i64,
    pub fleet_id: i32,
    pub reservation_id: i64,

    pub state: ShipPurchaseReservationState,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    sqlx::Type,
    serde::Serialize,
    serde::Deserialize,
    Default,
    async_graphql::Enum,
)]
#[sqlx(type_name = "ship_purchase_reservation_state")]
pub enum ShipPurchaseReservationState {
    #[default]
    #[sqlx(rename = "RESERVED")]
    Reserved,
    #[sqlx(rename = "USED")]
    Used,
    #[sqlx(rename = "CANCELLED")]
    Cancelled,
}

impl DatabaseConnectorAsync for ShipPurchaseReservation {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipPurchaseReservation,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO ship_purchase_reservations (
                    shipyard_ship_id,
                    assignment_id,
                    fleet_id,
                    reservation_id,
                    state
                )
                VALUES ($1, $2, $3, $4, $5::ship_purchase_reservation_state)
                RETURNING id
            "#,
            item.shipyard_ship_id,
            item.assignment_id,
            item.fleet_id,
            item.reservation_id,
            &item.state as &ShipPurchaseReservationState
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(
        database_pool: &super::DbPool,
        item: &ShipPurchaseReservation,
    ) -> crate::Result<()> {
        if item.id == 0 {
            let _ = ShipPurchaseReservation::insert_new(database_pool, item).await?;
            return Ok(());
        }

        sqlx::query!(
            r#"
                INSERT INTO ship_purchase_reservations (
                    id,
                    shipyard_ship_id,
                    assignment_id,
                    fleet_id,
                    reservation_id,
                    state
                )
                VALUES ($1, $2, $3, $4, $5, $6::ship_purchase_reservation_state)
                ON CONFLICT (id) DO UPDATE SET
                    shipyard_ship_id = EXCLUDED.shipyard_ship_id,
                    assignment_id = EXCLUDED.assignment_id,
                    fleet_id = EXCLUDED.fleet_id,
                    reservation_id = EXCLUDED.reservation_id,
                    state = EXCLUDED.state,
                    updated_at = NOW()
            "#,
            item.id,
            item.shipyard_ship_id,
            item.assignment_id,
            item.fleet_id,
            item.reservation_id,
            &item.state as &ShipPurchaseReservationState
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(
        database_pool: &super::DbPool,
        item: &ShipPurchaseReservation,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE ship_purchase_reservations
                SET
                    shipyard_ship_id = $1,
                    assignment_id = $2,
                    fleet_id = $3,
                    reservation_id = $4,
                    state = $5::ship_purchase_reservation_state,
                    updated_at = NOW()
                WHERE id = $6
            "#,
            item.shipyard_ship_id,
            item.assignment_id,
            item.fleet_id,
            item.reservation_id,
            &item.state as &ShipPurchaseReservationState,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipPurchaseReservation],
    ) -> crate::Result<()> {
        let (shipyard_ship_ids, assignment_ids, fleet_ids, reservation_ids, states): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|r| {
            (
                r.shipyard_ship_id,
                r.assignment_id,
                r.fleet_id,
                r.reservation_id,
                r.state,
            )
        }));

        sqlx::query!(
            r#"
                INSERT INTO ship_purchase_reservations (
                    shipyard_ship_id,
                    assignment_id,
                    fleet_id,
                    reservation_id,
                    state
                )
                SELECT * FROM UNNEST(
                    $1::bigint[],
                    $2::bigint[],
                    $3::integer[],
                    $4::bigint[],
                    $5::ship_purchase_reservation_state[]
                )
            "#,
            &shipyard_ship_ids,
            &assignment_ids,
            &fleet_ids,
            &reservation_ids,
            &states as &[ShipPurchaseReservationState]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipPurchaseReservation>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipPurchaseReservation,
                    r#"
                        SELECT
                            id,
                            shipyard_ship_id,
                            assignment_id,
                            fleet_id,
                            reservation_id,
                            state as "state: ShipPurchaseReservationState",
                            created_at,
                            updated_at
                        FROM ship_purchase_reservations
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
                    ShipPurchaseReservation,
                    r#"
                        SELECT
                            id,
                            shipyard_ship_id,
                            assignment_id,
                            fleet_id,
                            reservation_id,
                            state as "state: ShipPurchaseReservationState",
                            created_at,
                            updated_at
                        FROM ship_purchase_reservations
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
                        FROM ship_purchase_reservations
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
            ShipPurchaseReservation,
            r#"
                SELECT
                    id,
                    shipyard_ship_id,
                    assignment_id,
                    fleet_id,
                    reservation_id,
                    state as "state: ShipPurchaseReservationState",
                    created_at,
                    updated_at
                FROM ship_purchase_reservations
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
                DELETE FROM ship_purchase_reservations
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
