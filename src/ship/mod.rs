pub mod autopilot;
mod cargo;
mod mining;
mod modules;
mod mounts;
mod nav;
mod ship_manager;
mod ship_models;

// pub use nav::nav_models;
// pub use nav::stats;
pub use ship_manager::ShipManager;
pub use ship_models::MyShip;
pub use ship_models::ShipStatus;
pub use ship_models::ShippingStatus;
pub use ship_models::my_ship_update;
