use std::time::Duration;

use chrono::NaiveDateTime;
use log::{debug, info, warn};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
};

use super::ScrappingManager;

pub struct ShipyardScrapper<'a> {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    scrapping_manager: &'a ScrappingManager,
}

impl<'a> ShipyardScrapper<'a> {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        scrapping_manager: &'a ScrappingManager,
    ) -> Self {
        Self {
            cancel_token,
            context,
            scrapping_manager,
        }
    }
    pub async fn run_scrapping_worker(&self) -> crate::error::Result<()> {
        info!("Starting shipyard scrapping workers");

        if !CONFIG.market.active {
            info!("shipyard scrapping not active, exiting");

            return Ok(());
        }

        for i in 0..CONFIG.market.max_scraps {
            if i != 0 {
                let erg = tokio::select! {
                _ = self.cancel_token.cancelled() => {
                  info!("shipyard scrapping cancelled");
                  0},
                _ =  sleep(Duration::from_millis(CONFIG.market.scrap_interval)) => {1},
                };
                if erg == 0 {
                    break;
                }
            }

            let shipyards = self.get_all_shipyards().await?;

            info!("shipyards: {:?}", shipyards.len());
            self.update_shipyards(shipyards).await?;
        }

        info!("shipyard scrapping workers done");

        Ok(())
    }

    async fn get_all_shipyards(&self) -> crate::error::Result<Vec<models::Shipyard>> {
        let systems = self.scrapping_manager.get_system().await;
        let mut shipyard_handles = tokio::task::JoinSet::new();

        for system in systems {
            let waypoints =
                sql::Waypoint::get_by_system(&self.context.database_pool, &system).await?;
            for waypoint in waypoints.iter().filter(|w| w.is_shipyard()) {
                let api = self.context.api.clone();
                let waypoint = waypoint.clone();
                shipyard_handles.spawn(async move {
                    debug!("Shipyard: {}", waypoint.symbol);
                    loop {
                        let shipyard = api
                            .get_shipyard(&waypoint.system_symbol, &waypoint.symbol)
                            .await;
                        match shipyard {
                            Ok(shipyard) => {
                                break *shipyard.data;
                            }
                            Err(e) => {
                                warn!("Shipyard: {} Error: {}", waypoint.symbol, e);
                                sleep(Duration::from_millis(500)).await;
                            }
                        }
                    }
                });
            }
        }

        let mut shipyards = Vec::new();
        while let Some(shipyard_data) = shipyard_handles.join_next().await {
            debug!(
                "Shipyard: {} {}",
                shipyard_data.is_ok(),
                shipyard_data
                    .as_ref()
                    .map(|m| m.symbol.clone())
                    .unwrap_or("Error".to_string())
            );
            match shipyard_data {
                Ok(shipyard) => {
                    shipyards.push(shipyard);
                }
                Err(e) => {
                    warn!("Shipyard: Error: {}", e);
                }
            }
        }

        Ok(shipyards)
    }

    async fn update_shipyards(
        &self,
        shipyards: Vec<models::Shipyard>,
    ) -> Result<(), crate::error::Error> {
        for shipyard in shipyards {
            update_shipyard(&self.context.database_pool, shipyard).await?;
        }

        Ok(())
    }
}

pub async fn update_shipyard(
    database_pool: &crate::sql::DbPool,
    shipyard: models::Shipyard,
) -> Result<(), crate::error::Error> {
    let sql_shipyard = crate::sql::Shipyard::from(&shipyard);
    let id = crate::sql::Shipyard::insert_get_id(database_pool, &sql_shipyard).await?;
    let ship_types = shipyard
        .ship_types
        .iter()
        .map(|st| crate::sql::ShipyardShipTypes {
            id: 0,
            shipyard_id: id,
            ship_type: st.r#type,
            created_at: NaiveDateTime::MIN,
        })
        .collect::<Vec<_>>();

    crate::sql::ShipyardShipTypes::insert_bulk(database_pool, &ship_types).await?;

    if let Some(ships) = shipyard.ships {
        for ship in ships.iter() {
            crate::ship::MyShip::update_info_db_shipyard((ship).clone(), database_pool).await?;
        }

        let shipyard_ships = ships
            .into_iter()
            .map(|s| {
                let ship = crate::sql::ShipyardShip::with_waypoint(s, &shipyard.symbol);
                ship
            })
            .collect::<Vec<_>>();

        crate::sql::ShipyardShip::insert_bulk(database_pool, &shipyard_ships).await?;
    }

    if let Some(transactions) = shipyard.transactions {
        let shipyard_transactions = transactions
            .into_iter()
            .filter_map(|t| crate::sql::ShipyardTransaction::try_from(t).ok())
            .collect::<Vec<_>>();
        crate::sql::ShipyardTransaction::insert_bulk(database_pool, &shipyard_transactions).await?
    }

    Ok(())
}
