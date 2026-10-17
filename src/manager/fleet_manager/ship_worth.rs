use std::collections::HashMap;

use ::utils::get_system_symbol;

#[derive(Debug, Clone)]
pub struct ShipWorth<'a> {
    pub assignment: &'a database::ShipAssignment,
    pub shipyard_ship: &'a database::ShipyardShip,
    pub fleet: &'a database::Fleet,
    pub total_jumps: i32,
    pub total_distance: f64,
    pub travel_cost: i64,
    pub total_price: i64,
}

impl ShipWorth<'_> {
    pub fn new<'a>(
        assignment: &'a database::ShipAssignment,
        shipyard_ship: &'a database::ShipyardShip,
        fleet: &'a database::Fleet,
        travel_cost: &HashMap<(String, String), (i64, i32, f64)>,
    ) -> Option<ShipWorth<'a>> {
        let shipyard_system = get_system_symbol(&shipyard_ship.waypoint_symbol);
        let (total_cost, total_jumps, total_distance) =
            travel_cost.get(&(shipyard_system, fleet.system_symbol.clone()))?;
        Some(ShipWorth {
            assignment,
            shipyard_ship,
            fleet,
            total_jumps: *total_jumps,
            total_distance: *total_distance,
            travel_cost: *total_cost,
            total_price: *total_cost + shipyard_ship.purchase_price as i64,
        })
    }
}

impl PartialEq for ShipWorth<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.assignment.id == other.assignment.id
    }
}

impl Eq for ShipWorth<'_> {}

impl Ord for ShipWorth<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.assignment
            .priority
            .cmp(&other.assignment.priority)
            .then_with(|| self.total_price.cmp(&other.total_price))
            .then_with(|| self.assignment.id.cmp(&other.assignment.id))
    }
}

impl PartialOrd for ShipWorth<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
