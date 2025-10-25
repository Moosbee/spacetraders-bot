use std::{collections::HashMap, ops::Add};

#[derive(Debug, serde::Serialize, Clone, Copy)]
pub enum RequestedShipType {
    Scrapper, // Used for market scrapping does not need speed, cargo or anything, the cheapest ship will do
    Explorer, // ship with a warp drive
    Probe,    // a ship with no fuel thus infinite range, but nothing else
    Transporter, // ship with at least 40 units of cargo and decent range
    Mining,   // ship equipped with extractor setup and cargo
    Siphon,   // ship equipped with siphon setup and cargo
    Survey,   // ship equipped with survey setup does not need cargo
}

// how low money can go before we stop buying ships
#[derive(Debug, serde::Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    // High = 10_000_000,
    // Medium = 20_000_000,
    // Low = 100_000_000,
    High = 100_000,
    Medium = 500_000,
    Low = 1_000_000,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Priority::High, Priority::High) => std::cmp::Ordering::Equal,
            (Priority::High, _) => std::cmp::Ordering::Less,
            (_, Priority::High) => std::cmp::Ordering::Greater,
            (Priority::Medium, Priority::Medium) => std::cmp::Ordering::Equal,
            (Priority::Medium, _) => std::cmp::Ordering::Less,
            (_, Priority::Medium) => std::cmp::Ordering::Greater,
            (Priority::Low, Priority::Low) => std::cmp::Ordering::Equal,
        }
    }
}

// how expensive each ship can be
#[derive(Debug, serde::Serialize, Clone, Copy)]
pub enum Budget {
    VeryHigh = 10_000_000,
    High = 1_000_000,
    Medium = 500_000,
    Low = 100_000,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct RequiredShips {
    pub ships: HashMap<String, Vec<(RequestedShipType, Priority, Budget)>>,
}

impl RequiredShips {
    pub fn new() -> Self {
        Self {
            ships: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.ships.len()
    }
}

impl Add for RequiredShips {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_ships = RequiredShips::default();

        for (system_symbol, ships) in self.ships {
            let existing_ships = new_ships.ships.entry(system_symbol).or_default();
            existing_ships.extend(ships);
        }

        for (system_symbol, ships) in rhs.ships {
            let existing_ships = new_ships.ships.entry(system_symbol).or_default();
            existing_ships.extend(ships);
        }

        new_ships
    }
}

#[derive(Debug)]
pub enum FleetMessage {
    ScrapperAtShipyard {
        waypoint_symbol: String,
        ship_symbol: String,
        callback: tokio::sync::oneshot::Sender<String>,
    },
}

pub type FleetManagerMessage = FleetMessage;
