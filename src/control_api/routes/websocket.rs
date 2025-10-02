use super::*;
use crate::control_api::types::{WsData, WsObject};
use futures::{FutureExt, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tracing::instrument;

pub fn build_ws_routes(
    ship_rx: MyReceiver<ship::MyShip>,
    agent_rx: MyReceiver<database::Agent>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::path("all"))
        .and(warp::ws())
        .and(warp::any().map(move || ship_rx.clone()))
        .and(warp::any().map(move || agent_rx.clone()))
        .map(
            |ws: warp::ws::Ws,
             ship_rx: MyReceiver<ship::MyShip>,
             agent_rx: MyReceiver<database::Agent>| {
                ws.on_upgrade(move |websocket| handle_ws_connection(websocket, ship_rx, agent_rx))
            },
        )
}

#[instrument(skip(websocket, ship_rx, agent_rx))]
async fn handle_ws_connection(
    websocket: warp::ws::WebSocket,
    ship_rx: MyReceiver<ship::MyShip>,
    agent_rx: MyReceiver<database::Agent>,
) {
    let (tx, _rx) = websocket.split();

    let ship_stream = BroadcastStream::new(ship_rx.0)
        .filter_map(|ship_result| async {
            ship_result.ok().and_then(|ship| {
                serde_json::to_string(&WsObject {
                    data: WsData::RustShip(ship),
                })
                .ok()
            })
        })
        .map(|text| Ok(warp::ws::Message::text(text)));

    let agent_stream = BroadcastStream::new(agent_rx.0)
        .filter_map(|agent_result| async {
            agent_result.ok().and_then(|agent| {
                serde_json::to_string(&WsObject {
                    data: WsData::MyAgent(agent),
                })
                .ok()
            })
        })
        .map(|text| Ok(warp::ws::Message::text(text)));

    let combined_stream = futures::stream::select(ship_stream, agent_stream);

    let forward_future = combined_stream.forward(tx).map(|result| {
        if let Err(e) = result {
            tracing::error!("websocket error: {:?}", e);
        }
    });

    let _result = forward_future.await;
}
