pub fn get_system_symbol(waypoint_symbol: &str) -> String {
    let mut parts = waypoint_symbol.split('-');

    parts.next().unwrap_or_default().to_string() + "-" + parts.next().unwrap_or_default()
}

pub fn distance_between_waypoints(start: (i32, i32), end: (i32, i32)) -> f64 {
    (((end.0 as f64) - (start.0 as f64)).powi(2) + ((end.1 as f64) - (start.1 as f64)).powi(2))
        .sqrt()
}
