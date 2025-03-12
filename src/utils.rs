pub fn get_system_symbol(waypoint_symbol: &str) -> String {
    let mut parts = waypoint_symbol.split('-');

    return parts.next().unwrap_or_default().to_string() + "-" + parts.next().unwrap_or_default();
}
