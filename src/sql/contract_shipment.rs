use space_traders_client::models;

use super::DatabaseConnector;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize, Default,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContractShipment {
    pub id: i32,
    pub contract_id: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
    pub destination_symbol: String,
    pub purchase_symbol: String,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
    pub updated_at: sqlx::types::chrono::NaiveDateTime,
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
            created_at: sqlx::types::chrono::NaiveDateTime::MIN,
            updated_at: sqlx::types::chrono::NaiveDateTime::MIN,
            status: Default::default(),
        }
    }
}

impl DatabaseConnector<ContractShipment> for ContractShipment {
    async fn insert(database_pool: &super::DbPool, item: &ContractShipment) -> sqlx::Result<()> {
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
        items: &Vec<ContractShipment>,
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

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ContractShipment>> {
        sqlx::query_as!(
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
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl ContractShipment {
    pub async fn insert_new(
        database_pool: &super::DbPool,
        item: &ContractShipment,
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
    ) -> sqlx::Result<Vec<ContractShipment>> {
        sqlx::query_as!(
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
                "#,
            contract_id
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: i32,
    ) -> sqlx::Result<ContractShipment> {
        sqlx::query_as!(
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
        .fetch_one(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_ship_symbol(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> sqlx::Result<Vec<ContractShipment>> {
        sqlx::query_as!(
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
            "#,
            ship_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
