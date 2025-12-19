use ::utils::get_system_symbol;

#[derive(Debug, Clone, PartialEq)]
pub struct ShipWorth<'a> {
    pub assignment: &'a database::ShipAssignment,
    pub shipyard_ship: &'a database::ShipyardShip,
    pub fleet: &'a database::Fleet,
    pub total_jumps: i32,
    pub total_distance: f64,
    pub total_price: i64,
}

impl ShipWorth<'_> {
    pub fn new<'a>(
        assignment: &'a database::ShipAssignment,
        shipyard_ship: &'a database::ShipyardShip,
        fleet: &'a database::Fleet,
        jump_gate: &mut ship::autopilot::jump_gate_nav::JumpPathfinder,
        antimatter_cost: i64,
    ) -> ShipWorth<'a> {
        let shipyard_system = get_system_symbol(&shipyard_ship.waypoint_symbol);
        let route = jump_gate.find_cached_route(&shipyard_system, &fleet.system_symbol);
        let (total_jumps, total_distance) = route
            .iter()
            .fold((0, 0.0), |acc, c| (acc.0 + 1, acc.1 + c.conn.distance));
        let total_price =
            (shipyard_ship.purchase_price as i64) + (total_jumps as i64) * antimatter_cost;
        ShipWorth {
            assignment,
            shipyard_ship,
            fleet,
            total_jumps,
            total_distance,
            total_price,
        }
    }
}

impl PartialOrd for ShipWorth<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.assignment
                .priority
                .cmp(&other.assignment.priority)
                .then_with(|| self.total_price.cmp(&other.total_price)),
        )
    }
}
