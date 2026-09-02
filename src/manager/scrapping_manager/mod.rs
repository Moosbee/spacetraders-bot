mod message;
mod messanger;
pub mod priority_calculator;
mod scrapping_manager;
pub mod utils;

pub use message::ScrapResponse;
pub use messanger::ScrappingManagerMessanger;
pub use messanger::SystemScrapperState;
pub use scrapping_manager::ScrappingManager;
pub use scrapping_manager::ScrappingManagerReceiver;
