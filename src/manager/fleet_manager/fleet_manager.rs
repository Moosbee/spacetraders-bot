use std::{collections::HashMap, sync::Arc};

use database::{DatabaseConnector, ShipInfo};
use space_traders_client::models;
use tracing::debug;
use utils::get_system_symbol;

use crate::{
    error::{Error, Result},
    manager::{
        chart_manager::ChartManager, fleet_manager::message::RequiredShips,
        scrapping_manager::ScrappingManager, trade_manager::TradeManager, Manager,
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
        name = "spacetraders::manager::fleet_manager_worker",
        skip(self)
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
            super::message::FleetMessage::ShipArrived {
                waypoint_symbol,
                ship_symbol,
            } => {
                self.handle_ship_arrived(&waypoint_symbol, &ship_symbol)
                    .await?
            }
            super::message::FleetMessage::GetTransfer {
                ship_clone,
                callback,
            } => {
                let erg = self.handle_get_transfer(ship_clone).await?;
                callback.send(erg).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::handle_get_transfer",
        fields(ship_symbol = %ship_clone.symbol),
        skip(self, ship_clone),
    )]
    async fn handle_get_transfer(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<database::ShipTransfer> {
        debug!("Requesting next transfer for ship: {}", ship_clone.symbol);
        let db_transfers =
            database::ShipTransfer::get_unfinished(&self.context.database_pool).await?;
        let ship_transfer = db_transfers
            .iter()
            .find(|f| f.ship_symbol == ship_clone.symbol)
            .cloned();
        debug!("Found ship transfer: {:?}", ship_transfer);
        if let Some(ship_transfer) = ship_transfer {
            return Ok(ship_transfer);
        }
        todo!()
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::handle_scrapper_at_shipyard",
        skip(self)
    )]
    async fn handle_scrapper_at_shipyard(
        &mut self,
        waypoint_symbol: &str,
        _ship_symbol: &str,
    ) -> Result<()> {
        // return Ok(());

        let ship_puchase_stop = { self.context.config.read().await.ship_purchase_stop };

        if ship_puchase_stop {
            return Ok(());
        }

        let pathfinder = Arc::new(ship::autopilot::jump_gate_nav::JumpPathfinder::new(
            ship::autopilot::jump_gate_nav::generate_all_connections(&self.context.database_pool)
                .await?
                .into_iter()
                .filter(|c| !c.under_construction_a && !c.under_construction_b)
                .collect::<Vec<_>>(),
        ));

        let antimatter_prices: Arc<HashMap<String, i32>> = Arc::new(
            database::MarketTradeGood::get_last_by_symbol(
                &self.context.database_pool,
                &models::TradeSymbol::Antimatter,
            )
            .await?
            .iter()
            .map(|t| (t.waypoint_symbol.clone(), t.purchase_price))
            .collect(),
        );

        let count = { self.context.config.read().await.ship_purchase_amount };

        debug!("Buying ships at {}", waypoint_symbol);

        let all_ships = Arc::new(
            self.context
                .ship_manager
                .get_all_clone()
                .await
                .into_values()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );

        let all_systems_hashmap =
            Arc::new(database::Waypoint::get_hash_map(&self.context.database_pool).await?);
        let all_connections =
            database::JumpGateConnection::get_all(&self.context.database_pool).await?;

        let mut connection_hash_map: HashMap<String, Vec<database::JumpGateConnection>> =
            HashMap::new();

        for connection in all_connections {
            let entry = connection_hash_map
                .entry(connection.from.clone())
                .or_default();
            entry.push(connection);
        }

        let connection_hash_map: Arc<HashMap<String, Vec<database::JumpGateConnection>>> =
            Arc::new(connection_hash_map);

        for _ in 0..count {
            if self.needs_update() {
                self.update_required_ships(
                    all_ships.clone(),
                    all_systems_hashmap.clone(),
                    connection_hash_map.clone(),
                )
                .await?
            }

            let locations = self
                .get_purchase_locations(pathfinder.clone(), antimatter_prices.clone())
                .await?;

            debug!("Found {} purchase locations", locations.len());

            let wps_ships = locations.get(waypoint_symbol).cloned().unwrap_or_default();

            debug!(
                "Found {} ships to purchase at {}",
                wps_ships.len(),
                waypoint_symbol
            );

            if wps_ships.is_empty() {
                break;
            }

            let next_to_buy = wps_ships.first();

            if let Some((ship_type, predicted_total_price, priority, ship_role, system_symbol)) =
                next_to_buy
            {
                let reservation = self
                    .context
                    .budget_manager
                    .reserve_funds_with_remain(
                        &self.context.database_pool,
                        *predicted_total_price as i64,
                        *priority as i64,
                    )
                    .await;

                let reservation = if let Err(e) = reservation {
                    if let crate::error::Error::NotEnoughFunds {
                        remaining_funds,
                        required_funds,
                    } = e
                    {
                        debug!(
                            "Not enough funds to purchase ship: {}. Remaining: {}, Required: {}",
                            ship_type, remaining_funds, required_funds
                        );
                        continue;
                    } else {
                        return Err(e);
                    }
                } else {
                    reservation.unwrap()
                };

                debug!(
                    "Found ship to purchase at {} with price {} and priority {:?}: {:?} for system {}",
                    waypoint_symbol, predicted_total_price, priority, ship_type, system_symbol
                );

                let ship = self
                    .purchase_ship(
                        ship_type,
                        waypoint_symbol,
                        database::ShipInfoRole::Transfer,
                        true,
                        |ship| {
                            ship.status = ship::ShipStatus::Transfer {
                                id: None,
                                system_symbol: Some(system_symbol.clone()),
                                role: Some(*ship_role),
                            };
                        },
                    )
                    .await?;

                self.context
                    .budget_manager
                    .use_reservation(
                        &self.context.database_pool,
                        reservation.id,
                        ship.1.price as i64,
                    )
                    .await?;

                let transfer = database::ShipTransfer {
                    id: 0,
                    ship_symbol: ship.0.symbol,
                    system_symbol: system_symbol.clone(),
                    role: *ship_role,
                    finished: false,
                    reserved_fund: Some(reservation.id),
                };
                database::ShipTransfer::insert_new(&self.context.database_pool, &transfer).await?;
                self.is_dirty = true;
            }
        }

        Ok(())
    }

    async fn get_purchase_locations(
        &self,
        pathfinder: Arc<ship::autopilot::jump_gate_nav::JumpPathfinder>,
        antimatter_prices: Arc<HashMap<String, i32>>,
    ) -> Result<
        HashMap<
            String,
            Vec<(
                models::ShipType,
                i32, // predicted total price
                super::message::Priority,
                database::ShipInfoRole,
                String,
            )>,
        >,
    > {
        let expand = { self.context.config.read().await.expand };

        let shipyards = database::ShipyardShip::get_last(&self.context.database_pool).await?;

        let budget = self.context.budget_manager.get_spendable_funds().await;

        let required_ships = self.required_ships.ships.clone();

        let erg = tokio::task::spawn_blocking(move || {
            let mut ships = Vec::new();

            for system in required_ships {
                for ship in system.1.iter() {
                    // Stand-in for compute-heavy work or using synchronous APIs
                    // Pass ownership of the value back to the asynchronous context

                    let best_shipyards = Self::get_best_shipyard(
                        &system.0,
                        (ship.0, ship.1, ship.2, ship.3),
                        budget,
                        &shipyards,
                        &pathfinder,
                        &antimatter_prices,
                    )?;

                    let len = best_shipyards.len();

                    let mut min_price = i32::MAX;

                    for (
                        index,
                        (shipyard_symbol, ship_type, purchase_price, priority, ship_role),
                    ) in best_shipyards.into_iter().enumerate()
                    {
                        if purchase_price < min_price {
                            min_price = purchase_price;
                        }
                        ships.push((
                            shipyard_symbol,
                            ship_type,
                            purchase_price,
                            priority,
                            ship_role,
                            system.0.clone(),
                            index,
                            len,
                            min_price,
                        ));
                    }
                }
            }

            ships.sort_by(|a, b| b.3.cmp(&a.3));

            let mut map: HashMap<
                String,
                Vec<(
                    models::ShipType,
                    i32,
                    super::message::Priority,
                    database::ShipInfoRole,
                    String,
                )>,
            > = HashMap::new();

            for (
                shipyard_symbol,
                ship_type,
                purchase_price,
                priority,
                ship_role,
                system_symbol,
                index,
                _len,
                min_price,
            ) in ships
            {
                if (purchase_price as f32) > ((min_price as f32) * 1.5) {
                    continue;
                }

                if ship_role == database::ShipInfoRole::Charter && !expand {
                    continue;
                }

                if !map.contains_key(&shipyard_symbol) {
                    map.insert(shipyard_symbol.clone(), Vec::new());
                }
                map.get_mut(&shipyard_symbol).unwrap().push((
                    ship_type,
                    purchase_price,
                    priority,
                    ship_role,
                    system_symbol,
                ));
            }
            crate::error::Result::Ok(map)
        })
        .await
        .map_err(|e| crate::error::Error::General(e.to_string()))??;

        Ok(erg)
    }

    fn get_best_shipyard(
        system_symbol: &str,
        ship: (
            super::message::RequestedShipType,
            super::message::Priority,
            super::message::Budget,
            database::ShipInfoRole,
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
            database::ShipInfoRole,
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
            .filter(|f| f.2 > (ship.3 as i32))
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
                    ship.3,
                )
            })
            .collect::<Vec<_>>();

        Ok(first)
    }

    fn needs_update(&self) -> bool {
        self.is_dirty || self.last_update.elapsed().as_secs() > 5 * 60
    }

    async fn update_required_ships(
        &mut self,
        all_ships: Arc<Box<[ship::MyShip]>>,
        all_systems_hashmap: Arc<HashMap<String, HashMap<String, database::Waypoint>>>,
        connection_hash_map: Arc<HashMap<String, Vec<database::JumpGateConnection>>>,
    ) -> Result<()> {
        let config = { self.context.config.read().await.clone() };

        let mining_ships = self.context.mining_manager.get_ships(&self.context).await?;

        let construction_ships = self
            .context
            .construction_manager
            .get_ships(&self.context)
            .await?;

        let contract_ships = self
            .context
            .contract_manager
            .get_ships(&self.context)
            .await?;

        let (scrap_ships, trading_ships, chart_ships) = tokio::task::spawn_blocking(move || {
            // Stand-in for compute-heavy work or using synchronous APIs
            // Pass ownership of the value back to the asynchronous context

            let scrap_ships =
                ScrappingManager::get_required_ships(&all_ships, &all_systems_hashmap)?;

            let trading_ships = TradeManager::get_required_ships(
                &all_ships,
                &all_systems_hashmap,
                config.markets_per_ship,
            )?;

            let chart_ships = ChartManager::get_required_ships(
                &all_ships,
                &all_systems_hashmap,
                &connection_hash_map,
            )?;

            crate::error::Result::Ok((scrap_ships, trading_ships, chart_ships))
        })
        .await
        .map_err(|e| crate::error::Error::General(e.to_string()))??;

        self.required_ships = scrap_ships
            + trading_ships
            + mining_ships
            + construction_ships
            + chart_ships
            + contract_ships;

        self.last_update = std::time::Instant::now();

        self.is_dirty = false;

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::handle_ship_arrived",
        skip(self)
    )]
    async fn handle_ship_arrived(
        &mut self,
        _waypoint_symbol: &str,
        ship_symbol: &str,
    ) -> Result<()> {
        self.is_dirty = true;

        let unfinished_ships = database::ShipTransfer::get_unfinished(&self.context.database_pool)
            .await?
            .into_iter()
            .filter(|f| f.ship_symbol == ship_symbol)
            .collect::<Vec<_>>();

        if unfinished_ships.len() == 1 {
            let mut ship_transfer = unfinished_ships[0].clone();
            ship_transfer.finished = true;

            database::ShipTransfer::insert(&self.context.database_pool, &ship_transfer).await?;

            if let Some(reservation_id) = ship_transfer.reserved_fund {
                self.context
                    .budget_manager
                    .complete_reservation(&self.context.database_pool, reservation_id)
                    .await?;
            }
        } else {
            return Err(Error::General(format!(
                "Found multiple ship transfers for ship {}",
                ship_symbol
            )));
        }

        Ok(())
    }

    pub async fn purchase_ship(
        &self,
        ship_type: &models::ShipType,
        waypoint_symbol: &str,
        role: database::ShipInfoRole,
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

        database::ShipyardTransaction::insert(&self.context.database_pool, &transaction).await?;

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

        ship_i.role = role;
        ship_i.active = active;

        let ship_info = ship_i
            .apply_from_db(self.context.database_pool.clone())
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
