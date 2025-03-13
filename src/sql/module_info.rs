use space_traders_client::models;

use super::DatabaseConnector;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleInfo {
    pub symbol: models::ship_module::Symbol,
    pub name: String,
    pub description: String,
    pub capacity: Option<i32>,
    pub range: Option<i32>,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_module::ShipModule> for ModuleInfo {
    fn from(value: models::ship_module::ShipModule) -> Self {
        ModuleInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            capacity: value.capacity,
            range: value.range,
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl DatabaseConnector<ModuleInfo> for ModuleInfo {
    async fn insert(database_pool: &super::DbPool, item: &ModuleInfo) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO module_info (
                    symbol,
                    name,
                    description,
                    range,
                    capacity,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    range = EXCLUDED.range,
                    capacity = EXCLUDED.capacity,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_module::Symbol,
            item.name,
            item.description,
            item.range,
            item.capacity,
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
        items: &Vec<ModuleInfo>,
    ) -> sqlx::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            ranges,
            capacities,
            power_requireds,
            crew_requireds,
            slots_requireds,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|m| {
            (
                m.symbol,
                m.name.clone(),
                m.description.clone(),
                m.range,
                m.capacity,
                m.power_required,
                m.crew_required,
                m.slots_required,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO module_info (
                symbol,
                name,
                description,
                range,
                capacity,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_module_symbol[],
                $2::character varying[],
                $3::character varying[],
                $4::integer[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                range = EXCLUDED.range,
                capacity = EXCLUDED.capacity,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_module::Symbol],
            &names,
            &descriptions,
            &ranges as &[Option<i32>],
            &capacities as &[Option<i32>],
            &power_requireds as &[Option<i32>],
            &crew_requireds as &[Option<i32>],
            &slots_requireds as &[Option<i32>]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ModuleInfo>> {
        sqlx::query_as!(
            ModuleInfo,
            r#"
            SELECT
                symbol as "symbol: models::ship_module::Symbol",
                name,
                description,
                range,
                capacity,
                power_required,
                crew_required,
                slots_required
            FROM module_info
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
