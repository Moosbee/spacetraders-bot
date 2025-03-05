use std::time::Duration;

use chrono::NaiveDateTime;
use log::{debug, info};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::DatabaseConnector,
    types::{ConductorContext, WaypointCan},
};

pub struct ShipyardScrapper {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
}

impl ShipyardScrapper {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
    ) -> Self {
        Self {
            cancel_token,
            context,
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
            self.update_shipyards(&self.context.database_pool, shipyards)
                .await?;
        }

        info!("shipyard scrapping workers done");

        Ok(())
    }

    async fn get_all_shipyards(&self) -> crate::error::Result<Vec<models::Shipyard>> {
        let future_shipyards = self
            .context
            .all_waypoints
            .iter()
            .flat_map(|f| f.clone().into_iter())
            .map(|w| w.1.clone())
            .filter(|w| w.is_shipyard())
            .map(|w| {
                let api = self.context.api.clone();
                let w = w.clone();
                tokio::spawn(async move {
                    debug!("shipyard: {}", w.symbol);
                    api.get_shipyard(&w.system_symbol, &w.symbol).await.unwrap()
                })
            })
            .collect::<Vec<_>>();

        let mut shipyards = Vec::new();

        for shipyard in future_shipyards {
            let shipyard_data = shipyard.await.unwrap().data;
            debug!("shipyard: {:?}", shipyard_data.symbol);
            shipyards.push(*shipyard_data);
        }

        Ok(shipyards)
    }

    async fn update_shipyards(
        &self,
        database_pool: &crate::sql::DbPool,
        shipyards: Vec<models::Shipyard>,
    ) -> Result<(), sqlx::Error> {
        for shipyard in shipyards {
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
                crate::sql::ShipyardTransaction::insert_bulk(database_pool, &shipyard_transactions)
                    .await?
            }
        }

        Ok(())
    }
}
