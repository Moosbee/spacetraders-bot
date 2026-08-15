use std::collections::HashMap;

use serde::de::DeserializeOwned;

#[cfg(any(test, feature = "test-support"))]
pub fn get_waypoints<T: DeserializeOwned>() -> Vec<T> {
    let text = include_str!("system_X1_JP44.json");
    serde_json::from_str(text).unwrap()
}

#[cfg(any(test, feature = "test-support"))]
pub fn get_jump_gate_connections<T: DeserializeOwned>() -> GateConnections<T> {
    let text = include_str!("jump_gate_connections.json");
    serde_json::from_str(text).unwrap()
}

#[derive(Debug, serde::Deserialize)]
struct GateConnections<T> {
    connections: Vec<T>,
    system_to_gate_mapping: HashMap<String, String>,
}
