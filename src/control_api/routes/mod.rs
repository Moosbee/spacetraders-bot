mod api_routes;
mod handlers;
mod websocket;

use log::{debug, warn};
use tokio_util::sync::CancellationToken;
use warp::{reply::Reply, Filter};

use crate::{
    control_api::types::{MyReceiver, ServerError},
    utils::ConductorContext,
};

pub fn build_routes(
    context: ConductorContext,
    ship_rx: MyReceiver<ship::MyShip>,
    agent_rx: MyReceiver<database::Agent>,
    ship_cancellation_token: CancellationToken,
) -> impl Filter<Extract = impl Reply> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Origin",
            "Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST]);

    let main_routes = build_main_routes(&context);
    let ws_routes = websocket::build_ws_routes(ship_rx, agent_rx);
    let api_routes = api_routes::build_api_routes(&context, ship_cancellation_token);

    main_routes
        .or(ws_routes)
        .or(api_routes)
        .with(cors)
        .recover(handle_rejection)
}

async fn handle_rejection(err: warp::Rejection) -> crate::control_api::types::Result<impl Reply> {
    warn!("Rejection: {:?}", err);
    if let Some(e) = err.find::<ServerError>() {
        debug!("Error: {}", e);
        let code = match e {
            ServerError::BadRequest(_) => warp::http::StatusCode::BAD_REQUEST,
            ServerError::NotFound => warp::http::StatusCode::NOT_FOUND,
            _ => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(
                &serde_json::json!({ "error": e.to_string(),"debug": format!("{:?}", e) }),
            ),
            code,
        ));
    } else if let Some(e) = err.find::<warp::Rejection>() {
        if e.is_not_found() {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": "Not found" })),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "error": "Internal server error" })),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

fn build_main_routes(
    context: &ConductorContext,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let _ = context;

    warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./index.html"))
}

// Helper function to pass context to handlers
fn with_context(
    context: ConductorContext,
) -> impl Filter<Extract = (ConductorContext,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || context.clone())
}
