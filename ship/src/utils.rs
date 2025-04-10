use database::DatabaseConnector;
use space_traders_client::models;

pub async fn update_market(
    market: models::Market,
    database_pool: &database::DbPool,
) -> crate::error::Result<()> {
    if let Some(trade_goods) = market.trade_goods {
        database::MarketTradeGood::insert_bulk(
            database_pool,
            &trade_goods
                .iter()
                .map(|f| database::MarketTradeGood::from(f.clone(), &market.symbol))
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }
    if let Some(transactions) = market.transactions {
        database::MarketTransaction::insert_bulk(
            database_pool,
            &transactions
                .iter()
                .map(|f| database::MarketTransaction::try_from(f.clone()).unwrap())
                .collect::<Vec<_>>(),
        )
        .await?;
    }

    let market_trades = [
        market
            .exchange
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Exchange,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .exports
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Export,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .imports
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Import,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ]
    .iter()
    .flatten()
    .cloned()
    .collect::<Vec<_>>();
    database::MarketTrade::insert_bulk(database_pool, &market_trades).await?;

    Ok(())
}
