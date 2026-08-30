use std::collections::HashMap;

use futures::future;
use space_traders_client::models;

mod trade_route_candidate;
mod trade_route_proposal;

pub use trade_route_candidate::TradeRouteCandidate;
pub use trade_route_proposal::TradeRouteProposal;

pub use trade_route_proposal::filter_trade_route_proposal;
pub use trade_route_proposal::gen_trade_route_proposal;
pub use trade_route_proposal::sort_trade_route_proposal;

pub(crate) fn get_trade_systems(
    _trading_config: &database::TradingFleetConfig,
    ship_clone: &ship::RustShip<ship::status::ShipStatus>,
) -> Vec<String> {
    vec![ship_clone.nav.system_symbol.clone()]
}

pub async fn fetch_market_data(
    database_pool: &database::DbPool,
    trade_systems: &[String],
) -> Result<(Vec<database::MarketTradeGood>, Vec<database::MarketTrade>), database::Error> {
    let trades = future::try_join_all(trade_systems.iter().map(|system_symbol| async move {
        let trade_goods = database::MarketTradeGood::get_last_by_system(
            database_pool,
            system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;
        let market_trade = database::MarketTrade::get_last_by_system(
            database_pool,
            system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;
        Ok::<_, database::Error>((trade_goods, market_trade))
    }))
    .await?;

    let trade_goods = trades
        .iter()
        .flat_map(|(trade_goods, _)| trade_goods)
        .cloned()
        .collect::<Vec<_>>();
    let market_trade = trades
        .into_iter()
        .flat_map(|(_, market_trade)| market_trade)
        .collect::<Vec<_>>();

    Ok((trade_goods, market_trade))
}

// generates all possible trades
pub fn gen_all_trade_route_candidates(
    trade_goods: &[database::MarketTradeGood],
    market_trade: &[database::MarketTrade],
) -> Vec<TradeRouteCandidate> {
    let trade_goods_map = trade_goods
        .iter()
        .map(|t| ((t.symbol, t.waypoint_symbol.clone()), t.clone()))
        .collect::<HashMap<(models::TradeSymbol, String), database::MarketTradeGood>>();

    let waypoint_market_trades_map: HashMap<String, Vec<database::MarketTrade>> =
        market_trade.iter().fold(HashMap::new(), |mut acc, e| {
            acc.entry(e.waypoint_symbol.clone())
                .or_insert(Vec::new())
                .push(e.clone());
            acc
        });

    market_trade
        .iter()
        .flat_map(|t| market_trade.iter().map(move |t2| (t, t2)))
        .filter(|t| t.0.symbol == t.1.symbol)
        .map(|(t1, t2)| {
            let trade_good_1 = trade_goods_map.get(&(t1.symbol, t1.waypoint_symbol.clone()));

            let trade_good_2 = trade_goods_map.get(&(t2.symbol, t2.waypoint_symbol.clone()));

            assert!(
                t1.symbol == t2.symbol
                    && trade_good_1.map(|t| t.symbol).unwrap_or(t1.symbol)
                        == trade_good_2.map(|t| t.symbol).unwrap_or(t2.symbol)
            );

            TradeRouteCandidate {
                symbol: t1.symbol,
                purchase_good: trade_good_1.cloned(),
                sell_good: trade_good_2.cloned(),
                purchase: t1.clone(),
                sell: t2.clone(),
                purchase_waypoint_market_trades: waypoint_market_trades_map
                    .get(&t1.waypoint_symbol)
                    .cloned()
                    .unwrap_or_default(),
                sell_waypoint_market_trades: waypoint_market_trades_map
                    .get(&t2.waypoint_symbol)
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>()
}

pub(crate) fn filter_trade_route_candidates(
    trade_route_candidates_all: Vec<TradeRouteCandidate>,
    blacklist: &[models::TradeSymbol],
) -> Vec<TradeRouteCandidate> {
    trade_route_candidates_all
        .into_iter()
        .filter(|trade_route_candidate| trade_route_candidate.is_valid(blacklist))
        .collect()
}

pub(crate) async fn gen_trade_route_proposals(
    database_pool: &database::DbPool,
    trade_route_candidates_filtered: Vec<TradeRouteCandidate>,
    trade_systems: &[String],
    ship_stats: &ship::autopilot::ShipNavStats,
    purchase_multiplier: f64,
    fallback_purchase_price: i32,
    fallback_sell_price: i32,
) -> Result<Vec<TradeRouteProposal>, crate::error::Error> {
    // navigator_cache: &mut ship::autopilot::NavigatorCache,
    // travel_price_cache: &mut ship::autopilot::TravelPriceCache,
    let mut proposals = Vec::with_capacity(trade_route_candidates_filtered.len());
    let mut navigator_cache = ship::autopilot::NavigatorCache::default();
    let mut travel_price_cache = ship::autopilot::TravelPriceCache::new(database_pool.clone());
    for system_symbol in trade_systems {
        navigator_cache
            .preload_system_routers(database_pool, system_symbol)
            .await?;
        travel_price_cache
            .preload_system_prices(system_symbol)
            .await?;
    }
    if trade_systems.len() > 1 {
        navigator_cache
            .preload_jump_gate_router(database_pool)
            .await?;
    }
    for trade_route_candidate in trade_route_candidates_filtered {
        let proposal = trade_route_proposal::gen_trade_route_proposal(
            database_pool,
            trade_route_candidate,
            ship_stats,
            purchase_multiplier,
            fallback_purchase_price,
            fallback_sell_price,
            &mut navigator_cache,
            &mut travel_price_cache,
        )
        .await?;
        if let Some(proposal) = proposal {
            proposals.push(proposal);
        }
    }
    Ok(proposals)
}

pub(crate) fn get_best_trade_route_proposal(
    trade_route_proposals: Vec<TradeRouteProposal>,
    filter: impl Fn(&TradeRouteProposal) -> bool,
    sort: impl Fn(&TradeRouteProposal, &TradeRouteProposal) -> std::cmp::Ordering,
) -> Option<TradeRouteProposal> {
    trade_route_proposals
        .into_iter()
        .filter(filter)
        .max_by(sort)
}
