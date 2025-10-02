use tracing::instrument;

use crate::DatabaseConnector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipJump {
    pub id: i64,
    pub ship_symbol: String,
    pub from: String,
    pub to: String,
    pub distance: i64,
    pub ship_before: i64,
    pub ship_after: i64,
}

impl DatabaseConnector<ShipJump> for ShipJump {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &ShipJump) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_jumps (
                    ship_symbol,
                    "from",
                    "to",
                    distance,
                    ship_before,
                    ship_after
                )
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.ship_before,
            item.ship_after
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[ShipJump]) -> crate::Result<()> {
        let (ship_symbols, froms, tos, distances, ship_befores, ship_afters): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|j| {
            (
                j.ship_symbol.clone(),
                j.from.clone(),
                j.to.clone(),
                j.distance,
                j.ship_before,
                j.ship_after,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO ship_jumps (
                ship_symbol,
                "from",
                "to",
                distance,
                ship_before,
                ship_after
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::character varying[],
                $4::bigint[],
                $5::bigint[],
                $6::bigint[]
            )
            "#,
            &ship_symbols,
            &froms,
            &tos,
            &distances,
            &ship_befores,
            &ship_afters
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<ShipJump>> {
        let results = sqlx::query_as!(
            ShipJump,
            r#"
            SELECT
                id,
                ship_symbol,
                "from",
                "to",
                distance,
                ship_before,
                ship_after
            FROM ship_jumps
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(results)
    }
}
