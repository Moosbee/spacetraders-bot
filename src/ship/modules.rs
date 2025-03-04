use space_traders_client::models;

use super::ship_models::ModuleState;

impl ModuleState {
    pub fn update(&mut self, modules: &Vec<models::ShipModule>) {
        self.modules = modules.iter().map(|m| m.symbol.clone()).collect();
    }
}
