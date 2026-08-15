mod test_data;

#[cfg(any(test, feature = "test-support"))]
pub use test_data::get_jump_gate_connections;
#[cfg(any(test, feature = "test-support"))]
pub use test_data::get_waypoints;
