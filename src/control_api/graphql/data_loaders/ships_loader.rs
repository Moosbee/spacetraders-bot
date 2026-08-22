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

pub struct ShipsPerSystemLoader(crate::utils::ConductorContext);

impl ShipsPerSystemLoader {
    pub fn new(context: crate::utils::ConductorContext) -> Self {
        Self(context)
    }
}

impl async_graphql::dataloader::Loader<String> for ShipsPerSystemLoader {
    type Value = Vec<ship::RustShip<ShipStatus, ship::Immutable>>;
    type Error = Arc<crate::error::Error>;

    #[instrument(level = "trace", skip(self, _keys))]
    async fn load(
        &self,
        _keys: &[String],
    ) -> std::result::Result<HashMap<String, Self::Value>, Self::Error> {
        // let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let mut map = HashMap::new();
        let all_ships = self.0.ship_manager.get_all_clone().await;

        for ship in all_ships.into_values() {
            map.entry(ship.nav.system_symbol.clone())
                .or_insert_with(Vec::new)
                .push(ship);
        }

        Ok(map)
    }
}

pub struct ShipsAssignmentLoader(crate::utils::ConductorContext);

impl ShipsAssignmentLoader {
    pub fn new(context: crate::utils::ConductorContext) -> Self {
        Self(context)
    }
}

impl async_graphql::dataloader::Loader<i64> for ShipsAssignmentLoader {
    type Value = Vec<ship::RustShip<ShipStatus, ship::Immutable>>;
    type Error = Arc<crate::error::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(
        &self,
        keys: &[i64],
    ) -> std::result::Result<HashMap<i64, Self::Value>, Self::Error> {
        // let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let mut map = HashMap::new();
        let all_ships = self.0.ship_manager.get_all_clone().await;

        for ship in all_ships.into_values() {
            let assignment_id = ship.status.assignment_id.unwrap_or(0);
            let temp_assignment_id = ship.status.temp_assignment_id.unwrap_or(0);
            if keys.contains(&assignment_id) || keys.contains(&temp_assignment_id) {
                map.entry(assignment_id)
                    .or_insert_with(Vec::new)
                    .push(ship.clone());
                map.entry(temp_assignment_id)
                    .or_insert_with(Vec::new)
                    .push(ship);
            }
        }

        Ok(map)
    }
}
