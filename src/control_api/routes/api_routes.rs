// routes.rs

use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::workers::types::ConductorContext;

use super::{handlers, with_context};

pub(crate) fn build_api_routes(
    context: &ConductorContext,
    cancel_tokens: Vec<(String, bool, CancellationToken)>,
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

    // Waypoint routes
    let waypoints = warp::path("waypoints")
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_waypoints);

    // Shutdown route
    let shutdown = warp::path("shutdown").and(warp::post()).map(move || {
        log::info!("Shutting down server");
        cancel_tokens.iter().for_each(|(_, independent, token)| {
            if *independent {
                token.cancel();
            }
        });
        warp::reply()
    });

    let routes = ships
        .or(ship_buy)
        .or(ship_navigation)
        .or(ship_toggle_orbit)
        .or(ship_purchase_cargo)
        .or(trade_routes)
        .or(contract)
        .or(contracts)
        .or(transactions)
        .or(waypoints)
        .or(shutdown);

    routes
}
