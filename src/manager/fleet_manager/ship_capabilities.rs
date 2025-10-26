use itertools::Itertools;
use space_traders_client::models;

pub struct ShipCapabilities {
    cargo: i32,
    fuel: i32,
    survey: i32,
    sensor: i32,
    extractor: i32,
    siphon: i32,
    warp_drive: i32,
}

impl ShipCapabilities {
    pub fn can_assign(ship_clone: &ship::MyShip, assignment: &database::ShipAssignment) -> bool {
        let capabilities = Self::get_capabilities(ship_clone);

        capabilities.cargo >= assignment.cargo_min
            && capabilities.fuel >= assignment.range_min
            && capabilities.survey >= if assignment.survey { 1 } else { 0 }
            && capabilities.extractor >= if assignment.extractor { 1 } else { 0 }
            && capabilities.siphon >= if assignment.siphon { 1 } else { 0 }
            && capabilities.warp_drive >= if assignment.warp_drive { 1 } else { 0 }
    }

    fn get_capabilities(ship_clone: &ship::MyShip) -> ShipCapabilities {
        let cargo = Self::cargo_from_modules(&ship_clone.modules.modules);
        let fuel = ship_clone.fuel.capacity;
        let survey = Self::survey_from_mounts(&ship_clone.mounts.mounts);
        let extractor = Self::extractor_from_mounts_and_modules(
            &ship_clone.modules.modules,
            &ship_clone.mounts.mounts,
        );
        let siphon = Self::siphon_from_mounts_and_modules(
            &ship_clone.modules.modules,
            &ship_clone.mounts.mounts,
        );
        let warp_drive = Self::warp_drive_from_modules(&ship_clone.modules.modules);
        let sensor = Self::sensor_from_mounts(&ship_clone.mounts.mounts);

        ShipCapabilities {
            cargo,
            fuel,
            survey,
            extractor,
            siphon,
            warp_drive,
            sensor,
        }
    }

    fn cargo_from_modules(modules: &[models::ship_module::Symbol]) -> i32 {
        modules
            .iter()
            .map(|f| match f {
                models::ship_module::Symbol::CargoHoldI => 15,
                models::ship_module::Symbol::CargoHoldIi => 40,
                models::ship_module::Symbol::CargoHoldIii => 75,
                _ => 0,
            })
            .sum()
    }

    fn survey_from_mounts(mounts: &[models::ship_mount::Symbol]) -> i32 {
        mounts
            .iter()
            .map(|f| match f {
                models::ship_mount::Symbol::SurveyorI => 1,
                models::ship_mount::Symbol::SurveyorIi => 2,
                models::ship_mount::Symbol::SurveyorIii => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    fn sensor_from_mounts(mounts: &[models::ship_mount::Symbol]) -> i32 {
        mounts
            .iter()
            .map(|f| match f {
                models::ship_mount::Symbol::SensorArrayI => 1,
                models::ship_mount::Symbol::SensorArrayIi => 2,
                models::ship_mount::Symbol::SensorArrayIii => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    fn extractor_from_mounts_and_modules(
        modules: &[models::ship_module::Symbol],
        mounts: &[models::ship_mount::Symbol],
    ) -> i32 {
        let mount = mounts
            .iter()
            .map(|f| match f {
                models::ship_mount::Symbol::MiningLaserI => 1,
                models::ship_mount::Symbol::MiningLaserIi => 2,
                models::ship_mount::Symbol::MiningLaserIii => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        let has_processor = modules
            .iter()
            .contains(&models::ship_module::Symbol::MineralProcessorI);

        if mount > 0 && has_processor {
            mount
        } else {
            0
        }
    }

    fn siphon_from_mounts_and_modules(
        modules: &[models::ship_module::Symbol],
        mounts: &[models::ship_mount::Symbol],
    ) -> i32 {
        let mount = mounts
            .iter()
            .map(|f| match f {
                models::ship_mount::Symbol::GasSiphonI => 1,
                models::ship_mount::Symbol::GasSiphonIi => 2,
                models::ship_mount::Symbol::GasSiphonIii => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        let has_processor = modules
            .iter()
            .contains(&models::ship_module::Symbol::GasProcessorI);

        if mount > 0 && has_processor {
            mount
        } else {
            0
        }
    }

    fn warp_drive_from_modules(modules: &[models::ship_module::Symbol]) -> i32 {
        modules
            .iter()
            .map(|f| match f {
                models::ship_module::Symbol::WarpDriveI => 1,
                models::ship_module::Symbol::WarpDriveIi => 2,
                models::ship_module::Symbol::WarpDriveIii => 3,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }
}
