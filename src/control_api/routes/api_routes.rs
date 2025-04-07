// routes.rs

use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::utils::ConductorContext;

use super::{handlers, with_context};

// async fn handle_not_found() -> crate::control_api::types::Result<impl Reply> {
//     let _errr: () = Err(crate::control_api::types::ServerError::NotFound)?;
//     Ok(warp::reply::with_status(
//         warp::reply::reply(),
//         warp::http::StatusCode::NOT_FOUND,
//     ))
// }

pub(crate) fn build_api_routes(
    context: &ConductorContext,
    ship_cancellation_token: CancellationToken,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let context = context.clone();

    // let not_found_routes = warp::any().and_then(handle_not_found);
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

    let ship_jump = warp::path!("ship" / String / "jump")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_jump_ship);

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

    let ship_role = warp::path!("ship" / String / "role")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_change_role);

    let ship_chart = warp::path!("ship" / String / "chart")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_chart_waypoint);

    let ship_warp = warp::path!("ship" / String / "warp")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_warp_ship);

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

    let request_system = warp::path!("systems" / String / "request")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_request_system);

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

    let jump_gates = warp::path!("jumpGates")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_jump_gates);

    // insights routes

    // API Counter
    let api_counter = warp::path!("insights" / "apiCounter")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_api_counter);

    let running_contract_shipments = warp::path!("insights" / "contract" / "shipments")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_running_contract_shipments);

    let running_construction_shipments = warp::path!("insights" / "construction" / "shipments")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_running_construction_shipments);

    let running_mining_assignments = warp::path!("insights" / "mining" / "assignments")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_mining_assignments);

    let scrapping_queue = warp::path!("insights" / "scrapping" / "info" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_scrapping_info);

    let ship_discrepancy = warp::path!("insights" / "ship" / "discrepancy")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_ships_to_purchase);

    let possible_trades = warp::path!("insights" / "trade" / "possible")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_possible_trades);

    let run_info = warp::path!("insights" / "run" / "info")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_context(context.clone()))
        .and_then(handlers::handle_get_run_info);

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

    ships
        .or(ship_buy)
        .or(ship_navigation)
        .or(ship_jump)
        .or(ship_toggle_orbit)
        .or(ship_purchase_cargo)
        .or(ship_toggle_activation)
        .or(ship_role)
        .or(ship_chart)
        .or(ship_warp)
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
        .or(jump_gates)
        .or(api_counter)
        .or(running_contract_shipments)
        .or(running_construction_shipments)
        .or(running_mining_assignments)
        .or(request_system)
        .or(scrapping_queue)
        .or(ship_discrepancy)
        .or(possible_trades)
        .or(run_info)
        .or(shutdown)
    // .or(not_found_routes)
}
