use std::time::Duration;

use anyhow::Ok;
use log::info;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{self, Role},
    workers::mining::m_types::MiningShipAssignment,
};

pub struct MiningFleet {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: crate::workers::types::ConductorContext) -> Box<Self> {
        Box::new(MiningFleet {
            context: _context,
            cancellation_token: CancellationToken::new(),
        })
    }

    async fn run_mining_worker(&self) -> anyhow::Result<()> {
        info!("Starting mining workers");

        if !CONFIG.mining.active {
            info!("mining workers not active, exiting");
            return Ok(());
        }

        tokio::select! {
        _ = self.cancellation_token.cancelled() => {
          info!("Agent scrapping cancelled");
          0},
        _ =  sleep(Duration::from_millis(CONFIG.mining.start_sleep_duration)) => {1}
        };

        let ships = self.get_mining_ships();

        self.assign_ships(ships).await;

        info!("mining workers done");

        Ok(())
    }

    fn get_mining_ships(&self) -> Vec<String> {
        self.context
            .ship_roles
            .iter()
            .filter(|(_, role)| {
                if let ship::Role::Mining(_) = role {
                    true
                } else {
                    false
                }
            })
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }

    async fn run_mining_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut ship = self
            .context
            .ship_manager
            .get_ship_mut(&ship_symbol)
            .unwrap();

        if let Role::Mining(assignment) = ship.role {
            match assignment {
                MiningShipAssignment::Extractor => {
                    self.run_extractor_ship_worker(&mut ship).await?
                }
                MiningShipAssignment::Transporter => {
                    self.run_transporter_ship_worker(&mut ship).await?
                }
                MiningShipAssignment::Siphoner => self.run_siphoned_ship_worker(&mut ship).await?,
                MiningShipAssignment::Surveyor => self.run_surveyor_ship_worker(&mut ship).await?,
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn run_transporter_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }
    async fn run_extractor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }
    async fn run_siphoned_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }
    async fn run_surveyor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }

    async fn assign_ships(&self, ships: Vec<String>) {
        for ship_name in ships.iter() {
            let mut ship = self.context.ship_manager.get_ship_mut(ship_name).unwrap();
            let can_extract: bool = ship.mounts.can_extract();
            let can_siphon: bool = ship.mounts.can_siphon();
            let can_survey: bool = ship.mounts.can_survey();
            let can_cargo: bool = ship.cargo.capacity > 0;

            if can_extract && can_siphon && can_survey && can_cargo {
                println!("{} All capabilities are available.", ship_name);
            } else if can_extract && can_siphon && can_survey && !can_cargo {
                println!(
                    "{} Can extract, siphon, survey, but no cargo capacity.",
                    ship_name
                );
            } else if can_extract && can_siphon && !can_survey && can_cargo {
                println!(
                    "{} Can extract, siphon, and has cargo capacity, but cannot survey.",
                    ship_name
                );
            } else if can_extract && can_siphon && !can_survey && !can_cargo {
                println!(
                    "{} Can extract and siphon, but cannot survey or carry cargo.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Useless);
            } else if can_extract && !can_siphon && can_survey && can_cargo {
                println!(
                    "{} Can extract and survey, and has cargo capacity, but cannot siphon.",
                    ship_name
                );
            } else if can_extract && !can_siphon && can_survey && !can_cargo {
                println!(
                    "{} Can extract and survey, but cannot siphon or carry cargo.",
                    ship_name
                );
            } else if can_extract && !can_siphon && !can_survey && can_cargo {
                println!(
                    "{} Can extract and has cargo capacity, but cannot siphon or survey.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Extractor);
            } else if can_extract && !can_siphon && !can_survey && !can_cargo {
                println!(
                    "{} Can extract, but cannot siphon, survey, or carry cargo.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Useless);
            } else if !can_extract && can_siphon && can_survey && can_cargo {
                println!(
                    "{} Can siphon and survey, and has cargo capacity, but cannot extract.",
                    ship_name
                );
            } else if !can_extract && can_siphon && can_survey && !can_cargo {
                println!(
                    "{} Can siphon and survey, but cannot extract or carry cargo.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Useless);
            } else if !can_extract && can_siphon && !can_survey && can_cargo {
                println!(
                    "{} Can siphon and has cargo capacity, but cannot extract or survey.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Siphoner);
            } else if !can_extract && can_siphon && !can_survey && !can_cargo {
                println!(
                    "{} Can siphon, but cannot extract, survey, or carry cargo.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Useless);
            } else if !can_extract && !can_siphon && can_survey && can_cargo {
                println!(
                    "{} Can survey and has cargo capacity, but cannot extract or siphon.",
                    ship_name
                );
            } else if !can_extract && !can_siphon && can_survey && !can_cargo {
                println!(
                    "{} Can survey, but cannot extract, siphon, or carry cargo.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Surveyor);
            } else if !can_extract && !can_siphon && !can_survey && can_cargo {
                println!(
                    "{} Has cargo capacity, but cannot extract, siphon, or survey.",
                    ship_name
                );
                ship.role = Role::Mining(MiningShipAssignment::Transporter);
            } else if !can_extract && !can_siphon && !can_survey && !can_cargo {
                println!("{} No capabilities are available.", ship_name);
            }

            ship.notify().await;
        }
    }
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
