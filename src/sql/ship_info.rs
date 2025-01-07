use super::{sql_models::ShipInfo, DatabaseConnector, ShipInfoRole};

impl ShipInfo {
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> sqlx::Result<Option<ShipInfo>> {
        let erg = sqlx::query_as!(
            ShipInfo,
            r#"
        SELECT symbol, display_name, role as "role: crate::sql::sql_models::ShipInfoRole", active
        FROM ship_info WHERE symbol = $1
        LIMIT 1
      "#,
            symbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }

    pub async fn get_by_role(
        database_pool: &super::DbPool,
        symbol: &ShipInfoRole,
    ) -> sqlx::Result<Vec<ShipInfo>> {
        let erg = sqlx::query_as!(
            ShipInfo,
            r#"
        SELECT symbol, display_name, role as "role: crate::sql::sql_models::ShipInfoRole", active
        FROM ship_info WHERE role = $1
      "#,
            symbol as &crate::sql::sql_models::ShipInfoRole
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }
}

impl DatabaseConnector<ShipInfo> for ShipInfo {
    async fn insert(database_pool: &super::DbPool, item: &ShipInfo) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
              INSERT INTO public.ship_info(
                symbol,
                display_name,
                role,
                active
                ) VALUES (
                 $1,
                 $2,
                 $3::ship_info_role,
                 $4
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                role = EXCLUDED.role,
                active = EXCLUDED.active;
            "#,
            &item.symbol,
            &item.display_name,
            &item.role as &crate::sql::sql_models::ShipInfoRole,
            &item.active
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &super::DbPool, items: &Vec<ShipInfo>) -> sqlx::Result<()> {
        let ((symbol_s, display_name_s), (role_s, active_s)): (
            (Vec<String>, Vec<String>),
            (Vec<crate::sql::sql_models::ShipInfoRole>, Vec<bool>),
        ) = items
            .iter()
            .map(|s| {
                (
                    (s.symbol.clone(), s.display_name.clone()),
                    (s.role.clone(), s.active),
                )
            })
            .unzip();

        sqlx::query!(
            r#"
              INSERT INTO public.ship_info (
                symbol,
                display_name,
                role,
                active
                )
                SELECT * FROM UNNEST(
                  $1::character varying[],
                  $2::character varying[],
                  $3::ship_info_role[],
                  $4::boolean[]
                 )
                 on conflict (symbol) DO UPDATE SET 
                display_name = EXCLUDED.display_name,
                role = EXCLUDED.role,
                active = EXCLUDED.active
            "#,
            &symbol_s as &[String],
            &display_name_s as &[String],
            &role_s as &[crate::sql::sql_models::ShipInfoRole],
            &active_s as &[bool]
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ShipInfo>> {
        let erg = sqlx::query_as! {
            ShipInfo,
            r#"
                SELECT 
                    symbol,
                    display_name,
                    role as "role: crate::sql::sql_models::ShipInfoRole",
                    active
                FROM ship_info
            "#
        }
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }
}
