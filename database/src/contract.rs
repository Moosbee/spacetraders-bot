use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{ContractDelivery, DatabaseConnector, DbPool};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, async_graphql::SimpleObject)]
pub struct Contract {
    pub id: String,
    pub faction_symbol: String,
    pub contract_type: models::contract::Type,
    pub accepted: bool,
    pub fulfilled: bool,
    pub deadline_to_accept: Option<String>,
    pub on_accepted: i32,
    pub on_fulfilled: i32,
    pub deadline: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub reserved_fund: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractSummary {
    pub id: String,
    pub faction_symbol: String,
    pub contract_type: models::contract::Type,
    pub accepted: bool,
    pub fulfilled: bool,
    pub deadline_to_accept: Option<String>,
    pub on_accepted: i32,
    pub on_fulfilled: i32,
    pub deadline: String,
    pub totalprofit: Option<i32>,
    pub total_expenses: Option<i32>,
    pub net_profit: Option<i32>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub reserved_fund: Option<i64>,
}

impl From<models::Contract> for Contract {
    fn from(value: models::Contract) -> Self {
        Contract {
            id: value.id,
            faction_symbol: value.faction_symbol,
            contract_type: value.r#type as models::contract::Type,
            accepted: value.accepted,
            fulfilled: value.fulfilled,
            deadline_to_accept: value.deadline_to_accept,
            on_accepted: value.terms.payment.on_accepted,
            on_fulfilled: value.terms.payment.on_fulfilled,
            deadline: value.terms.deadline,
            updated_at: Utc::now(),
            created_at: Utc::now(),
            reserved_fund: None,
        }
    }
}

impl DatabaseConnector<Contract> for Contract {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &Contract) -> crate::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO contract (
              id,
              faction_symbol,
              contract_type,
              accepted,
              fulfilled,
              deadline_to_accept,
              on_accepted,
              on_fulfilled,
              deadline,
              reserved_fund
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET 
              faction_symbol = EXCLUDED.faction_symbol,
              contract_type = EXCLUDED.contract_type,
              accepted = EXCLUDED.accepted,
              fulfilled = EXCLUDED.fulfilled,
              deadline_to_accept = EXCLUDED.deadline_to_accept,
              on_accepted = EXCLUDED.on_accepted,
              on_fulfilled = EXCLUDED.on_fulfilled,
              deadline = EXCLUDED.deadline,
              updated_at = EXCLUDED.updated_at,
              reserved_fund = EXCLUDED.reserved_fund
        "#,
            item.id,
            item.faction_symbol,
            item.contract_type as models::contract::Type,
            item.accepted,
            item.fulfilled,
            item.deadline_to_accept,
            item.on_accepted,
            item.on_fulfilled,
            item.deadline,
            item.reserved_fund
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Contract]) -> crate::Result<()> {
        let (
            ids,
            contract_types,
            faction_symbols,
            accepteds,
            fulfilleds,
            deadlines_to_accept,
            deadlines,
            on_accepteds,
            on_fulfilleds,
            reserved_funds,
        ): (
            Vec<_>,
            Vec<_>,
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
                c.id.clone(),
                c.contract_type,
                c.faction_symbol.clone(),
                c.accepted,
                c.fulfilled,
                c.deadline_to_accept.clone(),
                c.deadline.clone(),
                c.on_accepted,
                c.on_fulfilled,
                c.reserved_fund,
            )
        }));

        let _insert = sqlx::query!(
            r#"
            INSERT INTO contract (
              id,
              contract_type,
              faction_symbol,
              accepted,
              fulfilled,
              deadline_to_accept,
              on_accepted,
              on_fulfilled,
              deadline,
              reserved_fund
            )
            SELECT * FROM UNNEST(
              $1::character varying[],
              $2::contract_type[],
              $3::character varying[],
              $4::boolean[],
              $5::boolean[],
              $6::character varying[],
              $7::integer[],
              $8::integer[],
              $9::character varying[],
              $10::bigint[]
            )
            ON CONFLICT (id) DO UPDATE SET 
              contract_type = EXCLUDED.contract_type,
              faction_symbol = EXCLUDED.faction_symbol,
              accepted = EXCLUDED.accepted,
              fulfilled = EXCLUDED.fulfilled,
              deadline_to_accept = EXCLUDED.deadline_to_accept,
              on_accepted = EXCLUDED.on_accepted,
              on_fulfilled = EXCLUDED.on_fulfilled,
              deadline = EXCLUDED.deadline,
              updated_at = EXCLUDED.updated_at,
              reserved_fund = EXCLUDED.reserved_fund
        "#,
            &ids as &[String],
            &contract_types as &[models::contract::Type],
            &faction_symbols as &[String],
            &accepteds as &[bool],
            &fulfilleds as &[bool],
            &deadlines_to_accept as &[Option<String>],
            &on_accepteds as &[i32],
            &on_fulfilleds as &[i32],
            &deadlines as &[String],
            &reserved_funds as &[Option<i64>],
        )
        .execute(&database_pool.database_pool)
        .await;

        Ok(())
    }
    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<Contract>> {
        let erg = sqlx::query_as!(
            Contract,
            r#"
                SELECT 
                    id,
                    faction_symbol,
                    contract_type as "contract_type: models::contract::Type",
                    accepted,
                    fulfilled,
                    deadline_to_accept,
                    on_accepted,
                    on_fulfilled,
                    deadline,
                    updated_at,
                    created_at,
                    reserved_fund
                FROM contract
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl Contract {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn insert_contract(
        database_pool: &DbPool,
        contract: models::Contract,
        reserved_fund: Option<i64>,
    ) -> crate::Result<()> {
        let mut contract_old = Contract::from(contract.clone());
        contract_old.reserved_fund = reserved_fund;
        Contract::insert(database_pool, &contract_old).await?;

        if let Some(deliveries) = &contract.terms.deliver {
            let deliveries: Vec<_> = deliveries
                .iter()
                .filter_map(|c| {
                    ContractDelivery::from_contract_deliver_good(c.clone(), &contract_old.id).ok()
                })
                .collect();
            ContractDelivery::insert_bulk(database_pool, &deliveries).await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_faction_symbol(
        database_pool: &DbPool,
        symbol: &String,
    ) -> crate::Result<Vec<Contract>> {
        let erg = sqlx::query_as!(
            Contract,
            r#"SELECT
          id,
          faction_symbol,
          contract_type as "contract_type: models::contract::Type",
          accepted,
          fulfilled,
          deadline_to_accept,
          on_accepted,
          on_fulfilled,
          deadline,
          updated_at,
          created_at,
          reserved_fund
        FROM public.contract WHERE faction_symbol = $1"#,
            &symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_id(database_pool: &DbPool, id: &String) -> crate::Result<Option<Contract>> {
        let erg = sqlx::query_as!(
            Contract,
            r#"SELECT
          id,
          faction_symbol,
          contract_type as "contract_type: models::contract::Type",
          accepted,
          fulfilled,
          deadline_to_accept,
          on_accepted,
          on_fulfilled,
          deadline,
          updated_at,
          created_at,
          reserved_fund
        FROM public.contract WHERE id = $1"#,
            &id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_all_sm(database_pool: &DbPool) -> crate::Result<Vec<ContractSummary>> {
        let erg = sqlx::query_as!(
    ContractSummary,
    r#"
SELECT
  contract.id,
  contract.faction_symbol,
  contract.contract_type as "contract_type: models::contract::Type",
  contract.accepted,
  contract.fulfilled,
  contract.deadline_to_accept,
  contract.on_accepted,
  contract.on_fulfilled,
  contract.deadline,
  contract.on_accepted + contract.on_fulfilled as "totalprofit: i32",
  COALESCE(sum(market_transaction.total_price), 0) as "total_expenses: i32",
  contract.on_accepted + contract.on_fulfilled - COALESCE(sum(market_transaction.total_price), 0) as "net_profit: i32",
  contract.updated_at,
  contract.created_at,
  contract.reserved_fund
FROM
  public.contract
 left join public.market_transaction ON market_transaction.contract = contract.id
group by
  contract.id
order by
  contract.deadline_to_accept ASC;
    "#,
)
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn update_reserved_fund(
        database_pool: &DbPool,
        contract_id: &String,
        reserved_fund: Option<i64>,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
            UPDATE contract
            SET reserved_fund = $1
            WHERE id = $2
        "#,
            reserved_fund,
            contract_id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }
}
