use std::collections::HashSet;

use database::DatabaseConnector;
use space_traders_client::models::{self};
use tracing::debug;
use utils::{distance_between_waypoints, WaypointCan};

use crate::{
    error::{Error, Result},
    manager::Manager,
    utils::ConductorContext,
};

use super::{
    messages::{ChartManagerMessage, NextChartResp},
    messanger::ChartManagerMessanger,
};

pub struct ChartManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ChartManagerMessage>,
    running_charts: HashSet<String>,
}

impl ChartManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ChartManagerMessage>,
        ChartManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ChartManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ChartManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
            running_charts: HashSet::new(),
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::chart_manager::chart_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_chart_worker(&mut self) -> Result<()> {
        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            debug!("Received chartManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_chart_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::chart_manager::chart_manager_handle_chart_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_chart_message(&mut self, message: super::messages::ChartMessage) -> Result<()> {
        match message {
            super::messages::ChartMessage::Next {
                ship_clone,
                callback,
            } => {
                let next_chart = self.get_next_chart(ship_clone).await;

                callback
                    .send(next_chart)
                    .map_err(|e| Error::General(format!("Failed to send message: {:?}", e)))?;
            }
            super::messages::ChartMessage::Fail { waypoint_symbol } => {
                self.fail_chart(waypoint_symbol)
            }
            super::messages::ChartMessage::Success { waypoint_symbol } => {
                self.success_chart(waypoint_symbol)
            }
        }

        Ok(())
    }

    // #[tracing::instrument(
    //     level = "info",
    //     name = "spacetraders::manager::chart_manager::get_required_ships",
    //     skip(all_ships, all_systems_hashmap, connection_hash_map)
    // )]
    // pub fn get_required_ships(
    //     all_ships: &[ship::MyShip],
    //     all_systems_hashmap: &HashMap<String, HashMap<String, database::Waypoint>>,
    //     connection_hash_map: &HashMap<String, Vec<database::JumpGateConnection>>,
    // ) -> Result<RequiredShips> {
    //     // we need a probe in every system, that has uncharted waypoints and to which we have a jump gate connection

    //     // let all_ships = context
    //     //     .ship_manager
    //     //     .get_all_clone()
    //     //     .await
    //     //     .into_values()
    //     //     .collect::<Vec<_>>();

    //     // let all_systems_hashmap: HashMap<String, HashMap<String, database::Waypoint>> =
    //     //     database::Waypoint::get_hash_map(&context.database_pool).await?;
    //     // let all_connections: Vec<database::JumpGateConnection> =
    //     //     database::JumpGateConnection::get_all(&context.database_pool).await?;

    //     // let mut connection_hash_map: HashMap<String, Vec<database::JumpGateConnection>> =
    //     //     HashMap::new();

    //     // for connection in all_connections {
    //     //     let entry = connection_hash_map
    //     //         .entry(connection.from.clone())
    //     //         .or_default();
    //     //     entry.push(connection);
    //     // }

    //     let all_systems = all_ships
    //         .iter()
    //         .map(|ship| ship.nav.system_symbol.clone())
    //         .collect::<HashSet<_>>();

    //     let with_chart = all_ships
    //         .iter()
    //         .filter(|ship| {
    //             ship.role == database::ShipInfoRole::Charter
    //                 || (ship.role == database::ShipInfoRole::Transfer
    //                     && match &ship.status {
    //                         ship::ShipStatus::Transfer { role, .. } => {
    //                             role == &Some(database::ShipInfoRole::Charter)
    //                         }
    //                         _ => false,
    //                     })
    //         })
    //         .map(|ship| match &ship.role {
    //             database::ShipInfoRole::Transfer => match &ship.status {
    //                 ship::ShipStatus::Transfer { system_symbol, .. } => {
    //                     system_symbol.clone().unwrap_or_default()
    //                 }
    //                 _ => ship.nav.system_symbol.clone(),
    //             },
    //             _ => ship.nav.system_symbol.clone(),
    //         })
    //         .collect::<HashSet<_>>();

    //     let mut reachable_systems = HashSet::new();
    //     let mut to_visit_systems = all_systems.iter().cloned().collect::<Vec<_>>();

    //     while let Some(system) = to_visit_systems.pop() {
    //         reachable_systems.insert(system.clone());
    //         let waypoints = all_systems_hashmap.get(&system);
    //         if waypoints.is_none() {
    //             continue;
    //         }
    //         let waypoints = waypoints
    //             .map(|w| w.values().collect::<Vec<_>>())
    //             .unwrap_or_default()
    //             .into_iter()
    //             .filter(|w| w.is_jump_gate())
    //             .collect::<Vec<_>>();

    //         for waypoint in waypoints {
    //             if waypoint.is_under_construction {
    //                 continue;
    //             }
    //             if let Some(connections) = connection_hash_map.get(&waypoint.symbol) {
    //                 for connection in connections {
    //                     let system_symbol = get_system_symbol(&connection.to);
    //                     let wp = all_systems_hashmap
    //                         .get(&system_symbol)
    //                         .and_then(|w| w.get(&connection.to).cloned());
    //                     if let Some(wp) = wp {
    //                         if !reachable_systems.contains(&wp.system_symbol)
    //                             && !to_visit_systems.contains(&wp.system_symbol)
    //                             && !wp.is_under_construction
    //                         {
    //                             to_visit_systems.push(wp.system_symbol);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     let mut needed_ships = HashMap::new();

    //     for system in reachable_systems.iter() {
    //         let waypoints = all_systems_hashmap
    //             .get(system)
    //             .unwrap()
    //             .values()
    //             .collect::<Vec<_>>();
    //         let has_uncharted = waypoints.iter().any(|w| !w.is_charted());
    //         if has_uncharted && !with_chart.contains(system) {
    //             needed_ships.insert(
    //                 system.clone(),
    //                 vec![(
    //                     RequestedShipType::Probe,
    //                     Priority::Low,
    //                     Budget::High,
    //                     database::ShipInfoRole::Charter,
    //                 )],
    //             );
    //         }
    //     }
    //     Ok(RequiredShips {
    //         ships: needed_ships,
    //     })
    // }

    async fn get_next_chart(
        &mut self,
        ship_clone: ship::MyShip,
    ) -> std::result::Result<NextChartResp, Error> {
        let ship_waypoint = database::Waypoint::get_by_symbol(
            &self.context.database_pool,
            &ship_clone.nav.waypoint_symbol,
        )
        .await?
        .ok_or(Error::General("Waypoint not found".to_string()))?;

        let waypoints = database::Waypoint::get_by_system(
            &self.context.database_pool,
            &ship_clone.nav.system_symbol,
        )
        .await?;
        let mut system = waypoints
            .iter()
            .filter(|w| !w.is_charted())
            .filter(|w| !self.running_charts.contains(&w.symbol))
            .collect::<Vec<_>>();

        if system.is_empty() {
            return Ok(NextChartResp::NoChartsInSystem);
        }

        system.sort_by(|a, b| {
            if a.waypoint_type == models::WaypointType::Asteroid
                && b.waypoint_type != models::WaypointType::Asteroid
            {
                return std::cmp::Ordering::Greater;
            } else if a.waypoint_type != models::WaypointType::Asteroid
                && b.waypoint_type == models::WaypointType::Asteroid
            {
                return std::cmp::Ordering::Less;
            }
            let distance_a =
                distance_between_waypoints((a.x, a.y), (ship_waypoint.x, ship_waypoint.y));
            let distance_b =
                distance_between_waypoints((b.x, b.y), (ship_waypoint.x, ship_waypoint.y));
            distance_a
                .partial_cmp(&distance_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let waypoint = system.first().map(|w| w.symbol.clone());

        if let Some(waypoint) = waypoint {
            self.running_charts.insert(waypoint.clone());
            Ok(NextChartResp::Next(waypoint))
        } else {
            Ok(NextChartResp::NoChartsInSystem)
        }
    }

    fn fail_chart(&mut self, waypoint_symbol: String) {
        self.running_charts.remove(&waypoint_symbol);
    }

    fn success_chart(&mut self, waypoint_symbol: String) {
        self.running_charts.remove(&waypoint_symbol);
    }
}

impl Manager for ChartManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_chart_worker().await })
    }

    fn get_name(&self) -> &str {
        "ChartManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
