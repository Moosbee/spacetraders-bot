use std::{i32, str::FromStr};

use anyhow::Ok;
use log::debug;
use space_traders_client::models::{self, TradeSymbol};

pub async fn update_contract(database_pool: &sqlx::PgPool, contract: &models::Contract) {
    update_base_contract(database_pool, contract).await;
    if let Some(deliveries) = &contract.terms.deliver {
        update_contract_deliveries(database_pool, &contract.id, deliveries).await;
    }
}
