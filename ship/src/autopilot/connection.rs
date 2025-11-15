use async_graphql::{SimpleObject, Union};
use space_traders_client::models::{self, ShipNavFlightMode};
use std::hash::Hash;
use utils::get_system_symbol;

#[derive(Debug, Clone)]
pub struct SimpleConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub connection_type: ConnectionType,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
    pub cost: f64,
    pub re_cost: f64,
    pub distance: f64,
}

impl PartialEq for SimpleConnection {
    fn eq(&self, other: &Self) -> bool {
        self.start_symbol == other.start_symbol
            && self.end_symbol == other.end_symbol
            && self.connection_type == other.connection_type
    }
}

impl Hash for SimpleConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start_symbol.hash(state);
        self.end_symbol.hash(state);
        self.connection_type.hash(state);
    }
}

impl Eq for SimpleConnection {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ConnectionType {
    JumpGate,
    Warp { nav_mode: models::ShipNavFlightMode },
    Navigate { nav_mode: models::ShipNavFlightMode },
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ConcreteConnection {
    JumpGate(JumpConnection),
    Warp(WarpConnection),
    Navigate(NavigateConnection),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JumpConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub distance: f64,
    pub cooldown_time: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WarpConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NavigateConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct Refuel {
    pub fuel_needed: i32,
    pub fuel_required: i32,
    pub start_is_marketplace: bool,
}

#[derive(Clone, Default, serde::Serialize, Debug)]
pub struct Route {
    pub connections: Vec<ConcreteConnection>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
    pub total_api_requests: i32,
}

// Connection variants as GraphQL objects
#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
#[graphql(name = "JumpConnection")]
#[graphql(complex)]
pub struct JumpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub distance: f64,
    pub cooldown_time: f64,
}

#[async_graphql::ComplexObject]
impl JumpConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.start_symbol).await?;
        Ok(erg)
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.end_symbol).await?;
        Ok(erg)
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
#[graphql(name = "WarpConnection")]
#[graphql(complex)]
pub struct WarpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[async_graphql::ComplexObject]
impl WarpConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.start_symbol).await?;
        Ok(erg)
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.end_symbol).await?;
        Ok(erg)
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
#[graphql(name = "NavigateConnection")]
#[graphql(complex)]
pub struct NavigateConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[async_graphql::ComplexObject]
impl NavigateConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.start_symbol).await?;
        Ok(erg)
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.end_symbol).await?;
        Ok(erg)
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }
}

#[derive(Debug, Clone, serde::Serialize, Union)]
#[graphql(name = "ConcreteConnection")]
pub enum ConcreteConnectionGQL {
    JumpGate(JumpConnectionGQL),
    Warp(WarpConnectionGQL),
    Navigate(NavigateConnectionGQL),
}

#[derive(Clone, Default, serde::Serialize, SimpleObject)]
#[graphql(name = "AutopilotRoute")]
pub struct RouteGQL {
    pub connections: Vec<ConcreteConnectionGQL>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
    pub total_api_requests: i32,
}

impl From<JumpConnection> for JumpConnectionGQL {
    fn from(conn: JumpConnection) -> Self {
        JumpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            distance: conn.distance,
            cooldown_time: conn.cooldown_time,
        }
    }
}

impl From<WarpConnection> for WarpConnectionGQL {
    fn from(conn: WarpConnection) -> Self {
        WarpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel,
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<NavigateConnection> for NavigateConnectionGQL {
    fn from(conn: NavigateConnection) -> Self {
        NavigateConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel,
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<ConcreteConnection> for ConcreteConnectionGQL {
    fn from(conn: ConcreteConnection) -> Self {
        match conn {
            ConcreteConnection::JumpGate(jump) => ConcreteConnectionGQL::JumpGate(jump.into()),
            ConcreteConnection::Warp(warp) => ConcreteConnectionGQL::Warp(warp.into()),
            ConcreteConnection::Navigate(nav) => ConcreteConnectionGQL::Navigate(nav.into()),
        }
    }
}

impl From<Route> for RouteGQL {
    fn from(route: Route) -> Self {
        RouteGQL {
            connections: route.connections.into_iter().map(|c| c.into()).collect(),
            total_distance: route.total_distance,
            total_fuel_cost: route.total_fuel_cost,
            total_travel_time: route.total_travel_time,
            total_api_requests: route.total_api_requests,
        }
    }
}
