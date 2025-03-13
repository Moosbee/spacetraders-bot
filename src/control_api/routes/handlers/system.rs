use warp::reply::Reply;

use crate::{control_api::types::Result, types::ConductorContext};

pub async fn handle_get_api_counter(context: ConductorContext) -> Result<impl Reply> {
    let counter = context.api.get_limiter().get_counter();
    Ok(warp::reply::json(&serde_json::json!({"counter": counter})))
}
