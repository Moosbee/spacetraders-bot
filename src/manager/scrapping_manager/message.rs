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

#[derive(Debug)]
pub enum ScrapResponse {
    Unassigned,
    Scrapping {
        waypoint_symbol: String,
        date: chrono::DateTime<chrono::Utc>,
    },
}

pub enum ScrapMessage {
    Next {
        ship_clone: ship::MyShipCopy,
        callback: tokio::sync::oneshot::Sender<ScrapResponse>,
    },
    Complete {
        ship_clone: ship::MyShipCopy,
        waypoint_symbol: String,
    },
    Fail {
        // or cancel
        ship_clone: ship::MyShipCopy,
        waypoint_symbol: String,
    },
    GetAll {
        ship_clone: ship::MyShipCopy,

        callback: tokio::sync::oneshot::Sender<Vec<(String, chrono::DateTime<chrono::Utc>)>>,
    },
}

impl std::fmt::Debug for ScrapMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Next {
                ship_clone,
                callback,
            } => f
                .debug_struct("Next")
                .field(
                    "ship_clone",
                    &format!(
                        "{} @ {}/{}",
                        ship_clone.symbol,
                        ship_clone.nav.system_symbol,
                        ship_clone.nav.waypoint_symbol
                    ),
                )
                .field("callback", callback)
                .finish(),
            Self::Complete {
                ship_clone,
                waypoint_symbol,
            } => f
                .debug_struct("Complete")
                .field(
                    "ship_clone",
                    &format!(
                        "{} @ {}/{}",
                        ship_clone.symbol,
                        ship_clone.nav.system_symbol,
                        ship_clone.nav.waypoint_symbol
                    ),
                )
                .field("waypoint_symbol", waypoint_symbol)
                .finish(),
            Self::Fail {
                ship_clone,
                waypoint_symbol,
            } => f
                .debug_struct("Fail")
                .field(
                    "ship_clone",
                    &format!(
                        "{} @ {}/{}",
                        ship_clone.symbol,
                        ship_clone.nav.system_symbol,
                        ship_clone.nav.waypoint_symbol
                    ),
                )
                .field("waypoint_symbol", waypoint_symbol)
                .finish(),
            Self::GetAll {
                ship_clone,
                callback,
            } => f
                .debug_struct("GetAll")
                .field(
                    "ship_clone",
                    &format!(
                        "{} @ {}/{}",
                        ship_clone.symbol,
                        ship_clone.nav.system_symbol,
                        ship_clone.nav.waypoint_symbol
                    ),
                )
                .field("callback", callback)
                .finish(),
        }
    }
}

impl std::fmt::Display for ScrapMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapMessage::Next { ship_clone, .. } => {
                write!(f, "ScrapMessage::Next {}", ship_clone.symbol)
            }
            ScrapMessage::Complete {
                ship_clone,
                waypoint_symbol,
            } => write!(
                f,
                "ScrapMessage::Complete {} {}",
                ship_clone.symbol, waypoint_symbol
            ),
            ScrapMessage::Fail {
                ship_clone,
                waypoint_symbol,
            } => write!(
                f,
                "ScrapMessage::Fail {} {}",
                ship_clone.symbol, waypoint_symbol
            ),
            ScrapMessage::GetAll { .. } => write!(f, "ScrapMessage::GetAll"),
        }
    }
}

pub type ScrappingManagerMessage = ScrapMessage;
