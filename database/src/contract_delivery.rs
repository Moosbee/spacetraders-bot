use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnector, DbPool};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, async_graphql::SimpleObject)]
#[graphql(name = "DBContractDelivery")]
pub struct ContractDelivery {
    pub contract_id: String,
    pub trade_symbol: models::TradeSymbol,
    pub destination_symbol: String,
    pub units_required: i32,
    pub units_fulfilled: i32,
}

impl ContractDelivery {
    pub fn from_contract_deliver_good(
        contract_delivery: models::contract_deliver_good::ContractDeliverGood,
        contract_id: &str,
    ) -> crate::Result<Self> {
        let trade_symbol = models::TradeSymbol::try_from(contract_delivery.trade_symbol.as_str())
            .map_err(|_| {
            crate::Error::InvalidTradeSymbol(contract_delivery.trade_symbol.clone())
        })?;

        Ok(ContractDelivery {
            contract_id: contract_id.to_string(),
            trade_symbol,
            destination_symbol: contract_delivery.destination_symbol,
            units_required: contract_delivery.units_required,
            units_fulfilled: contract_delivery.units_fulfilled,
        })
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_contract_id(
        database_pool: &DbPool,
        contract_id: &str,
    ) -> crate::Result<Vec<ContractDelivery>> {
        let erg = sqlx::query_as!(
            ContractDelivery,
            r#"
            SELECT 
              contract_id,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              destination_symbol,
              units_required,
              units_fulfilled
            FROM contract_delivery
            WHERE contract_id = $1
        "#,
            contract_id
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_trade_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Vec<ContractDelivery>> {
        let erg = sqlx::query_as!(
            ContractDelivery,
            r#"
            SELECT 
              contract_id,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              destination_symbol,
              units_required,
              units_fulfilled
            FROM contract_delivery
            WHERE trade_symbol = $1
        "#,
            *trade_symbol as models::TradeSymbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_destination_symbol(
        database_pool: &DbPool,
        destination_symbol: &str,
    ) -> crate::Result<Vec<ContractDelivery>> {
        let erg = sqlx::query_as!(
            ContractDelivery,
            r#"
            SELECT 
              contract_id,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              destination_symbol,
              units_required,
              units_fulfilled
            FROM contract_delivery
            WHERE destination_symbol = $1
        "#,
            destination_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_system_symbol(
        database_pool: &DbPool,
        system_symbol: &str,
    ) -> crate::Result<Vec<ContractDelivery>> {
        let erg = sqlx::query_as!(
            ContractDelivery,
            r#"
            SELECT 
              contract_id,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              destination_symbol,
              units_required,
              units_fulfilled
            FROM contract_delivery JOIN waypoint ON contract_delivery.destination_symbol = waypoint.symbol
            WHERE waypoint.system_symbol = $1
        "#,
            system_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<ContractDelivery> for ContractDelivery {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert(database_pool: &DbPool, item: &ContractDelivery) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO contract_delivery (contract_id, trade_symbol, destination_symbol, units_required, units_fulfilled)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (contract_id, trade_symbol, destination_symbol) DO UPDATE
                SET units_required = EXCLUDED.units_required,
                units_fulfilled = EXCLUDED.units_fulfilled
            "#,
            item.contract_id,
            item.trade_symbol as models::TradeSymbol,
            item.destination_symbol,
            item.units_required,
            item.units_fulfilled
        ).execute(&database_pool.database_pool).await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ContractDelivery]) -> crate::Result<()> {
        let (contract_ids, trade_symbols, units_fulfilled, units_required, destination_symbols): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|c| {
            (
                c.contract_id.clone(),
                c.trade_symbol,
                c.units_fulfilled,
                c.units_required,
                c.destination_symbol.clone(),
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO contract_delivery (
              contract_id,
              trade_symbol,
              destination_symbol,
              units_required,
              units_fulfilled
            )
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::trade_symbol[],
                $3::character varying[],
                $4::integer[],
                $5::integer[]
            )
            ON CONFLICT (contract_id, trade_symbol, destination_symbol) DO UPDATE
            SET units_fulfilled = EXCLUDED.units_fulfilled
        "#,
            &contract_ids,
            &trade_symbols as &[models::TradeSymbol],
            &destination_symbols,
            &units_required,
            &units_fulfilled
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<ContractDelivery>> {
        let erg = sqlx::query_as!(
            ContractDelivery,
            r#"
            SELECT 
              contract_id,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              destination_symbol,
              units_required,
              units_fulfilled
            FROM contract_delivery
        "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
