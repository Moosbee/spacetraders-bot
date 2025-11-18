use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct System {
    pub symbol: String,
    pub sector_symbol: String,
    pub system_type: models::SystemType,
    pub x: i32,
    pub y: i32,
    // pub factions: Vec<String>,
}

#[async_graphql::ComplexObject]
impl System {
    async fn waypoints(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::Waypoint>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        crate::Waypoint::get_by_system(database_pool, &self.symbol).await
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::MarketTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::MarketTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::ShipyardTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::ShipyardTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::ChartTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::ChartTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::RepairTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::RepairTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::ScrapTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::ScrapTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<crate::ShipModificationTransaction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let transactions =
            crate::ShipModificationTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::ShipyardShip>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history = crate::ShipyardShip::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::ShipyardShipTypes>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history =
            crate::ShipyardShipTypes::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn market_trades(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::MarketTrade>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history = crate::MarketTrade::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::MarketTradeGood>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history =
            crate::MarketTradeGood::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn fleets(&self, ctx: &async_graphql::Context<'_>) -> crate::Result<Vec<crate::Fleet>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history = crate::Fleet::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn surveys(&self, ctx: &async_graphql::Context<'_>) -> crate::Result<Vec<crate::Survey>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history = crate::Survey::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn extractions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::Extraction>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history = crate::Extraction::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::ConstructionMaterial>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history =
            crate::ConstructionMaterial::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn construction_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::ConstructionShipment>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history =
            crate::ConstructionShipment::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<crate::ContractDelivery>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let history =
            crate::ContractDelivery::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    // async fn contract_shipments(
    //     &self,
    //     ctx: &async_graphql::Context<'_>,
    // ) -> crate::Result<Vec<crate::ContractShipment>> {
    //     let database_pool = ctx.data::<crate::DbPool>().unwrap();
    //     let history =
    //         crate::ContractShipment::get_by_system_symbol(database_pool, &self.symbol).await?;
    //     Ok(history)
    // }

    async fn seen_agents(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> crate::Result<Vec<KnownAgent>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let system_market_transactions =
            crate::MarketTransaction::get_by_system(database_pool, &self.symbol).await?;

        let system_shipyard_transactions =
            crate::ShipyardTransaction::get_by_system(database_pool, &self.symbol).await?;

        let known_agents_iter = system_market_transactions
            .iter()
            .filter_map(|f| {
                f.ship_symbol
                    .chars()
                    .rev()
                    .collect::<String>()
                    .split_once("-")
                    .map(|f| f.1.chars().rev().collect::<String>())
            })
            .chain(
                system_shipyard_transactions
                    .iter()
                    .map(|f| f.agent_symbol.clone()),
            );

        let known_agents = known_agents_iter
            .fold(std::collections::HashMap::new(), |mut acc, f| {
                acc.entry(f).and_modify(|e: &mut u32| *e += 1).or_insert(1);
                acc
            })
            .into_iter()
            .map(|f| KnownAgent {
                symbol: f.0,
                count: f.1,
            })
            .collect();
        Ok(known_agents)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
struct KnownAgent {
    symbol: String,
    count: u32,
}

#[async_graphql::ComplexObject]
impl KnownAgent {
    async fn agent(&self, ctx: &async_graphql::Context<'_>) -> crate::Result<Option<crate::Agent>> {
        let database_pool = ctx.data::<crate::DbPool>().unwrap();
        let agent = crate::Agent::get_last_by_symbol(database_pool, &self.symbol).await?;
        Ok(agent)
    }
}

impl From<System> for (i32, i32) {
    fn from(value: System) -> Self {
        (value.x, value.y)
    }
}
impl From<&System> for (i32, i32) {
    fn from(value: &System) -> Self {
        (value.x, value.y)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct RespSystem {
    pub symbol: String,
    pub sector_symbol: String,
    pub system_type: models::SystemType,
    pub x: i32,
    pub y: i32,
    pub waypoints: Option<i32>,
    pub marketplaces: Option<i32>,
    pub shipyards: Option<i32>,
    pub has_my_ships: Option<bool>,
}

impl From<&models::System> for System {
    fn from(system: &models::System) -> Self {
        System {
            symbol: system.symbol.clone(),
            sector_symbol: system.sector_symbol.clone(),
            system_type: system.r#type,
            x: system.x,
            y: system.y,
        }
    }
}

impl RespSystem {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<RespSystem>> {
        let erg = sqlx::query_as!(
            RespSystem,
            r#"
            SELECT 
                system.symbol,
                system.sector_symbol,
                system.system_type as "system_type: models::SystemType",
                system.x,
                system.y,
            		count(waypoint.symbol) as "waypoints: i32",
				      	sum(CASE when waypoint.has_shipyard THEN 1 ELSE 0 END) as "shipyards: i32",
			      		sum(CASE when waypoint.has_marketplace THEN 1 ELSE 0 END) as "marketplaces: i32",
            		false as "has_my_ships: bool"
            FROM system left join waypoint on system.symbol = waypoint.system_symbol
			group by system.symbol
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;

        Ok(erg)
    }
}

impl System {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            System,
            r#"
            SELECT 
                symbol,
                sector_symbol,
                system_type as "system_type: models::SystemType",
                x,
                y
            FROM system
            WHERE symbol = $1
            LIMIT 1
            "#,
            symbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<System> for System {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &System) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO system (
                    symbol,
                    sector_symbol,
                    system_type,
                    x,
                    y
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (symbol) DO UPDATE
                SET sector_symbol = EXCLUDED.sector_symbol,
                    system_type = EXCLUDED.system_type,
                    x = EXCLUDED.x,
                    y = EXCLUDED.y
            "#,
            item.symbol,
            item.sector_symbol,
            item.system_type as models::SystemType,
            item.x,
            item.y
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[System]) -> crate::Result<()> {
        let (symbols, sector_symbols, system_types, xs, ys): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|s| {
            (
                s.symbol.clone(),
                s.sector_symbol.clone(),
                s.system_type,
                s.x,
                s.y,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO system (
                symbol,
                sector_symbol,
                system_type,
                x,
                y
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::system_type[],
                $4::integer[],
                $5::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET sector_symbol = EXCLUDED.sector_symbol,
                system_type = EXCLUDED.system_type,
                x = EXCLUDED.x,
                y = EXCLUDED.y
            "#,
            &symbols,
            &sector_symbols,
            &system_types as &[models::SystemType],
            &xs,
            &ys
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<System>> {
        let erg = sqlx::query_as!(
            System,
            r#"
            SELECT 
                symbol,
                sector_symbol,
                system_type as "system_type: models::SystemType",
                x,
                y
            FROM system
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl System {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: &String,
    ) -> crate::Result<Option<System>> {
        let erg = sqlx::query_as!(
            System,
            r#"
        SELECT 
            symbol,
            sector_symbol,
            system_type as "system_type: models::SystemType",
            x,
            y
        FROM system
        WHERE symbol = $1
        LIMIT 1
        "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}
