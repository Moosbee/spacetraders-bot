use space_traders_client::models;

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct ModuleState {
    pub modules: Vec<models::ship_module::Symbol>,
}

impl ModuleState {
    pub fn update(&mut self, modules: &[models::ShipModule]) {
        self.modules = modules.iter().map(|m| m.symbol).collect();
    }

    pub fn can_cargo(&self) -> bool {
        self.modules.iter().any(|m| {
            m == &models::ship_module::Symbol::CargoHoldI
                || m == &models::ship_module::Symbol::CargoHoldIi
                || m == &models::ship_module::Symbol::CargoHoldIii
        })
    }
}
