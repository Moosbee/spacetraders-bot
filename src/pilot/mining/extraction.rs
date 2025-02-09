use std::sync::{atomic::AtomicI32, Arc};

use crate::{
    config::CONFIG, error::Result, manager::mining_manager::ActionType, ship,
    sql::TransactionReason, types::safely_get_map, workers::types::ConductorContext,
};

pub struct ExtractionPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl ExtractionPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    pub async fn execute_extraction_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
        is_syphon: bool,
    ) -> Result<()> {
        let waypoint_symbol = self
            .context
            .mining_manager
            .get_waypoint(&ship, is_syphon)
            .await?;

        if !self.has_space(ship) {
            return Ok(());
        }

        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.go_to_waypoint(ship, &waypoint_symbol).await?;

        self.context.mining_manager.notify_waypoint(ship).await?;

        let i = self.wait_for_extraction(ship, pilot).await?;

        if i == 0 {
            self.context.mining_manager.unassign_waypoint(ship).await?;
            return Ok(());
        }

        self.extract(ship, is_syphon).await?;

        self.eject_blacklist(ship).await?;

        self.context.mining_manager.unassign_waypoint(ship).await?;

        Ok(())
    }

    async fn go_to_waypoint(&self, ship: &mut ship::MyShip, waypoint_symbol: &str) -> Result<()> {
        if ship.nav.waypoint_symbol == waypoint_symbol {
            return Ok(());
        }

        let system_symbol = ship.nav.system_symbol.clone();

        if !waypoint_symbol.contains(&system_symbol) {
            return Err("Waypoint is not in ship's system".into());
        }

        let waypoints = safely_get_map(&self.context.all_waypoints, &system_symbol)
            .unwrap()
            .clone();

        ship.nav_to(
            &waypoint_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            TransactionReason::MiningWaypoint(waypoint_symbol.to_string()),
        )
        .await?;

        Ok(())
    }

    fn has_space(&self, ship: &mut ship::MyShip) -> bool {
        ship.cargo.units >= ship.cargo.capacity
    }

    async fn extract(&self, ship: &mut ship::MyShip, is_syphon: bool) -> Result<()> {
        if ship.is_on_cooldown() {
            return Err("Ship is on cooldown".into());
        }

        ship.ensure_undocked(&self.context.api).await?;

        let action = if is_syphon {
            ActionType::Siphon
        } else {
            ActionType::Extract
        };
        // let action = ActionType::get_action(&ship).ok_or("Invalid ship role")?;

        match action {
            ActionType::Extract => {
                let _erg = ship.extract(&self.context.api).await?;
            }
            ActionType::Siphon => {
                let _erg = ship.siphon(&self.context.api).await?;
            }
        }
        ship.notify().await;

        Ok(())
    }

    async fn eject_blacklist(&self, ship: &mut ship::MyShip) -> Result<()> {
        let cargo = ship.cargo.inventory.clone();
        for item in cargo.iter() {
            if CONFIG.mining.blacklist.contains(&item.0) {
                ship.jettison(&self.context.api, *item.0, *item.1).await?;
            }
        }
        Ok(())
    }

    async fn wait_for_extraction(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<i32> {
        //needs revisit

        let i = tokio::select! {
            _ = pilot.cancellation_token.cancelled() => {0},// it's the end of the Programm we don't care(for now)
            _ = ship.wait_for_cooldown(&self.context.api) => {1},
        };

        Ok(i)
    }
}
