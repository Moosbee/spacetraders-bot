use space_traders_client::models;

use super::sql_models::DatabaseConnector;

impl super::sql_models::ContractDelivery {
    pub fn from_contract_deliver_good(
        contract_delivery: models::contract_deliver_good::ContractDeliverGood,
        contract_id: &str,
    ) -> Result<Self, anyhow::Error> {
        let trade_symbol =
            models::TradeSymbol::try_from(contract_delivery.trade_symbol.as_str())?;

        Ok(super::sql_models::ContractDelivery {
            contract_id: contract_id.to_string(),
            trade_symbol,
            destination_symbol: contract_delivery.destination_symbol,
            units_required: contract_delivery.units_required,
            units_fulfilled: contract_delivery.units_fulfilled,
        })
    }
}

impl DatabaseConnector<super::sql_models::ContractDelivery>
    for super::sql_models::ContractDelivery
{
    async fn insert(
        database_pool: &sqlx::PgPool,
        item: &super::sql_models::ContractDelivery,
    ) -> sqlx::Result<()> {
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
        ).execute(database_pool).await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &sqlx::PgPool,
        items: &Vec<super::sql_models::ContractDelivery>,
    ) -> sqlx::Result<()> {
        let (
            ((contract_ids, trade_symbols), (units_fulfilled, units_required)),
            ((destination_symbols, _), (_, _)),
        ): (
            (
                (Vec<String>, Vec<models::TradeSymbol>),
                (Vec<i32>, Vec<i32>),
            ),
            ((Vec<String>, Vec<()>), (Vec<()>, Vec<()>)),
        ) = items
            .iter()
            .map(|c| {
                (
                    (
                        (c.contract_id.clone(), c.trade_symbol),
                        (c.units_fulfilled, c.units_required),
                    ),
                    ((c.destination_symbol.clone(), ()), ((), ())),
                )
            })
            .unzip();

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
        .execute(database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(
        database_pool: &sqlx::PgPool,
    ) -> sqlx::Result<Vec<super::sql_models::ContractDelivery>> {
        sqlx::query_as!(
            super::sql_models::ContractDelivery,
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
        .fetch_all(database_pool)
        .await
    }
}
