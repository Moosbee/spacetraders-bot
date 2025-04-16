use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WaypointInfo {
    pub waypoint_symbol: String,
    assigned_ships: HashMap<String, AssignLevel>, // ship_symbol -> level,
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl WaypointInfo {
    pub fn get_ship_level(&self, ship_symbol: &str) -> Option<AssignLevel> {
        self.assigned_ships.get(ship_symbol).cloned()
    }

    pub fn get_count(&self) -> usize {
        self.assigned_ships.len()
    }

    pub fn get_count_on_way(&self) -> usize {
        self.assigned_ships
            .iter()
            .filter(|(_, level)| *level == &AssignLevel::OnTheWay)
            .count()
    }

    pub fn get_count_active(&self) -> usize {
        self.assigned_ships
            .iter()
            .filter(|(_, level)| *level == &AssignLevel::Active)
            .count()
    }

    pub fn get_count_inactive(&self) -> usize {
        self.assigned_ships
            .iter()
            .filter(|(_, level)| *level == &AssignLevel::Inactive)
            .count()
    }

    pub fn get_count_active_on_way(&self) -> usize {
        self.assigned_ships
            .iter()
            .filter(|(_, level)| *level != &AssignLevel::Inactive)
            .count()
    }

    pub fn ship_iter(&self) -> impl Iterator<Item = (&String, &AssignLevel)> {
        self.assigned_ships.iter()
    }

    pub fn get_last_updated(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_updated
    }
}

#[derive(Debug)]
pub struct MiningPlaces {
    mining_places: HashMap<String, WaypointInfo>,
    max_miners_per_waypoint: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AssignLevel {
    /// Ship is at the waypoint but not active
    Inactive,
    /// Ship is assigned to a waypoint and is on its way
    OnTheWay,
    /// Ship is assigned to a waypoint and is there
    Active,
}

impl MiningPlaces {
    pub fn new(max_miners_per_waypoint: u32) -> MiningPlaces {
        MiningPlaces {
            mining_places: HashMap::new(),
            max_miners_per_waypoint,
        }
    }

    /// Assigns a ship to a waypoint, but not as the active ship
    ///
    /// If the ship is already assigned to the waypoint, it will be upgraded to on the way
    ///
    /// If the waypoint is not at max capacity, the ship will be assigned to the waypoint at level "on the way"
    ///
    /// Returns 1 if the ship was assigned, 0 if the waypoint is at max capacity, 3 if the ship is already the active ship
    pub fn try_assign_on_way(&mut self, ship_symbol: &str, waypoint: &str, no_limit: bool) -> u8 {
        let wp = self
            .mining_places
            .entry(waypoint.to_string())
            .or_insert_with(|| WaypointInfo {
                waypoint_symbol: waypoint.to_string(),
                assigned_ships: HashMap::new(),
                last_updated: chrono::DateTime::<chrono::Utc>::MIN_UTC, //never been updated
            });

        let size = wp.get_count_active_on_way() as u32;

        let ship = wp.assigned_ships.entry(ship_symbol.to_string());

        match ship {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                match occupied_entry.get() {
                    AssignLevel::Inactive => {
                        if (size < self.max_miners_per_waypoint) || (no_limit) {
                            occupied_entry.insert(AssignLevel::OnTheWay);
                            1
                        } else {
                            occupied_entry.remove();
                            0
                        }
                    }
                    AssignLevel::OnTheWay => 1,
                    AssignLevel::Active => 3,
                }
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                if size < self.max_miners_per_waypoint || (no_limit) {
                    vacant_entry.insert(AssignLevel::OnTheWay);
                    1
                } else {
                    0
                }
            }
        }
    }

    /// Assigns a ship to a waypoint, making it the active ship if possible
    ///
    /// If the ship is already assigned to the waypoint, it will be upgraded to active
    ///
    /// If the waypoint is not at max capacity, the ship will be assigned to the waypoint at level "active"
    ///
    /// Returns true if the ship was assigned, false otherwise
    pub fn try_assign_active(&mut self, ship_symbol: &str, waypoint: &str, no_limit: bool) -> bool {
        let wp = self.mining_places.get_mut(waypoint);

        match wp {
            Some(waypoint) => {
                let count = waypoint.get_count_active_on_way();

                let ship = waypoint.assigned_ships.get_mut(ship_symbol);

                match ship {
                    Some(ship) => match *ship {
                        AssignLevel::OnTheWay => {
                            *ship = AssignLevel::Active;
                            true
                        }
                        AssignLevel::Active => true,
                        AssignLevel::Inactive => {
                            if count < self.max_miners_per_waypoint.try_into().unwrap() || no_limit
                            {
                                *ship = AssignLevel::OnTheWay;
                                true
                            } else {
                                false
                            }
                        }
                    },
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Assigns a ship to a waypoint, making it the inactive ship
    ///
    /// If the ship is already assigned to the waypoint, it will be downgraded to inactive
    ///
    /// Returns true if the ship was assigned, false otherwise
    pub fn try_assign_inactive(&mut self, ship_symbol: &str, waypoint: &str) -> bool {
        let wp = self.mining_places.get_mut(waypoint);

        match wp {
            Some(waypoint) => {
                let ship = waypoint.assigned_ships.get_mut(ship_symbol);

                match ship {
                    Some(ship) => {
                        *ship = AssignLevel::Inactive;
                        true
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Tries to unassign a ship from a waypoint
    ///
    /// If the ship is assigned to the waypoint, it will be removed from the waypoint's list of assigned ships
    ///
    /// Returns true if the ship was unassigned, false otherwise
    pub fn try_unassign(&mut self, ship_symbol: &str, waypoint: &str) -> bool {
        let wp = self.mining_places.get_mut(waypoint);

        match wp {
            Some(waypoint) => waypoint.assigned_ships.remove(ship_symbol).is_some(),
            None => false,
        }
    }

    // fn get_all(&self) -> HashMap<String, WaypointInfo> {
    //     self.mining_places.clone()
    // }

    // fn get_info(&self, waypoint: &str) -> Option<WaypointInfo> {
    //     self.mining_places.get(waypoint).cloned()
    // }

    pub fn has_ship(&self, ship_symbol: &str, waypoint: &str) -> bool {
        self.mining_places
            .get(waypoint)
            .map(|s| s.assigned_ships.contains_key(ship_symbol))
            .unwrap_or(false)
    }

    /// Finds the waypoint and assignment level of a ship by its symbol.
    ///
    /// Returns `None` if the ship is not assigned to any waypoint.
    /// Returns `Some((waypoint_symbol, assignment_level))` if the ship is assigned.
    pub fn get_ship(&self, ship_symbol: &str) -> Option<(String, AssignLevel)> {
        self.mining_places
            .values()
            .find(|s| s.assigned_ships.contains_key(ship_symbol))
            .map(|s| {
                (
                    s.waypoint_symbol.clone(),
                    *s.assigned_ships.get(ship_symbol).unwrap(),
                )
            })
    }

    pub fn get_count(&self, waypoint: &str) -> usize {
        self.mining_places
            .get(waypoint)
            .map(|s| s.get_count())
            .unwrap_or(0)
    }

    pub fn up_date(&mut self, waypoint: &str) {
        let wp = self.mining_places.get_mut(waypoint);
        if let Some(waypoint) = wp {
            waypoint.last_updated = chrono::Utc::now();
        }
    }

    pub fn get_max_miners_per_waypoint(&self) -> u32 {
        self.max_miners_per_waypoint
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &WaypointInfo)> {
        self.mining_places.iter()
    }
}
