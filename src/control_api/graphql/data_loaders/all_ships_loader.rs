use std::{collections::HashMap, sync::Arc};

use ship::status::ShipStatus;
use tracing::instrument;

pub struct AllShipLoader(crate::utils::ConductorContext);

impl AllShipLoader {
    pub fn new(context: crate::utils::ConductorContext) -> Self {
        Self(context)
    }
}

impl async_graphql::dataloader::Loader<()> for AllShipLoader {
    type Value = HashMap<String, ship::RustShip<ShipStatus, ship::Immutable>>;
    type Error = Arc<crate::error::Error>;

    #[instrument(level = "trace", skip(self, _keys))]
    async fn load(
        &self,
        _keys: &[()],
    ) -> std::result::Result<HashMap<(), Self::Value>, Self::Error> {
        // let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let mut map = HashMap::new();
        let all_ships = self.0.ship_manager.get_all_clone().await;
        map.insert((), all_ships);
        Ok(map)
    }
}
