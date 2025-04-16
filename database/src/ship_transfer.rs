use crate::{DatabaseConnector, DbPool, ShipInfoRole};

#[derive(Debug, Clone)]
pub struct ShipTransfer {
    pub id: i64,
    pub ship_symbol: String,
    pub system_symbol: String,
    pub role: ShipInfoRole,
    pub finished: bool,
}

impl ShipTransfer {
    pub async fn insert_new(database_pool: &DbPool, item: &ShipTransfer) -> crate::Result<i64> {
        let id = sqlx::query!(
            r#"
                INSERT INTO ship_transfers (
                  ship_symbol,
                  system_symbol,
                  role,
                  finished
                )
                VALUES ($1, $2, $3::ship_info_role, $4)
                  RETURNING id;
            "#,
            &item.ship_symbol,
            &item.system_symbol,
            &item.role as &ShipInfoRole,
            &item.finished
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(id.id)
    }

    pub async fn get_unfinished(database_pool: &DbPool) -> crate::Result<Vec<ShipTransfer>> {
        let erg = sqlx::query_as!(
            ShipTransfer,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  system_symbol,
                  role as "role: ShipInfoRole",
                  finished
                FROM ship_transfers
                WHERE finished = false
            "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<ShipTransfer> for ShipTransfer {
    async fn insert(database_pool: &DbPool, item: &ShipTransfer) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_transfers (
                  id,
                  ship_symbol,
                  system_symbol,
                  role,
                  finished
                )
                VALUES ($1, $2, $3, $4::ship_info_role, $5)
                ON CONFLICT (id) DO UPDATE SET
                  ship_symbol = EXCLUDED.ship_symbol,
                  system_symbol = EXCLUDED.system_symbol,
                  role = EXCLUDED.role,
                  finished = EXCLUDED.finished;
            "#,
            &item.id,
            &item.ship_symbol,
            &item.system_symbol,
            &item.role as &ShipInfoRole,
            &item.finished
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[ShipTransfer]) -> crate::Result<()> {
        let (ship_symbols, system_symbols, roles, finished_values): (
            Vec<String>,
            Vec<String>,
            Vec<ShipInfoRole>,
            Vec<bool>,
        ) = itertools::multiunzip(items.iter().map(|st| {
            (
                st.ship_symbol.clone(),
                st.system_symbol.clone(),
                st.role,
                st.finished,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO ship_transfers (
                ship_symbol,
                system_symbol,
                role,
                finished
            )
            SELECT ship, system, r, fin FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::ship_info_role[],
                $4::boolean[]
            ) AS t(ship, system, r, fin)
            ON CONFLICT (id) DO UPDATE
            SET ship_symbol = EXCLUDED.ship_symbol,
                system_symbol = EXCLUDED.system_symbol,
                role = EXCLUDED.role,
                finished = EXCLUDED.finished;
            "#,
            &ship_symbols,
            &system_symbols,
            &roles as &[ShipInfoRole],
            &finished_values
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<ShipTransfer>> {
        let erg = sqlx::query_as!(
            ShipTransfer,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  system_symbol,
                  role as "role: ShipInfoRole",
                  finished
                FROM ship_transfers
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}
