use space_traders_client::models;

use crate::sql::sql_models::ContractSummary;

use super::{
    sql_models::{Contract, ContractDelivery, DatabaseConnector},
    DbPool,
};

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
        }
    }
}

impl DatabaseConnector<Contract> for Contract {
    async fn insert(database_pool: &DbPool, item: &Contract) -> sqlx::Result<()> {
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
              deadline
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET 
              faction_symbol = EXCLUDED.faction_symbol,
              contract_type = EXCLUDED.contract_type,
              accepted = EXCLUDED.accepted,
              fulfilled = EXCLUDED.fulfilled,
              deadline_to_accept = EXCLUDED.deadline_to_accept,
              on_accepted = EXCLUDED.on_accepted,
              on_fulfilled = EXCLUDED.on_fulfilled,
              deadline = EXCLUDED.deadline
        "#,
            item.id,
            item.faction_symbol,
            item.contract_type as models::contract::Type,
            item.accepted,
            item.fulfilled,
            item.deadline_to_accept,
            item.on_accepted,
            item.on_fulfilled,
            item.deadline
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<Contract>) -> sqlx::Result<()> {
        let (
            ((id_and_contract_type, faction_symbol), (accepted, fulfilled)),
            ((deadline_to_accept, deadline), (on_accepted, on_fulfilled)),
        ): (
            ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
            ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
        ) = items
            .iter()
            .map(|c| {
                (
                    (
                        ((c.id.clone(), c.contract_type), c.faction_symbol.clone()),
                        (c.accepted, c.fulfilled),
                    ),
                    (
                        (c.deadline_to_accept.clone(), c.deadline.clone()),
                        (c.on_accepted, c.on_fulfilled),
                    ),
                )
            })
            .unzip();

        let (id, contract_type): (Vec<String>, Vec<models::contract::Type>) =
            id_and_contract_type.into_iter().unzip();

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
              deadline
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
              $9::character varying[]
            )
            ON CONFLICT (id) DO UPDATE SET 
              contract_type = EXCLUDED.contract_type,
              faction_symbol = EXCLUDED.faction_symbol,
              accepted = EXCLUDED.accepted,
              fulfilled = EXCLUDED.fulfilled,
              deadline_to_accept = EXCLUDED.deadline_to_accept,
              on_accepted = EXCLUDED.on_accepted,
              on_fulfilled = EXCLUDED.on_fulfilled,
              deadline = EXCLUDED.deadline
        "#,
            &id as &[String],
            &contract_type as &[models::contract::Type],
            &faction_symbol as &[String],
            &accepted as &[bool],
            &fulfilled as &[bool],
            &deadline_to_accept as &[Option<String>],
            &on_accepted as &[i32],
            &on_fulfilled as &[i32],
            &deadline as &[String]
        )
        .execute(&database_pool.database_pool)
        .await;

        Ok(())
    }
    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<Contract>> {
        sqlx::query_as!(
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
                    deadline
                FROM contract
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl Contract {
    pub async fn insert_contract(
        database_pool: &DbPool,
        contract: models::Contract,
    ) -> sqlx::Result<()> {
        let contract_old = Contract::from(contract.clone());
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

    pub async fn get_by_id(database_pool: &DbPool, id: &String) -> sqlx::Result<Contract> {
        sqlx::query_as!(
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
          deadline
        FROM public.contract WHERE id = $1"#,
            &id
        )
        .fetch_one(&database_pool.database_pool)
        .await
    }

    pub async fn get_all_sm(database_pool: &DbPool) -> sqlx::Result<Vec<ContractSummary>> {
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
  contract.on_accepted + contract.on_fulfilled - COALESCE(sum(market_transaction.total_price), 0) as "net_profit: i32"
FROM
  public.contract
  join public.market_transaction ON market_transaction.contract = contract.id
group by
  contract.id
order by
  contract.deadline_to_accept ASC;
    "#,
)
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}
