use database::DatabaseConnector;
use log::debug;
use tracing::instrument;
use utils::WaypointCan;

use crate::{
    error::{Error, Result},
    utils::ConductorContext,
};

pub struct TransferPilot {
    context: ConductorContext,
    ship_symbol: String,
}

impl TransferPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            context,
            ship_symbol,
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_transfer", skip(self, pilot), fields(self.ship_symbol = %self.ship_symbol, transfer))]
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next transfer for ship: {:?}", ship.symbol);

        let transfer: database::ShipTransfer = self
            .context
            .fleet_manager
            .get_transfer(ship.clone())
            .await?;

        tracing::Span::current().record("transfer", format!("{:?}", transfer));

        debug!("Next transfer: {:?}", transfer);

        self.start_transfer(ship, transfer.clone()).await?;

        Ok(())
    }

    async fn start_transfer(
        &self,
        ship: &mut ship::MyShip,
        transfer: database::ShipTransfer,
    ) -> std::result::Result<(), Error> {
        ship.status = ship::ShipStatus::Transfer {
            id: Some(transfer.id),
            system_symbol: Some(transfer.system_symbol.clone()),
            role: Some(transfer.role),
        };
        ship.notify().await;

        if ship.nav.system_symbol != transfer.system_symbol {
            let waypoints = database::Waypoint::get_by_system(
                &self.context.database_pool,
                &transfer.system_symbol,
            )
            .await?;
            let jump_gate = waypoints
                .iter()
                .find(|w| w.is_jump_gate())
                .cloned()
                .ok_or(Error::General("No jump gate found".to_string()))?;

            let budget_manager = self.context.budget_manager.clone();

            let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

            ship.nav_to(
                &jump_gate.symbol,
                true,
                database::TransactionReason::None,
                &self.context.database_pool,
                &self.context.api,
                update_funds_fn,
            )
            .await?;
        }

        ship.role = transfer.role;

        ship.status = ship::ShipStatus::Manuel;

        let mut sql_ship =
            database::ShipInfo::get_by_symbol(&self.context.database_pool, &ship.symbol)
                .await?
                .ok_or(Error::General("Ship not found".to_string()))?;

        sql_ship.role = transfer.role;
        database::ShipInfo::insert(&self.context.database_pool, &sql_ship).await?;

        ship.apply_from_db(self.context.database_pool.clone())
            .await?;

        self.context
            .fleet_manager
            .ship_arrived(ship.nav.waypoint_symbol.clone(), ship.symbol.clone())
            .await?;

        ship.notify().await;

        Ok(())
    }
}
