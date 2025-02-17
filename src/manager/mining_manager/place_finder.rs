use space_traders_client::models;

use crate::types::WaypointCan;

use super::mining_places::MiningPlaces;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Extract,
    Siphon,
}

impl ActionType {
    pub fn get_action(ship_clone: &crate::ship::MyShip) -> Option<ActionType> {
        match ship_clone.role {
            crate::ship::Role::Construction => None,
            crate::ship::Role::Trader(_) => None,
            crate::ship::Role::Contract(_) => None,
            crate::ship::Role::Scraper => None,
            crate::ship::Role::Mining(mining_ship_assignment) => match mining_ship_assignment {
                crate::workers::mining::m_types::MiningShipAssignment::Transporter => None,
                crate::workers::mining::m_types::MiningShipAssignment::Extractor => {
                    Some(ActionType::Extract)
                }
                crate::workers::mining::m_types::MiningShipAssignment::Siphoner => {
                    Some(ActionType::Siphon)
                }
                crate::workers::mining::m_types::MiningShipAssignment::Surveyor => None,
                crate::workers::mining::m_types::MiningShipAssignment::Idle => None,
                crate::workers::mining::m_types::MiningShipAssignment::Useless => None,
            },
            crate::ship::Role::Manuel => None,
        }
    }
}

pub struct FoundWaypointInfo {
    pub waypoint: models::Waypoint,
    pub distance: i32,
    #[allow(dead_code)]
    pub next: String,
}

#[derive(Debug)]
pub struct PlaceFinder {
    context: crate::workers::types::ConductorContext,
}

impl PlaceFinder {
    pub fn new(context: crate::workers::types::ConductorContext) -> Self {
        Self { context }
    }

    pub async fn find(
        &self,
        ship_clone: crate::ship::MyShip,
        filter_fn: fn(&models::Waypoint) -> bool,
        mining_places: &MiningPlaces,
    ) -> Vec<FoundWaypointInfo> {
        let waypoints: Vec<FoundWaypointInfo> =
            self.get_best_waypoints(ship_clone.nav.system_symbol.clone(), filter_fn);

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

        possible_waypoints
    }

    fn get_best_waypoints(
        &self,
        system_symbol: String,
        filter: fn(&models::Waypoint) -> bool,
    ) -> Vec<FoundWaypointInfo> {
        let waypoints = {
            let erg = self.context.all_waypoints.try_get(&system_symbol);

            let waypoints = if erg.is_locked() {
                log::warn!("Failed to get system: {} waiting", system_symbol);
                let erg = self.context.all_waypoints.get(&system_symbol);
                log::warn!("Got system: {} waiting", system_symbol);
                erg
            } else {
                erg.try_unwrap()
            };

            let waypoints = waypoints.unwrap();

            waypoints.clone()
        };

        let points = waypoints
            .iter()
            .map(|w| w.1.clone())
            .filter(|w| filter(w))
            .collect::<Vec<_>>();

        let markets = waypoints
            .iter()
            .map(|w| w.1.clone())
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
                    waypoint: wp.clone(),
                    distance: dis.1,
                    next: dis.0.clone(),
                }
            })
            .collect::<Vec<_>>();

        d_points.sort_by(|a, b| a.distance.cmp(&b.distance));

        d_points
    }

    fn distance_squared(&self, a: &models::Waypoint, b: &models::Waypoint) -> i32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }
}
