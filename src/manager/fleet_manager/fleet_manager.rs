use std::collections::HashMap;

use database::{DatabaseConnector, ShipInfo};
use log::debug;
use space_traders_client::models;
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

        let pathfinder = ship::autopilot::jump_gate_nav::JumpPathfinder::new(
            ship::autopilot::jump_gate_nav::generate_all_connections(&self.context.database_pool)
                .await?
                .into_iter()
                .filter(|c| !c.under_construction_a && !c.under_construction_b)
                .collect::<Vec<_>>(),
        );

        let antimatter_prices = database::MarketTradeGood::get_last_by_symbol(
            &self.context.database_pool,
            &models::TradeSymbol::Antimatter,
        )
        .await?
        .iter()
        .map(|t| (t.waypoint_symbol.clone(), t.purchase_price))
        .collect();

        let count = { self.context.config.read().await.ship_purchase_amount };

        debug!("Buying ships at {}", waypoint_symbol);

        let all_ships = self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .into_values()
            .collect::<Vec<_>>();

        let all_systems_hashmap: HashMap<String, HashMap<String, database::Waypoint>> =
            database::Waypoint::get_hash_map(&self.context.database_pool).await?;
        let all_connections: Vec<database::JumpGateConnection> =
            database::JumpGateConnection::get_all(&self.context.database_pool).await?;

        let mut connection_hash_map: HashMap<String, Vec<database::JumpGateConnection>> =
            HashMap::new();

        for connection in all_connections {
            let entry = connection_hash_map
                .entry(connection.from.clone())
                .or_default();
            entry.push(connection);
        }

        for _ in 0..count {
            if self.needs_update() {
                self.update_required_ships(&all_ships, &all_systems_hashmap, &connection_hash_map)
                    .await?
            }

            let locations = self
                .get_purchase_locations(&pathfinder, &antimatter_prices)
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

            if let Some((ship_type, predicted_price, priority, ship_role, system_symbol)) =
                next_to_buy
            {
                debug!(
                    "Found ship to purchase at {} with price {} and priority {:?}: {:?} for system {}",
                    waypoint_symbol, predicted_price, priority, ship_type, system_symbol
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

                let transfer = database::ShipTransfer {
                    id: 0,
                    ship_symbol: ship.0.symbol,
                    system_symbol: system_symbol.clone(),
                    role: *ship_role,
                    finished: false,
                };
                database::ShipTransfer::insert_new(&self.context.database_pool, &transfer).await?;
                self.is_dirty = true;
            }
        }

        Ok(())
    }

    async fn get_purchase_locations(
        &self,
        pathfinder: &ship::autopilot::jump_gate_nav::JumpPathfinder,
        antimatter_prices: &HashMap<String, i32>,
    ) -> Result<
        HashMap<
            String,
            Vec<(
                models::ShipType,
                i32,
                super::message::Priority,
                database::ShipInfoRole,
                String,
            )>,
        >,
    > {
        let agent_symbol = { self.context.run_info.read().await.agent_symbol.clone() };

        let expand = { self.context.config.read().await.expand };

        let agent = database::Agent::get_last_by_symbol(&self.context.database_pool, &agent_symbol)
            .await?
            .ok_or(crate::error::Error::General("Agent not found".to_string()))?;
        let shipyards = database::ShipyardShip::get_last(&self.context.database_pool).await?;

        let required_ships = self.required_ships.ships.clone();

        let mut ships = Vec::new();

        for system in required_ships {
            for ship in system.1.iter() {
                let best_shipyards = self
                    .get_best_shipyard(
                        &system.0,
                        (ship.0, ship.1, ship.2, ship.3),
                        &agent,
                        &shipyards,
                        pathfinder,
                        antimatter_prices,
                    )
                    .await?;

                let len = best_shipyards.len();

                let mut min_price = i32::MAX;

                for (index, (shipyard_symbol, ship_type, purchase_price, priority, ship_role)) in
                    best_shipyards.into_iter().enumerate()
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

        Ok(map)
    }

    async fn get_best_shipyard(
        &self,
        system_symbol: &str,
        ship: (
            super::message::RequestedShipType,
            super::message::Priority,
            super::message::Budget,
            database::ShipInfoRole,
        ),
        agent: &database::Agent,
        shipyards: &[database::ShipyardShip],
        pathfinder: &ship::autopilot::jump_gate_nav::JumpPathfinder,
        antimatter_prices: &HashMap<String, i32>,
    ) -> Result<
        Vec<(
            String,
            models::ShipType,
            i32,
            super::message::Priority,
            database::ShipInfoRole,
        )>,
    > {
        let mut ships = shipyards
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

                    sum >= 80
                }
                super::message::RequestedShipType::Mining => {
                    f.mounts.iter().any(|f| {
                        *f == models::ship_mount::Symbol::MiningLaserI
                            || *f == models::ship_mount::Symbol::MiningLaserIi
                            || *f == models::ship_mount::Symbol::MiningLaserIii
                    }) && f
                        .modules
                        .iter()
                        .any(|f| *f == models::ship_module::Symbol::MineralProcessorI)
                }
                super::message::RequestedShipType::Siphon => {
                    f.mounts.iter().any(|f| {
                        *f == models::ship_mount::Symbol::GasSiphonI
                            || *f == models::ship_mount::Symbol::GasSiphonIi
                            || *f == models::ship_mount::Symbol::GasSiphonIii
                    }) && f
                        .modules
                        .iter()
                        .any(|f| *f == models::ship_module::Symbol::GasProcessorI)
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
            .filter(|f| agent.credits - (f.2 as i64) > (ship.1 as i64))
            .collect::<Vec<_>>();

        ships.sort_by(|a, b| a.2.cmp(&b.2));

        let first = ships
            .into_iter()
            .map(|f| {
                (
                    f.0.waypoint_symbol.clone(),
                    f.0.ship_type,
                    f.2,
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
        all_ships: &[ship::MyShip],
        all_systems_hashmap: &HashMap<String, HashMap<String, database::Waypoint>>,
        connection_hash_map: &HashMap<String, Vec<database::JumpGateConnection>>,
    ) -> Result<()> {
        let config = { self.context.config.read().await.clone() };

        let scrap_ships = ScrappingManager::get_required_ships(all_ships, all_systems_hashmap)?;

        let trading_ships = TradeManager::get_required_ships(
            all_ships,
            all_systems_hashmap,
            config.markets_per_ship,
        )?;

        let mining_ships = self.context.mining_manager.get_ships(&self.context).await?;

        let construction_ships = self
            .context
            .construction_manager
            .get_ships(&self.context)
            .await?;

        let chart_ships =
            ChartManager::get_required_ships(all_ships, all_systems_hashmap, connection_hash_map)?;

        let contract_ships = self
            .context
            .contract_manager
            .get_ships(&self.context)
            .await?;

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
            .purchase_ship(Some(purchase_ship_request.clone()))
            .await?;

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
