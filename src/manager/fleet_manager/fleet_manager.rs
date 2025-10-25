use std::collections::HashMap;

use database::{DatabaseConnector, ShipInfo};
use space_traders_client::models;
use tracing::debug;
use utils::get_system_symbol;

use crate::{
    error::Result,
    manager::{
        fleet_manager::message::RequiredShips, Manager,
    },
    utils::ConductorContext,
};

use super::{message::FleetManagerMessage, messanger::FleetManagerMessanger};

pub struct FleetManager {
    cancel_token: tokio_util::sync::CancellationToken,
    receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    context: ConductorContext,
    required_ships: RequiredShips,
    last_update: std::time::Instant,
    is_dirty: bool,
}

impl FleetManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<FleetManagerMessage>,
        FleetManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(24);
        debug!("Created FleetManager channel");

        (receiver, FleetManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    ) -> Self {
        debug!("Creating new FleetManager");
        Self {
            cancel_token,
            context,
            receiver,
            required_ships: RequiredShips::default(),
            last_update: std::time::Instant::now(),
            is_dirty: true,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::fleet_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_fleet_worker(&mut self) -> std::result::Result<(), crate::error::Error> {
        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            debug!("Received FleetManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_fleet_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_fleet_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_fleet_message(&mut self, message: super::message::FleetMessage) -> Result<()> {
        match message {
            super::message::FleetMessage::ScrapperAtShipyard {
                waypoint_symbol,
                ship_symbol,
                callback,
            } => {
                let erg = self
                    .handle_scrapper_at_shipyard(&waypoint_symbol, &ship_symbol)
                    .await;
                callback.send(ship_symbol).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
                erg?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_scrapper_at_shipyard",
        skip(self)
    )]
    async fn handle_scrapper_at_shipyard(
        &mut self,
        waypoint_symbol: &str,
        _ship_symbol: &str,
    ) -> Result<()> {
        return Ok(());
    }

    fn get_best_shipyard(
        system_symbol: &str,
        ship: (
            super::message::RequestedShipType,
            super::message::Priority,
            super::message::Budget,
        ),
        available_budget: i64,
        shipyards: &[database::ShipyardShip],
        pathfinder: &ship::autopilot::jump_gate_nav::JumpPathfinder,
        antimatter_prices: &HashMap<String, i32>,
    ) -> Result<
        Vec<(
            String,
            models::ShipType,
            i32, // predicted total price
            super::message::Priority,
        )>,
    > {
        let mut ships: Vec<(
            &database::ShipyardShip,
            i32, /*travel cost */
            i32, /* total price */
        )> = shipyards
            .iter()
            .filter(|f| match ship.0 {
                super::message::RequestedShipType::Scrapper => true,
                super::message::RequestedShipType::Explorer => {
                    f.ship_type == models::ShipType::Explorer
                }
                super::message::RequestedShipType::Probe => f.ship_type == models::ShipType::Probe,
                super::message::RequestedShipType::Transporter => {
                    let sum: i32 = f
                        .modules
                        .iter()
                        .map(|f| match f {
                            models::ship_module::Symbol::CargoHoldI => 15,
                            models::ship_module::Symbol::CargoHoldIi => 40,
                            models::ship_module::Symbol::CargoHoldIii => 75,
                            _ => 0,
                        })
                        .sum();

                    sum >= 40
                    // sum >= 80
                }
                super::message::RequestedShipType::Mining => {
                    f.mounts.iter().any(|f| {
                        *f == models::ship_mount::Symbol::MiningLaserI
                            || *f == models::ship_mount::Symbol::MiningLaserIi
                            || *f == models::ship_mount::Symbol::MiningLaserIii
                    }) && f
                        .modules
                        .contains(&models::ship_module::Symbol::MineralProcessorI)
                }
                super::message::RequestedShipType::Siphon => {
                    f.mounts.iter().any(|f| {
                        *f == models::ship_mount::Symbol::GasSiphonI
                            || *f == models::ship_mount::Symbol::GasSiphonIi
                            || *f == models::ship_mount::Symbol::GasSiphonIii
                    }) && f
                        .modules
                        .contains(&models::ship_module::Symbol::GasProcessorI)
                }
                super::message::RequestedShipType::Survey => f.mounts.iter().any(|f| {
                    *f == models::ship_mount::Symbol::SurveyorI
                        || *f == models::ship_mount::Symbol::SurveyorIi
                        || *f == models::ship_mount::Symbol::SurveyorIii
                }),
            })
            .filter(|f| f.purchase_price <= (ship.2 as i32))
            .map(|f| {
                let route =
                    pathfinder.find_route(&get_system_symbol(&f.waypoint_symbol), system_symbol);
                let travel_cost = route
                    .iter()
                    .map(|f| antimatter_prices.get(&f.start_system).unwrap_or(&0))
                    .sum::<i32>();

                (f, travel_cost, f.purchase_price + travel_cost)
            })
            .filter(|f| f.1 > (ship.2 as i32))
            .filter(|f| available_budget - (f.2 as i64) > (ship.1 as i64))
            .collect::<Vec<_>>();

        ships.sort_by(|a, b| a.2.cmp(&b.2));

        let first = ships
            .into_iter()
            .map(|f| {
                (
                    f.0.waypoint_symbol.clone(),
                    f.0.ship_type,
                    f.2, // total price
                    ship.1,
                )
            })
            .collect::<Vec<_>>();

        Ok(first)
    }

    pub async fn purchase_ship(
        &self,
        ship_type: &models::ShipType,
        waypoint_symbol: &str,
        active: bool,
        start_fn: impl Fn(&mut ship::MyShip),
    ) -> Result<(ShipInfo, database::ShipyardTransaction)> {
        let purchase_ship_request =
            models::PurchaseShipRequest::new(*ship_type, waypoint_symbol.to_string());

        let resp = self
            .context
            .api
            .purchase_ship(purchase_ship_request)
            .await?;

        self.context
            .budget_manager
            .set_current_funds(resp.data.agent.credits);

        database::Agent::insert(
            &self.context.database_pool,
            &database::Agent::from(*resp.data.agent),
        )
        .await?;

        let transaction = database::ShipyardTransaction::try_from(*resp.data.transaction)?;

        let id =
            database::ShipyardTransaction::insert_new(&self.context.database_pool, &transaction)
                .await?;

        ship::MyShip::update_info_db((*resp.data.ship).clone(), &self.context.database_pool)
            .await?;

        let shipyard = self
            .context
            .api
            .get_shipyard(
                &resp.data.ship.nav.system_symbol,
                &resp.data.ship.nav.waypoint_symbol,
            )
            .await?;

        let mut ship_i =
            ship::MyShip::from_ship(*resp.data.ship, self.context.ship_manager.get_broadcaster());

        ship_i.active = active;

        let ship_info = ship_i
            .apply_from_db_ship(self.context.database_pool.clone(), Some(id))
            .await?;

        start_fn(&mut ship_i);

        ship_i.notify().await;

        ship::ShipManager::add_ship(&self.context.ship_manager, ship_i).await;

        {
            let mut ship_g = self.context.ship_manager.get_mut(&ship_info.symbol).await;
            let ship = ship_g
                .value_mut()
                .ok_or_else(|| crate::error::Error::General("Ship not found".into()))?;
            ship.notify().await;
        }

        crate::manager::scrapping_manager::utils::update_shipyard(
            &self.context.database_pool,
            *shipyard.data,
        )
        .await?;

        self.context.ship_tasks.start_ship(ship_info.clone()).await;

        Ok((ship_info, transaction))
    }
}

impl Manager for FleetManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_fleet_worker().await })
    }

    fn get_name(&self) -> &str {
        "FleetManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
