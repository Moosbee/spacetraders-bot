use chrono::NaiveDateTime;

use super::{DatabaseConnector, DbPool};

#[derive(Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JumpGateConnection {
    pub id: i64,
    pub from: String,
    pub to: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl JumpGateConnection {
    pub async fn get_all_from(
        database_pool: &DbPool,
        from: &String,
    ) -> sqlx::Result<Vec<JumpGateConnection>> {
        sqlx::query_as!(
            JumpGateConnection,
            r#"
        SELECT
          id,
          waypoint_from as "from",
          waypoint_to as "to",
          created_at,
          updated_at
        FROM jump_gate_connections
        WHERE waypoint_from = $1
      "#,
            from
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<JumpGateConnection> for JumpGateConnection {
    async fn insert(database_pool: &DbPool, item: &JumpGateConnection) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO jump_gate_connections (
                  waypoint_from,
                  waypoint_to,
                  updated_at
                )
                VALUES ($1, $2, NOW())
                ON CONFLICT (waypoint_from, waypoint_to) DO UPDATE SET
                  updated_at = NOW();
            "#,
            &item.from,
            &item.to
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[JumpGateConnection]) -> sqlx::Result<()> {
        // for item in items.iter() {
        //     Self::insert(database_pool, item).await?;
        // }

        let (waypoints_from, waypoints_to): (Vec<String>, Vec<String>) =
            itertools::multiunzip(items.iter().map(|jg| (jg.from.clone(), jg.to.clone())));

        sqlx::query!(
            r#"
            INSERT INTO jump_gate_connections (
                waypoint_from,
                waypoint_to,
                updated_at
            )
            SELECT from_wp, to_wp, NOW() FROM UNNEST(
                $1::character varying[],
                $2::character varying[]
            ) AS t(from_wp, to_wp)
            ON CONFLICT (waypoint_from, waypoint_to) DO UPDATE
            SET updated_at = NOW();
            "#,
            &waypoints_from,
            &waypoints_to
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<JumpGateConnection>> {
        sqlx::query_as!(
            JumpGateConnection,
            r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
