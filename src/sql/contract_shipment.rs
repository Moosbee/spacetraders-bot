use space_traders_client::models;

use super::{sql_models::ShipmentStatus, DatabaseConnector};

impl DatabaseConnector<super::sql_models::ContractShipment>
    for super::sql_models::ContractShipment
{
    async fn insert(
        database_pool: &super::DbPool,
        item: &super::sql_models::ContractShipment,
    ) -> sqlx::Result<()> {
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
                    created_at,
                    updated_at,
                    status
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO UPDATE
                SET contract_id = EXCLUDED.contract_id,
                    ship_symbol = EXCLUDED.ship_symbol,
                    trade_symbol = EXCLUDED.trade_symbol,
                    units = EXCLUDED.units,
                    destination_symbol = EXCLUDED.destination_symbol,
                    purchase_symbol = EXCLUDED.purchase_symbol,
                    updated_at = EXCLUDED.updated_at,
                    status = EXCLUDED.status
            "#,
            item.id,
            item.contract_id,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.units,
            item.destination_symbol,
            item.purchase_symbol,
            item.created_at,
            item.updated_at,
            item.status as ShipmentStatus
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<super::sql_models::ContractShipment>,
    ) -> sqlx::Result<()> {
        let (
            ids,
            contract_ids,
            ship_symbols,
            trade_symbols,
            units,
            destination_symbols,
            purchase_symbols,
            created_ats,
            updated_ats,
            statuses,
        ): (
            Vec<i32>,
            Vec<String>,
            Vec<String>,
            Vec<models::TradeSymbol>,
            Vec<i32>,
            Vec<String>,
            Vec<String>,
            Vec<sqlx::types::chrono::NaiveDateTime>,
            Vec<sqlx::types::chrono::NaiveDateTime>,
            Vec<ShipmentStatus>,
        ) = itertools::multiunzip(items.iter().map(|c| {
            (
                c.id,
                c.contract_id.clone(),
                c.ship_symbol.clone(),
                c.trade_symbol,
                c.units,
                c.destination_symbol.clone(),
                c.purchase_symbol.clone(),
                c.created_at,
                c.updated_at,
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
                created_at,
                updated_at,
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
                $8::timestamp[],
                $9::timestamp[],
                $10::shipment_status[]
            )
            ON CONFLICT (id) DO UPDATE
            SET contract_id = EXCLUDED.contract_id,
                ship_symbol = EXCLUDED.ship_symbol,
                trade_symbol = EXCLUDED.trade_symbol,
                units = EXCLUDED.units,
                destination_symbol = EXCLUDED.destination_symbol,
                purchase_symbol = EXCLUDED.purchase_symbol,
                updated_at = EXCLUDED.updated_at,
                status = EXCLUDED.status
            "#,
            &ids,
            &contract_ids,
            &ship_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &units,
            &destination_symbols,
            &purchase_symbols,
            &created_ats,
            &updated_ats,
            &statuses as &[ShipmentStatus]
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(
        database_pool: &super::DbPool,
    ) -> sqlx::Result<Vec<super::sql_models::ContractShipment>> {
        sqlx::query_as!(
            super::sql_models::ContractShipment,
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
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl super::sql_models::ContractShipment {
    pub async fn insert_new(
        database_pool: &super::DbPool,
        item: &super::sql_models::ContractShipment,
    ) -> sqlx::Result<i32> {
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

    pub async fn get_by_contract_id(
        database_pool: &super::DbPool,
        contract_id: &str,
    ) -> sqlx::Result<Vec<super::sql_models::ContractShipment>> {
        sqlx::query_as!(
            super::sql_models::ContractShipment,
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
                "#,
            contract_id
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: i32,
    ) -> sqlx::Result<super::sql_models::ContractShipment> {
        sqlx::query_as!(
            super::sql_models::ContractShipment,
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
        .fetch_one(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_ship_symbol(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> sqlx::Result<Vec<super::sql_models::ContractShipment>> {
        sqlx::query_as!(
            super::sql_models::ContractShipment,
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
            "#,
            ship_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
