use crate::{
    sql,
    types::{ConductorContext, WaypointCan},
};

use super::mining_places::MiningPlaces;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Extract,
    Siphon,
}

impl ActionType {
    pub fn get_action(ship_clone: &crate::ship::MyShip) -> Option<ActionType> {
        match &ship_clone.status {
            crate::ship::ShipStatus::Mining {
                assignment: mining_ship_assignment,
            } => match mining_ship_assignment {
                crate::pilot::MiningShipAssignment::Transporter { .. } => None,
                crate::pilot::MiningShipAssignment::Extractor { .. } => Some(ActionType::Extract),
                crate::pilot::MiningShipAssignment::Siphoner { .. } => Some(ActionType::Siphon),
                crate::pilot::MiningShipAssignment::Surveyor => None,
                crate::pilot::MiningShipAssignment::Idle => None,
                crate::pilot::MiningShipAssignment::Useless => None,
            },
            _ => None,
        }
    }
}

pub struct FoundWaypointInfo {
    pub waypoint: sql::Waypoint,
    pub distance: i32,
    #[allow(dead_code)]
    pub next: String,
}

#[derive(Debug)]
pub struct PlaceFinder {
    context: ConductorContext,
}

impl PlaceFinder {
    pub fn new(context: ConductorContext) -> Self {
        Self { context }
    }

    pub async fn find(
        &self,
        ship_clone: crate::ship::MyShip,
        filter_fn: fn(&sql::Waypoint) -> bool,
        mining_places: &MiningPlaces,
    ) -> Result<Vec<FoundWaypointInfo>, crate::error::Error> {
        let sql_waypoints = sql::Waypoint::get_by_system(
            &self.context.database_pool,
            &ship_clone.nav.system_symbol,
        )
        .await?;
        let waypoints: Vec<FoundWaypointInfo> = self.get_best_waypoints(&sql_waypoints, filter_fn);

        let possible_waypoints: Vec<FoundWaypointInfo> = waypoints
            .into_iter()
            .filter(|wp| {
                let count = mining_places.get_count(&wp.waypoint.symbol);
                count
                    < mining_places
                        .get_max_miners_per_waypoint()
                        .try_into()
                        .unwrap()
            })
            .collect::<Vec<_>>();

        Ok(possible_waypoints)
    }

    fn get_best_waypoints(
        &self,
        system_waypoints: &[sql::Waypoint],
        filter: fn(&sql::Waypoint) -> bool,
    ) -> Vec<FoundWaypointInfo> {
        let points = system_waypoints
            .iter()
            .filter(|w| filter(w))
            .collect::<Vec<_>>();

        let markets = system_waypoints
            .iter()
            .filter(|w| w.is_marketplace())
            .collect::<Vec<_>>();

        let mut d_points: Vec<FoundWaypointInfo> = points
            .iter()
            .map(|wp| {
                let dis = markets
                    .iter()
                    .map(|market| {
                        let distance = self.distance_squared(wp, market);
                        (market.symbol.clone(), distance)
                    })
                    .min_by(|a, b| a.1.cmp(&b.1));

                let dis = dis.unwrap();

                FoundWaypointInfo {
                    waypoint: (*wp).clone(),
                    distance: dis.1,
                    next: dis.0.clone(),
                }
            })
            .collect::<Vec<_>>();

        d_points.sort_by(|a, b| a.distance.cmp(&b.distance));

        d_points
    }

    fn distance_squared(&self, a: &sql::Waypoint, b: &sql::Waypoint) -> i32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }
}
