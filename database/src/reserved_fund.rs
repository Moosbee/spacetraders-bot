use tracing::instrument;

use crate::DatabaseConnector;

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ReservedFund {
    pub id: i64,
    pub amount: i64,
    pub status: FundStatus,
    pub actual_amount: i64,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl PartialEq for ReservedFund {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.amount == other.amount
            && self.status == other.status
            && self.actual_amount == other.actual_amount
    }
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
#[sqlx(type_name = "fund_status")]
pub enum FundStatus {
    #[default]
    #[sqlx(rename = "RESERVED")]
    Reserved,
    #[sqlx(rename = "USED")]
    Used,
    #[sqlx(rename = "CANCELLED")]
    Cancelled,
}

impl ReservedFund {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn insert_new(
        database_pool: &crate::DbPool,
        funds: ReservedFund,
    ) -> crate::Result<i64> {
        let id = sqlx::query!(
            r#"
              INSERT INTO reserved_funds (amount, status, actual_amount, created_at, updated_at)
              VALUES ($1, $2::fund_status, $3, NOW(), NOW())
              RETURNING id
          "#,
            &funds.amount,
            &funds.status as &FundStatus,
            &funds.actual_amount
        )
        .fetch_one(&database_pool.database_pool)
        .await?
        .id;

        Ok(id)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_id(
        database_pool: &crate::DbPool,
        id: &i64,
    ) -> crate::Result<Option<ReservedFund>> {
        let result = sqlx::query_as!(
            ReservedFund,
            r#"
              SELECT
                id,
                amount,
                status as "status: FundStatus",
                actual_amount,
                created_at,
                updated_at
              FROM reserved_funds
              WHERE id = $1
          "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(result)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_status(
        database_pool: &crate::DbPool,
        status: FundStatus,
    ) -> crate::Result<Vec<ReservedFund>> {
        let result = sqlx::query_as!(
            ReservedFund,
            r#"
              SELECT
                id,
                amount,
                status as "status: FundStatus",
                actual_amount,
                created_at,
                updated_at
              FROM reserved_funds
              WHERE status = $1::fund_status
          "#,
            &status as &FundStatus
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(result)
    }
}

impl DatabaseConnector<ReservedFund> for ReservedFund {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &crate::DbPool, item: &ReservedFund) -> crate::Result<()> {
        sqlx::query!(
            r#"
              INSERT INTO reserved_funds (
                id,
                amount,
                status,
                actual_amount,
                created_at,
                updated_at
              )
              VALUES (
                $1, $2, $3::fund_status, $4, NOW(), NOW()
              )
              ON CONFLICT (id) DO UPDATE SET
                amount = EXCLUDED.amount,
                status = EXCLUDED.status,
                actual_amount = EXCLUDED.actual_amount,
                updated_at = NOW();
          "#,
            &item.id,
            &item.amount,
            &item.status as &FundStatus,
            &item.actual_amount
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &crate::DbPool,
        items: &[ReservedFund],
    ) -> crate::Result<()> {
        let (ids, amounts, statuses, actual_amounts): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(
                items
                    .iter()
                    .map(|rf| (rf.id, rf.amount, rf.status, rf.actual_amount)),
            );

        sqlx::query!(
            r#"
          INSERT INTO reserved_funds (
              id,
              amount,
              actual_amount,
              status,
              created_at,
              updated_at
          )
          SELECT 
              id,
              amt,
              actual_amte,
              stat,
              NOW(),
              NOW()
          FROM UNNEST(
              $1::bigint[],
              $2::bigint[],
              $3::bigint[],
              $4::fund_status[]
          ) AS t(id, amt, actual_amte, stat)
          ON CONFLICT (id) DO UPDATE
          SET amount = EXCLUDED.amount,
              actual_amount = EXCLUDED.actual_amount,
              status = EXCLUDED.status,
              updated_at = NOW();
          "#,
            &ids,
            &amounts,
            &actual_amounts,
            &statuses as &[FundStatus],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &crate::DbPool) -> crate::Result<Vec<ReservedFund>> {
        let result = sqlx::query_as!(
            ReservedFund,
            r#"
              SELECT
                id,
                amount,
                status as "status: FundStatus",
                actual_amount,
                created_at,
                updated_at
              FROM reserved_funds
          "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(result)
    }
}
