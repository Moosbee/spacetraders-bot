struct ShipyardShipReservation {}

enum ShipState {
    Present { waypoint: WaypointSymbol },
    Reserved { waypoint: WaypointSymbol },
}

impl ShipyardShipReservation {
    // new workflow
    // the scrapper arrives at the shipyard
    // registers himself as present
    // waits until the scrap and scraps
    // the ship will try to unregister himself, but the manager can reserve the ship
    // the ship cannot unregister itself while it is reserved and waits for the reservation to release

    pub async fn register(&self, ship: &ShipSymbol, wp: &WaypointSymbol) -> Result<()>;
    /// Waits while reserved, then removes the ship.
    pub async fn deregister(&self, ship: &ShipSymbol) -> Result<WaypointSymbol>;
    pub async fn state(&self, ship: &ShipSymbol) -> Option<ShipState>;
    /// Fails if the ship isn't present; released on drop.
    pub async fn reserve(&self, ship: &ShipSymbol) -> Result<ShipReservation>;
}
