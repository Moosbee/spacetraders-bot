use space_traders_client::models;

struct Modes {
    burn: Mode,
    cruise: Mode,
    drift: Mode,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NavMode {
    Burn,
    Cruise,
    Drift,
    BurnAndCruise,
    CruiseAndDrift,
    BurnAndDrift,
    BurnAndCruiseAndDrift,
}

pub struct Mode {
    pub radius: f64,
    pub cost_multiplier: f64,
    pub mode: models::ShipNavFlightMode,
}

impl NavMode {
    pub fn get_flight_modes(&self, max_fuel: u32) -> Vec<Mode> {
        self.get_modes(NavMode::get_flight_mode_configs(max_fuel))
    }

    fn get_flight_mode_configs(max_fuel: u32) -> Modes {
        let cruise_base_radius = if max_fuel == 0 {
            f64::INFINITY
        } else {
            max_fuel as f64
        };
        Modes {
            burn: Mode {
                radius: (max_fuel as f64) / 2.0,
                cost_multiplier: 0.5,
                mode: models::ShipNavFlightMode::Burn,
            },
            cruise: Mode {
                radius: cruise_base_radius,
                cost_multiplier: 1.0,
                mode: models::ShipNavFlightMode::Cruise,
            },
            drift: Mode {
                radius: f64::INFINITY,
                cost_multiplier: 10.0,
                mode: models::ShipNavFlightMode::Drift,
            },
        }
    }
    fn get_modes(&self, all_modes: Modes) -> Vec<Mode> {
        let mut modes = Vec::new();
        if self.is_burn_mode() {
            modes.push(all_modes.burn);
        }
        if self.is_cruise_mode() {
            modes.push(all_modes.cruise);
        }
        if self.is_drift_mode() {
            modes.push(all_modes.drift);
        }
        modes
    }

    fn is_burn_mode(&self) -> bool {
        *self == NavMode::Burn
            || *self == NavMode::BurnAndCruise
            || *self == NavMode::BurnAndDrift
            || *self == NavMode::BurnAndCruiseAndDrift
    }

    fn is_cruise_mode(&self) -> bool {
        *self == NavMode::Cruise
            || *self == NavMode::BurnAndCruise
            || *self == NavMode::CruiseAndDrift
            || *self == NavMode::BurnAndCruiseAndDrift
    }

    fn is_drift_mode(&self) -> bool {
        *self == NavMode::Drift
            || *self == NavMode::CruiseAndDrift
            || *self == NavMode::BurnAndDrift
            || *self == NavMode::BurnAndCruiseAndDrift
    }
}
