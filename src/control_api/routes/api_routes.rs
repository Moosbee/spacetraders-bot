// routes.rs

use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::workers::types::ConductorContext;

use super::{handlers, with_context};

pub(crate) fn build_api_routes(
    context: &ConductorContext,
    ship_cancellation_token: CancellationToken,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let context = context.clone();

    // Ships routes
    let ships = warp::path("ships")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_ships);

    // Ships routes
    let ship_buy = warp::path!("ship" / "buy")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_buy_ship);

    let ship_navigation = warp::path!("ship" / String / "navigate")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_navigate_ship);

    let ship_toggle_orbit = warp::path!("ship" / String / "toggleOrbit")
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_toggle_orbit);

    let ship_purchase_cargo = warp::path!("ship" / String / "purchaseCargo")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_purchase_cargo_ship);

    let ship_toggle_activation = warp::path!("ship" / String / "toggleActivation")
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_toggle_activation);

    let ship_cargo = warp::path!("ship" / String / "role")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_change_role);

    // Trade routes
    let trade_routes = warp::path("tradeRoutes")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_trade_routes);

    // Contract routes
    let contracts = warp::path("contracts")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_contracts);

    let contract = warp::path!("contracts" / String)
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_contract);

    // Transaction routes
    let transactions = warp::path("transactions")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_transactions);

    // Waypoints routes
    let waypoints = warp::path("waypoints")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_waypoints);
    // Waypoint routes
    let waypoint = warp::path!("waypoints" / String)
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_waypoint);

    // systems routes
    let systems = warp::path("systems")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_systems);

    // system routes
    let system = warp::path!("systems" / String)
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_system);

    // agents routes
    let agents = warp::path!("agents")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agents);

    // agents routes
    let agent = warp::path!("agents" / String)
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agent);

    // agents routes
    let agent_history = warp::path!("agents" / String / "history")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agent_history);

    // Shutdown route
    let shutdown = warp::path("shutdown").and(warp::post()).map(move || {
        log::info!("Shutting down server");
        ship_cancellation_token.cancel();
        log::debug!("Shut down server");

        warp::reply()
    });

    let routes = ships
        .or(ship_buy)
        .or(ship_navigation)
        .or(ship_toggle_orbit)
        .or(ship_purchase_cargo)
        .or(ship_toggle_activation)
        .or(ship_cargo)
        .or(trade_routes)
        .or(contract)
        .or(contracts)
        .or(transactions)
        .or(waypoints)
        .or(waypoint)
        .or(systems)
        .or(system)
        .or(agents)
        .or(agent)
        .or(agent_history)
        .or(shutdown);

    routes
}
