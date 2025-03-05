use space_traders_client::models;

use super::DatabaseConnector;

pub struct MountInfo {
    pub symbol: models::ship_mount::Symbol,
    pub name: String,
    pub description: Option<String>,
    pub strength: Option<i32>,
    pub deposits: Option<Vec<models::TradeSymbol>>,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_mount::ShipMount> for MountInfo {
    fn from(value: models::ship_mount::ShipMount) -> Self {
        MountInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            strength: value.strength,
            deposits: value
                .deposits
                .map(|d| d.into_iter().map(|d| d.into()).collect()),
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl DatabaseConnector<MountInfo> for MountInfo {
    async fn insert(database_pool: &super::DbPool, item: &MountInfo) -> sqlx::Result<()> {
        let deposits = item.deposits.clone();
        sqlx::query!(
            r#"
                INSERT INTO mount_info (
                    symbol,
                    name,
                    description,
                    strength,
                    deposits,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    strength = EXCLUDED.strength,
                    deposits = EXCLUDED.deposits,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_mount::Symbol,
            item.name,
            item.description,
            item.strength,
            deposits as Option<Vec<models::TradeSymbol>>,
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
        items: &Vec<MountInfo>,
    ) -> sqlx::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }
        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<MountInfo>> {
        sqlx::query_as!(
            MountInfo,
            r#"
            SELECT
                symbol as "symbol: models::ship_mount::Symbol",
                name,
                description,
                strength,
                deposits as "deposits: Vec<models::TradeSymbol>",
                power_required,
                crew_required,
                slots_required
            FROM mount_info
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
