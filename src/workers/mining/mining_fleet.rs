use std::{sync::Arc, time::Duration};

use anyhow::Ok;
use futures::FutureExt;
use log::{debug, error, info};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{self, Role},
    sql::ShipInfo,
    workers::mining::m_types::MiningShipAssignment,
};

use super::{
    extraction_processor::ExtractionProcessor, mining_manager::MiningManager,
    transport_processor::TransportProcessor,
};

#[derive(Debug, Clone)]
pub struct MiningFleet {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<MiningManager>,
}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: crate::workers::types::ConductorContext) -> Box<Self> {
        let cancellation_token = CancellationToken::new();
        let mining_places = Arc::new(MiningManager::new());
        Box::new(MiningFleet {
            context: _context.clone(),

            cancellation_token,
            mining_places,
        })
    }

    /// Main mining worker entry point
    async fn run_mining_worker(&self) -> anyhow::Result<()> {
        if !CONFIG.mining.active {
            info!("Mining workers not active, exiting");
            return Ok(());
        }

        // Optional initial sleep to stagger worker start
        sleep(Duration::from_millis(CONFIG.mining.start_sleep_duration)).await;

        let ships = self.get_mining_ships().await;
        self.assign_ships(&ships).await;

        let handles = self.spawn_mining_ship_workers(ships)?;

        let future_handes = handles
            .into_iter()
            .map(|f| {
                f.0.then(|result| async move {
                    if let Err(e) = &result {
                        error!("Lel Error: {}", e);
                    }
                    match &result {
                        Result::Ok(result) => {
                            if let Err(e) = result {
                                error!(
                                    "We got Mining Error: {} {:?} {:?} {:?}",
                                    e,
                                    e.backtrace(),
                                    e.source(),
                                    e.root_cause()
                                );
                            }
                        }
                        _ => (),
                    }
                    result
                })
            })
            .collect::<Vec<_>>();

        let _ergs = futures::future::join_all(future_handes).await;

        info!("Mining workers completed");
        Ok(())
    }

    /// Spawn individual mining ship workers with error handling
    fn spawn_mining_ship_workers(
        &self,
        ships: Vec<String>,
    ) -> anyhow::Result<
        Vec<(
            tokio::task::JoinHandle<anyhow::Result<()>>,
            CancellationToken,
        )>,
    > {
        let mut handles = Vec::new();

        for ship in ships {
            let child_stopper = self.cancellation_token.child_token();
            let fleet = self.with_cancel(child_stopper.clone());

            let handle = tokio::spawn(async move { fleet.run_mining_ship_worker(ship).await });

            handles.push((handle, child_stopper));
        }

        Ok(handles)
    }

    async fn get_mining_ships(&self) -> Vec<String> {
        let ships = ShipInfo::get_by_role(
            &self.context.database_pool,
            &crate::sql::ShipInfoRole::Mining,
        )
        .await
        .unwrap();
        ships.iter().map(|s| s.symbol.clone()).collect()
    }

    async fn run_mining_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut guard = self.context.ship_manager.get_mut(&ship_symbol).await;
        let ship = guard.value_mut().expect("Failed to get mining ship");

        if let Role::Mining(assignment) = ship.role {
            match assignment {
                MiningShipAssignment::Extractor => self.run_extractor_ship_worker(ship).await?,
                MiningShipAssignment::Transporter => self.run_transporter_ship_worker(ship).await?,
                MiningShipAssignment::Siphoner => self.run_siphoned_ship_worker(ship).await?,
                MiningShipAssignment::Surveyor => self.run_surveyor_ship_worker(ship).await?,
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn run_extractor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        let extraction_processor = ExtractionProcessor::new(
            self.context.clone(),
            self.cancellation_token.clone(),
            self.mining_places.clone(),
        );
        extraction_processor.run_ship_worker(ship, false).await
    }
    async fn run_siphoned_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        let extraction_processor = ExtractionProcessor::new(
            self.context.clone(),
            self.cancellation_token.clone(),
            self.mining_places.clone(),
        );
        extraction_processor.run_ship_worker(ship, true).await
    }

    async fn run_transporter_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        let transport_processor = TransportProcessor::new(
            self.context.clone(),
            self.cancellation_token.clone(),
            self.mining_places.clone(),
        );
        transport_processor.run_transporter_ship_worker(ship).await
    }

    async fn run_surveyor_ship_worker(&self, _ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }

    fn with_cancel(&self, cancellation_token: CancellationToken) -> MiningFleet {
        MiningFleet {
            cancellation_token: cancellation_token,
            mining_places: Arc::clone(&self.mining_places),
            ..self.clone()
        }
    }

    async fn assign_ships(&self, ships: &Vec<String>) {
        for ship_name in ships {
            let mut guard = self.context.ship_manager.get_mut(ship_name).await;
            let ship = guard.value_mut().unwrap();
            let ship_capabilities = self.analyze_ship_capabilities(ship);

            ship.role = match ship_capabilities {
                ShipCapabilities {
                    can_extract: true,
                    can_cargo: true,
                    ..
                } => Role::Mining(MiningShipAssignment::Extractor),

                ShipCapabilities {
                    can_extract: false,
                    can_siphon: true,
                    can_cargo: true,
                    ..
                } => Role::Mining(MiningShipAssignment::Siphoner),

                ShipCapabilities {
                    can_survey: true, ..
                } => Role::Mining(MiningShipAssignment::Surveyor),

                ShipCapabilities {
                    can_cargo: true, ..
                } => Role::Mining(MiningShipAssignment::Transporter),

                _ => Role::Mining(MiningShipAssignment::Useless),
            };

            debug!("Assigning role {:?} to ship {}", ship.role, ship.symbol);

            ship.notify().await;
        }
    }
    fn analyze_ship_capabilities(&self, ship: &ship::MyShip) -> ShipCapabilities {
        ShipCapabilities {
            can_extract: ship.mounts.can_extract(),
            can_siphon: ship.mounts.can_siphon(),
            can_survey: ship.mounts.can_survey(),
            can_cargo: ship.cargo.capacity > 0,
        }
    }
}

#[derive(Debug, Clone)]
struct ShipCapabilities {
    can_extract: bool,
    can_siphon: bool,
    can_survey: bool,
    can_cargo: bool,
}

impl crate::workers::types::Conductor for MiningFleet {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_mining_worker().await })
    }

    fn get_name(&self) -> String {
        "MiningFleet".to_string()
    }
    fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}
