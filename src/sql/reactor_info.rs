use space_traders_client::models;

use super::DatabaseConnector;

pub struct ReactorInfo {
    pub symbol: models::ship_reactor::Symbol,
    pub name: String,
    pub description: String,
    pub power_output: i32,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_reactor::ShipReactor> for ReactorInfo {
    fn from(value: models::ship_reactor::ShipReactor) -> Self {
        ReactorInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            power_output: value.power_output,
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl DatabaseConnector<ReactorInfo> for ReactorInfo {
    async fn insert(database_pool: &super::DbPool, item: &ReactorInfo) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO reactor_info (
                    symbol,
                    name,
                    description,
                    power_output,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    power_output = EXCLUDED.power_output,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_reactor::Symbol,
            item.name,
            item.description,
            item.power_output,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<ReactorInfo>,
    ) -> sqlx::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            power_outputs,
            power_requireds,
            crew_requireds,
            slots_requireds,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(items.iter().map(|r| {
                (
                    r.symbol,
                    r.name.clone(),
                    r.description.clone(),
                    r.power_output,
                    r.power_required,
                    r.crew_required,
                    r.slots_required,
                )
            }));

        sqlx::query!(
            r#"
            INSERT INTO reactor_info (
                symbol,
                name,
                description,
                power_output,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_reactor_symbol[],
                $2::character varying[],
                $3::character varying[],
                $4::integer[],
                $5::integer[],
                $6::integer[],
                $7::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                power_output = EXCLUDED.power_output,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_reactor::Symbol],
            &names,
            &descriptions,
            &power_outputs,
            &power_requireds as &[Option<i32>],
            &crew_requireds as &[Option<i32>],
            &slots_requireds as &[Option<i32>]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ReactorInfo>> {
        sqlx::query_as!(
            ReactorInfo,
            r#"
            SELECT
                symbol as "symbol: models::ship_reactor::Symbol",
                name,
                description,
                power_output,
                power_required,
                crew_required,
                slots_required
            FROM reactor_info
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
