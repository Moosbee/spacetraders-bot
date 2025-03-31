/*
the ship requests what to scrap next in the current system

if all markets have a scrapper the manager tells him to stay at the waypoint and a date to wait and scrap.
the scrap date is calculated since the last time it updated plus a interval constant and the waypoints importance
importance is calculated based on the amount of item the marketplace sells, what type they are(EXCHANGE being the worst)
if the ship isn't at a market and all markets are already taken the ship will be unassigned from scrapping
if the waypoint also contains a shipyard the shipyard will be scrapped at the same time

If there are multiple free markets the ship will go to the closest that exceeded it's scrap date and which doesn't have a scrapper on it's way
The ship will then go to the waypoint and wait until said date comes and scrap

if there are ships to spare, shipyards(all or only requested) get a static scrapper directly
*/

use crate::manager::fleet_manager::message::RequiredShips;

#[derive(Debug)]
pub enum ScrapResponse {
    Unassigned,
    Scrapping {
        waypoint_symbol: String,
        date: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug)]
pub enum ScrapMessage {
    Next {
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<ScrapResponse>,
    },
    Complete {
        ship_clone: crate::ship::MyShip,
        waypoint_symbol: String,
    },
    Fail {
        // or cancel
        ship_clone: crate::ship::MyShip,
        waypoint_symbol: String,
    },
    GetAll {
        ship_clone: crate::ship::MyShip,

        callback: tokio::sync::oneshot::Sender<Vec<(String, chrono::DateTime<chrono::Utc>)>>,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

pub type ScrappingManagerMessage = ScrapMessage;
