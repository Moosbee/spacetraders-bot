use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

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
#[sqlx(type_name = "shipment_status")]
pub enum ShipmentStatus {
    #[default]
    #[sqlx(rename = "IN_TRANSIT")]
    InTransit,
    #[sqlx(rename = "FAILED")]
    Failed,
    #[sqlx(rename = "DELIVERED")]
    Delivered,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBContractShipment")]
pub struct ContractShipment {
    pub id: i32,
    pub contract_id: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
    pub destination_symbol: String,
    pub purchase_symbol: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub status: ShipmentStatus,
}

impl Default for ContractShipment {
    fn default() -> Self {
        Self {
            id: Default::default(),
            contract_id: Default::default(),
            ship_symbol: Default::default(),
            trade_symbol: Default::default(),
            units: Default::default(),
            destination_symbol: Default::default(),
            purchase_symbol: Default::default(),
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
            updated_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
            status: Default::default(),
        }
    }
}

impl DatabaseConnectorAsync for ContractShipment {
    type ID = i32;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &DbPool,
        item: &ContractShipment,
    ) -> crate::Result<Self::ID> {
        let id = sqlx::query!(
            r#"
                INSERT INTO contract_shipment (
                    contract_id,
                    ship_symbol,
                    trade_symbol,
                    units,
                    destination_symbol,
                    purchase_symbol,
                    status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            "#,
            item.contract_id,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.units,
            item.destination_symbol,
            item.purchase_symbol,
            item.status as ShipmentStatus
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(id.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ContractShipment) -> crate::Result<()> {
        if item.id == 0 {
            let _ = Self::insert_new(database_pool, item).await?;
            return Ok(());
        }

        sqlx::query!(
            r#"
                INSERT INTO contract_shipment (
                    id,
                    contract_id,
                    ship_symbol,
                    trade_symbol,
                    units,
                    destination_symbol,
                    purchase_symbol,
                    status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE
                SET contract_id = EXCLUDED.contract_id,
                    ship_symbol = EXCLUDED.ship_symbol,
                    trade_symbol = EXCLUDED.trade_symbol,
                    units = EXCLUDED.units,
                    destination_symbol = EXCLUDED.destination_symbol,
                    purchase_symbol = EXCLUDED.purchase_symbol,
                    updated_at = now(),
                    status = EXCLUDED.status
            "#,
            item.id,
            item.contract_id,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.units,
            item.destination_symbol,
            item.purchase_symbol,
            item.status as ShipmentStatus
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ContractShipment) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE contract_shipment
                SET
                    contract_id = $1,
                    ship_symbol = $2,
                    trade_symbol = $3,
                    units = $4,
                    destination_symbol = $5,
                    purchase_symbol = $6,
                    updated_at = now(),
                    status = $7
                WHERE id = $8
            "#,
            item.contract_id,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.units,
            item.destination_symbol,
            item.purchase_symbol,
            item.status as ShipmentStatus,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ContractShipment]) -> crate::Result<()> {
        let (
            ids,
            contract_ids,
            ship_symbols,
            trade_symbols,
            units,
            destination_symbols,
            purchase_symbols,
            statuses,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|c| {
            (
                c.id,
                c.contract_id.clone(),
                c.ship_symbol.clone(),
                c.trade_symbol,
                c.units,
                c.destination_symbol.clone(),
                c.purchase_symbol.clone(),
                c.status,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO contract_shipment (
                id,
                contract_id,
                ship_symbol,
                trade_symbol,
                units,
                destination_symbol,
                purchase_symbol,
                status
            )
            SELECT * FROM UNNEST(
                $1::integer[],
                $2::character varying[],
                $3::character varying[],
                $4::trade_symbol[],
                $5::integer[],
                $6::character varying[],
                $7::character varying[],
                $8::shipment_status[]
            )
            ON CONFLICT (id) DO UPDATE
            SET contract_id = EXCLUDED.contract_id,
                ship_symbol = EXCLUDED.ship_symbol,
                trade_symbol = EXCLUDED.trade_symbol,
                units = EXCLUDED.units,
                destination_symbol = EXCLUDED.destination_symbol,
                purchase_symbol = EXCLUDED.purchase_symbol,
                updated_at = now(),
                status = EXCLUDED.status
            "#,
            &ids,
            &contract_ids,
            &ship_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &units,
            &destination_symbols,
            &purchase_symbols,
            &statuses as &[ShipmentStatus]
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        ORDER BY created_at DESC, id DESC
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
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        ORDER BY created_at DESC, id DESC
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
                        FROM contract_shipment
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
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            ContractShipment,
            r#"
                SELECT 
                    id,
                    contract_id,
                    ship_symbol,
                    trade_symbol as "trade_symbol: models::TradeSymbol",
                    units,
                    destination_symbol,
                    purchase_symbol,
                    created_at,
                    updated_at,
                    status as "status: ShipmentStatus"
                FROM contract_shipment
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
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM contract_shipment
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

impl ContractShipment {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_contract_id(
        database_pool: &DbPool,
        contract_id: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE contract_id = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    contract_id,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE contract_id = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    contract_id
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE contract_id = $1
                    "#,
                    contract_id
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_id(database_pool: &DbPool, id: i32) -> crate::Result<ContractShipment> {
        let erg = sqlx::query_as!(
            ContractShipment,
            r#"
            SELECT 
                id,
                contract_id,
                ship_symbol,
                trade_symbol as "trade_symbol: models::TradeSymbol",
                units,
                destination_symbol,
                purchase_symbol,
                created_at,
                updated_at,
                status as "status: ShipmentStatus"
            FROM contract_shipment
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE ship_symbol = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    ship_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE ship_symbol = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    ship_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE ship_symbol = $1
                    "#,
                    ship_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_trade_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE trade_symbol = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    *trade_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE trade_symbol = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE trade_symbol = $1
                    "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_destination_symbol(
        database_pool: &DbPool,
        destination_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE destination_symbol = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    destination_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE destination_symbol = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    destination_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE destination_symbol = $1
                    "#,
                    destination_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_source_symbol(
        database_pool: &DbPool,
        source_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE purchase_symbol = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    source_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE purchase_symbol = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    source_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE purchase_symbol = $1
                    "#,
                    source_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_contract_id_trade_symbol_destination_symbol(
        database_pool: &DbPool,
        contract_id: &str,
        trade_symbol: &models::TradeSymbol,
        destination_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ContractShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE contract_id = $1 AND trade_symbol = $2 AND destination_symbol = $3
                        ORDER BY created_at DESC, id DESC
                        LIMIT $4 OFFSET $5
                    "#,
                    contract_id,
                    *trade_symbol as models::TradeSymbol,
                    destination_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ContractShipment,
                    r#"
                        SELECT 
                            id,
                            contract_id,
                            ship_symbol,
                            trade_symbol as "trade_symbol: models::TradeSymbol",
                            units,
                            destination_symbol,
                            purchase_symbol,
                            created_at,
                            updated_at,
                            status as "status: ShipmentStatus"
                        FROM contract_shipment
                        WHERE contract_id = $1 AND trade_symbol = $2 AND destination_symbol = $3
                        ORDER BY created_at DESC, id DESC
                    "#,
                    contract_id,
                    *trade_symbol as models::TradeSymbol,
                    destination_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM contract_shipment
                        WHERE contract_id = $1 AND trade_symbol = $2 AND destination_symbol = $3
                    "#,
                    contract_id,
                    *trade_symbol as models::TradeSymbol,
                    destination_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}
