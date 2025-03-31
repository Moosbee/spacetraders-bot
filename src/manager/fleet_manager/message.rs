use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
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
#[derive(Debug, serde::Serialize)]
pub enum Priority {
    High = 100_000,
    Medium = 500_000,
    Low = 1_000_000,
}

// how expensive each ship can be
#[derive(Debug, serde::Serialize)]
pub enum Budget {
    VeryHigh = 10_000_000,
    High = 1_000_000,
    Medium = 500_000,
    Low = 100_000,
}

#[derive(Debug, serde::Serialize)]
pub struct RequiredShips {
    pub ships: HashMap<String, Vec<(RequestedShipType, Priority, Budget)>>,
}

impl RequiredShips {
    pub fn new() -> Self {
        Self {
            ships: HashMap::new(),
        }
    }
}
