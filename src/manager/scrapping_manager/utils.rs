mod agent_scrapper;
mod jump_gate_scrapper;
mod market_scrapper;
mod shipyard_scrapper;
mod system_scrapper;

pub use agent_scrapper::update_all_agents;
pub use jump_gate_scrapper::get_all_jump_gates;
pub use jump_gate_scrapper::update_jump_gate;
pub use jump_gate_scrapper::update_jump_gates;
pub use market_scrapper::get_all_markets;
pub use market_scrapper::update_market;
pub use market_scrapper::update_markets;
pub use shipyard_scrapper::update_shipyard;
pub use system_scrapper::update_all_systems;
pub use system_scrapper::update_system;
