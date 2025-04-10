use chrono::DateTime;
use database::DatabaseConnector;
use space_traders_client::models;

pub async fn update_shipyard(
    database_pool: &database::DbPool,
    shipyard: models::Shipyard,
) -> Result<(), crate::error::Error> {
    let sql_shipyard = database::Shipyard::from(&shipyard);
    let id = database::Shipyard::insert_get_id(database_pool, &sql_shipyard).await?;
    let ship_types = shipyard
        .ship_types
        .iter()
        .map(|st| database::ShipyardShipTypes {
            id: 0,
            shipyard_id: id,
            ship_type: st.r#type,
            created_at: DateTime::<chrono::Utc>::MIN_UTC,
        })
        .collect::<Vec<_>>();

    database::ShipyardShipTypes::insert_bulk(database_pool, &ship_types).await?;

    if let Some(ships) = shipyard.ships {
        for ship in ships.iter() {
            ship::MyShip::update_info_db_shipyard((ship).clone(), database_pool).await?;
        }

        let shipyard_ships = ships
            .into_iter()
            .map(|s| database::ShipyardShip::with_waypoint(s, &shipyard.symbol))
            .collect::<Vec<_>>();

        database::ShipyardShip::insert_bulk(database_pool, &shipyard_ships).await?;
    }

    if let Some(transactions) = shipyard.transactions {
        let shipyard_transactions = transactions
            .into_iter()
            .filter_map(|t| database::ShipyardTransaction::try_from(t).ok())
            .collect::<Vec<_>>();
        database::ShipyardTransaction::insert_bulk(database_pool, &shipyard_transactions).await?
    }

    Ok(())
}
