// routes.rs

use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::types::ConductorContext;

use super::{handlers, with_context};

pub(crate) fn build_api_routes(
    context: &ConductorContext,
    ship_cancellation_token: CancellationToken,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let context = context.clone();

    // Ships routes
    let ships = warp::path("ships")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_ships);

    // Ship routes
    let ship_buy = warp::path!("ship" / "buy")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_buy_ship);

    let ship_navigation = warp::path!("ship" / String / "navigate")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_navigate_ship);

    let ship_toggle_orbit = warp::path!("ship" / String / "toggleOrbit")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_toggle_orbit);

    let ship_purchase_cargo = warp::path!("ship" / String / "purchaseCargo")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_purchase_cargo_ship);

    let ship_toggle_activation = warp::path!("ship" / String / "toggleActivation")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_toggle_activation);

    let ship_cargo = warp::path!("ship" / String / "role")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_change_role);

    // Trade routes
    let trade_routes = warp::path("tradeRoutes")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_trade_routes);

    // Contract routes
    let contracts = warp::path("contracts")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_contracts);

    let contract = warp::path!("contracts" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_contract);

    // Construction routes
    let construction_materials = warp::path!("construction" / "materials")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_construction_materials);

    let construction_shipments = warp::path!("construction" / "shipments")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_construction_shipments);

    // Transaction routes
    let transactions = warp::path("transactions")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_transactions);

    // Waypoints routes
    let waypoints = warp::path("waypoints")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_waypoints);
    // Waypoint routes
    let waypoint = warp::path!("waypoints" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_waypoint);

    // systems routes
    let systems = warp::path("systems")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_systems);

    // system routes
    let system = warp::path!("systems" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_system);

    // agents routes
    let agents = warp::path!("agents")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agents);

    // agents routes
    let agent = warp::path!("agents" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agent);

    // agents routes
    let agent_history = warp::path!("agents" / String / "history")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_agent_history);

    // Shutdown route
    let shutdown = warp::path("shutdown")
        .and(warp::path::end())
        .and(warp::post())
        .map(move || {
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
        .or(construction_materials)
        .or(construction_shipments)
        .or(transactions)
        .or(waypoint)
        .or(waypoints)
        .or(system)
        .or(systems)
        .or(agent_history)
        .or(agent)
        .or(agents)
        .or(shutdown);

    routes
}
