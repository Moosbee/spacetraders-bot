use space_traders_client::models;

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct ModuleState {
    pub modules: Vec<models::ship_module::Symbol>,
}

impl ModuleState {
    pub fn update(&mut self, modules: &[models::ShipModule]) {
        self.modules = modules.iter().map(|m| m.symbol).collect();
    }
}
