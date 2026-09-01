use space_traders_client::models;

use crate::supply_chain_mapping::SupplyChainMapping;

#[derive(Debug, Clone, PartialEq, Default, async_graphql::SimpleObject)]
#[graphql(name = "InternalTradeRouteProposal")]
pub struct TradeRouteProposal {
    pub symbol: space_traders_client::models::TradeSymbol,
    #[graphql(skip)]
    pub purchase_good: Option<database::MarketTradeGood>,
    #[graphql(skip)]
    pub sell_good: Option<database::MarketTradeGood>,
    #[graphql(skip)]
    pub purchase: database::MarketTrade,
    #[graphql(skip)]
    pub sell: database::MarketTrade,

    #[graphql(skip)]
    pub purchase_waypoint_market_trades: Vec<database::MarketTrade>,
    #[graphql(skip)]
    pub sell_waypoint_market_trades: Vec<database::MarketTrade>,

    pub fuel_units: i32,
    pub time: f64,
    pub distance: f64,
    pub api_requests: i32,

    pub trade_volume: i32,

    pub travel_cost: i32,
    pub good_cost: i32,
    pub total_cost: i32,
    pub good_total_sell_price: i32,
    pub good_profit: i32,
    pub total_profit: i32,

    /// trips per hour, one trip is a roundtrip
    pub trips_per_hour: f64,
    pub profit_per_hour: f64,
    pub profit_per_api_request: f64,
}

impl From<TradeRouteProposal> for database::TradeRoute {
    fn from(value: TradeRouteProposal) -> Self {
        database::TradeRoute {
            symbol: value.symbol,
            status: database::ShipmentStatus::InTransit,
            trade_volume: value.trade_volume,
            purchase_waypoint: value.purchase.waypoint_symbol,
            sell_waypoint: value.sell.waypoint_symbol,
            purchase_trade_good_id: value.purchase_good.map(|g| g.id),
            sell_trade_good_id: value.sell_good.map(|g| g.id),
            estimated_fuel: Some(value.travel_cost),
            ..Default::default()
        }
    }
}

impl From<&TradeRouteProposal> for crate::manager::trade_manager::routes_tracker::MinTradeRoute {
    fn from(value: &TradeRouteProposal) -> Self {
        Self {
            symbol: value.symbol.clone(),
            purchase_wp_symbol: value.purchase.waypoint_symbol.clone(),
            sell_wp_symbol: value.sell.waypoint_symbol.clone(),
        }
    }
}

pub async fn gen_trade_route_proposal(
    database_pool: &database::DbPool,
    trade_route_candidate: super::TradeRouteCandidate,
    ship_stats: &ship::autopilot::ShipNavStats,
    purchase_multiplier: f64,
    fallback_purchase_price: i32,
    fallback_sell_price: i32,
    navigator_cache: &mut ship::autopilot::NavigatorCache,
    travel_price_cache: &mut ship::autopilot::TravelPriceCache,
) -> Result<Option<TradeRouteProposal>, crate::error::Error> {
    // calculate route between buy and sell wp
    let route = navigator_cache
        .get_route(
            database_pool,
            &trade_route_candidate.purchase.waypoint_symbol,
            &trade_route_candidate.sell.waypoint_symbol,
            ship_stats,
        )
        .await?;
    if route.is_none() {
        return Ok(None);
    }
    // calculate travel cost, based on wp fuel data
    let trip_information: TripInformation =
        gen_trip_information(&route.unwrap(), ship_stats, travel_price_cache).await?;
    // calculate trade volume
    let min_trade_volume = trade_route_candidate
        .purchase_good
        .as_ref()
        .map(|t| t.trade_volume)
        .unwrap_or(i32::MAX)
        .min(
            trade_route_candidate
                .sell_good
                .as_ref()
                .map(|t| t.trade_volume)
                .unwrap_or(i32::MAX),
        );
    let trip_volume = ship_stats
        .max_cargo
        .min((min_trade_volume as f64 * purchase_multiplier) as u32) as i32;
    // calculate key figures
    let purchase_unit_price = trade_route_candidate
        .purchase_good
        .as_ref()
        .map(|f| f.purchase_price)
        .unwrap_or(fallback_purchase_price);
    let sell_unit_price = trade_route_candidate
        .sell_good
        .as_ref()
        .map(|f| f.sell_price)
        .unwrap_or(fallback_sell_price);

    let travel_cost = trip_information.total_travel_cost;
    let good_cost = trip_volume * purchase_unit_price;
    let total_cost = good_cost + travel_cost;
    let good_total_sell_price = trip_volume * sell_unit_price;
    let revenue = good_total_sell_price;
    let good_profit = good_total_sell_price - total_cost;
    let total_profit = revenue - total_cost;

    let roundtrip_time = (trip_information.total_time) * 2.0;
    let trips_per_hour = 3600.0 / roundtrip_time;
    let profit_per_hour = total_profit as f64 / trips_per_hour;
    let profit_per_api_request = total_profit as f64 / trip_information.total_api_requests as f64;

    // assemble proposal
    Ok(Some(TradeRouteProposal {
        symbol: trade_route_candidate.symbol,
        purchase_good: trade_route_candidate.purchase_good,
        sell_good: trade_route_candidate.sell_good,
        purchase: trade_route_candidate.purchase,
        sell: trade_route_candidate.sell,
        purchase_waypoint_market_trades: trade_route_candidate.purchase_waypoint_market_trades,
        sell_waypoint_market_trades: trade_route_candidate.sell_waypoint_market_trades,
        fuel_units: trip_information.total_fuel_units + trip_information.total_antimatter_units,
        time: trip_information.total_time,
        distance: trip_information.total_distance,
        api_requests: trip_information.total_api_requests,
        trade_volume: trip_volume,

        travel_cost,
        good_cost,
        total_cost,
        good_total_sell_price,
        good_profit,
        total_profit,

        trips_per_hour,
        profit_per_hour,
        profit_per_api_request,
    }))
}

async fn gen_trip_information(
    route: &[ship::autopilot::SimpleConnection],
    ship_stats: &ship::autopilot::ShipNavStats,
    travel_price_cache: &mut ship::autopilot::TravelPriceCache,
) -> Result<TripInformation, crate::error::Error> {
    let route = ship::autopilot::assemble_route(route, ship_stats, travel_price_cache).await?;

    Ok(TripInformation {
        total_time: route.total_travel_time + 1.0,
        total_distance: route.total_distance,
        total_api_requests: route.total_api_requests + 2,
        total_fuel_units: route.total_refuel,
        total_antimatter_units: route.total_anti_matter,
        total_fuel_cost: route.total_fuel_cost,
        total_antimatter_cost: route.total_anti_matter_cost,
        total_travel_cost: route.total_fuel_cost + route.total_anti_matter_cost,
    })
}

pub struct TripInformation {
    // trip time in seconds
    pub total_time: f64,
    pub total_distance: f64,
    pub total_api_requests: i32,
    pub total_fuel_units: i32,
    pub total_antimatter_units: i32,
    pub total_fuel_cost: i32,
    pub total_antimatter_cost: i32,
    pub total_travel_cost: i32,
}

pub fn filter_trade_route_proposal(
    trade_route_proposal: &TradeRouteProposal,
    trading_config: &database::TradingFleetConfig,
) -> bool {
    trade_route_proposal.total_profit > trading_config.trade_profit_threshold
}

pub fn sort_trade_route_proposal(
    trade_route_proposal_a: &TradeRouteProposal,
    trade_route_proposal_b: &TradeRouteProposal,
    trading_config: &database::TradingFleetConfig,
    supply_chain_mapping: &SupplyChainMapping,
) -> Option<std::cmp::Ordering> {
    // sorts how filled-in a trade route proposal is
    match (
        &trade_route_proposal_a.sell_good,
        &trade_route_proposal_b.sell_good,
    ) {
        (Some(_), None) => return Some(std::cmp::Ordering::Greater),
        (None, Some(_)) => return Some(std::cmp::Ordering::Less),
        _ => {}
    }

    match (
        &trade_route_proposal_a.purchase_good,
        &trade_route_proposal_b.purchase_good,
    ) {
        (Some(_), None) => return Some(std::cmp::Ordering::Greater),
        (None, Some(_)) => return Some(std::cmp::Ordering::Less),
        _ => {}
    }

    // either both are Some or both are None

    if trade_route_proposal_a.purchase_good.is_none()
        && trade_route_proposal_b.purchase_good.is_none()
        && trade_route_proposal_a.sell_good.is_none()
        && trade_route_proposal_b.sell_good.is_none()
    {
        return Some(std::cmp::Ordering::Equal);
    }

    match trading_config.trade_mode {
        database::TradeMode::ProfitPerHour => trade_route_proposal_a
            .profit_per_hour
            .partial_cmp(&trade_route_proposal_b.profit_per_hour),
        database::TradeMode::ProfitPerAPIRequest => trade_route_proposal_a
            .profit_per_api_request
            .partial_cmp(&trade_route_proposal_b.profit_per_api_request),
        database::TradeMode::ProfitPerTrip => trade_route_proposal_a
            .total_profit
            .partial_cmp(&trade_route_proposal_b.total_profit),
        database::TradeMode::MarketBalanced => {
            // prefer trades that are imports for an export in the prefer list

            // a preferable trade in this case is, if it's  an import to a item on the prefer list and the sell wp exports that same preferred item
            let is_a_preferable = is_preferable_trade_route_proposal(
                trade_route_proposal_a,
                trading_config,
                supply_chain_mapping,
            );
            let is_b_preferable = is_preferable_trade_route_proposal(
                trade_route_proposal_b,
                trading_config,
                supply_chain_mapping,
            );

            match (is_a_preferable, is_b_preferable) {
                (true, false) => return Some(std::cmp::Ordering::Greater),
                (false, true) => return Some(std::cmp::Ordering::Less),
                _ => {}
            }

            // prefer trades that trade from high supply to low supply the higher difference the better

            // better if sell pw exporting
            // better if buy pw importing

            let purchase_good_a = trade_route_proposal_a.purchase_good.as_ref().unwrap();
            let purchase_good_b = trade_route_proposal_b.purchase_good.as_ref().unwrap();
            let sell_good_a = trade_route_proposal_a.sell_good.as_ref().unwrap();
            let sell_good_b = trade_route_proposal_b.sell_good.as_ref().unwrap();

            // todo calculate divide via the length not of a linear function but a curve to make it better to fill up scarce markets
            // let route_a_diff = (purchase_good_a.supply as i32) - (sell_good_a.supply as i32);
            // let route_b_diff = (purchase_good_b.supply as i32) - (sell_good_b.supply as i32);
            // Some(route_a_diff.cmp(&route_b_diff))

            let route_a_diff =
                curve_score(purchase_good_a.supply) - curve_score(sell_good_a.supply);
            let route_b_diff =
                curve_score(purchase_good_b.supply) - curve_score(sell_good_b.supply);
            Some(
                route_a_diff
                    .partial_cmp(&route_b_diff)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        }
    }
}

fn curve_score(supply: models::SupplyLevel) -> f64 {
    2f64.powi(4 - supply as i32)
}

/// A preferable trade is one where the traded good is an import needed to
/// produce an item on the prefer list, and the sell waypoint exports that
/// same preferred item.
fn is_preferable_trade_route_proposal(
    trade_route_proposal: &TradeRouteProposal,
    trading_config: &database::TradingFleetConfig,
    supply_chain_mapping: &SupplyChainMapping,
) -> bool {
    trade_route_proposal
        .sell_waypoint_market_trades
        .iter()
        .filter(|trade| {
            trade.r#type == space_traders_client::models::market_trade_good::Type::Export
        })
        .filter(|trade| trading_config.market_prefer_list.contains(&trade.symbol))
        .any(|preferred_export| {
            supply_chain_mapping
                .get_import_mapping(preferred_export.symbol)
                .contains(&trade_route_proposal.symbol)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use space_traders_client::models::TradeSymbol;
    use space_traders_client::models::market_trade_good::Type;

    fn make_trading_config(prefer_list: Vec<TradeSymbol>) -> database::TradingFleetConfig {
        database::TradingFleetConfig {
            market_blacklist: vec![],
            market_prefer_list: prefer_list,
            purchase_multiplier: 1.0,
            trade_mode: database::TradeMode::MarketBalanced,
            trade_profit_threshold: 0,
            ship_market_ratio: 1.0,
            min_cargo_space: 0,
        }
    }

    fn make_market_trade(symbol: TradeSymbol, r#type: Type) -> database::MarketTrade {
        database::MarketTrade {
            waypoint_symbol: "SELL_WP".to_string(),
            symbol,
            r#type,
            ..Default::default()
        }
    }

    fn make_proposal(
        traded_symbol: TradeSymbol,
        sell_waypoint_market_trades: Vec<database::MarketTrade>,
    ) -> TradeRouteProposal {
        TradeRouteProposal {
            symbol: traded_symbol,
            sell_waypoint_market_trades,
            ..Default::default()
        }
    }

    fn make_supply_chain_mapping(export: TradeSymbol, import: TradeSymbol) -> SupplyChainMapping {
        SupplyChainMapping::new(&[database::ExportImportMapping {
            export_symbol: export,
            import_symbol: import,
        }])
    }

    #[test]
    fn true_when_traded_good_is_import_of_preferred_export() {
        let config = make_trading_config(vec![TradeSymbol::AdvancedCircuitry]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Microprocessors);
        let proposal = make_proposal(
            TradeSymbol::Microprocessors,
            vec![make_market_trade(
                TradeSymbol::AdvancedCircuitry,
                Type::Export,
            )],
        );

        assert!(is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }

    #[test]
    fn false_when_traded_good_is_not_import_of_preferred_export() {
        let config = make_trading_config(vec![TradeSymbol::AdvancedCircuitry]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Electronics);
        let proposal = make_proposal(
            TradeSymbol::Microprocessors,
            vec![make_market_trade(
                TradeSymbol::AdvancedCircuitry,
                Type::Export,
            )],
        );

        assert!(!is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }

    #[test]
    fn false_when_preferred_item_is_not_exported_by_sell_waypoint() {
        let config = make_trading_config(vec![TradeSymbol::AdvancedCircuitry]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Microprocessors);
        let proposal = make_proposal(
            TradeSymbol::Microprocessors,
            vec![make_market_trade(
                TradeSymbol::AdvancedCircuitry,
                Type::Import,
            )],
        );

        assert!(!is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }

    #[test]
    fn false_when_sell_waypoint_has_no_preferred_trades() {
        let config = make_trading_config(vec![TradeSymbol::AdvancedCircuitry]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Microprocessors);
        let proposal = make_proposal(
            TradeSymbol::Microprocessors,
            vec![make_market_trade(TradeSymbol::Gold, Type::Export)],
        );

        assert!(!is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }

    #[test]
    fn false_when_prefer_list_is_empty() {
        let config = make_trading_config(vec![]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Microprocessors);
        let proposal = make_proposal(
            TradeSymbol::Microprocessors,
            vec![make_market_trade(
                TradeSymbol::AdvancedCircuitry,
                Type::Export,
            )],
        );

        assert!(!is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }

    #[test]
    fn false_when_sell_waypoint_has_no_market_trades() {
        let config = make_trading_config(vec![TradeSymbol::AdvancedCircuitry]);
        let mapping =
            make_supply_chain_mapping(TradeSymbol::AdvancedCircuitry, TradeSymbol::Microprocessors);
        let proposal = make_proposal(TradeSymbol::Microprocessors, vec![]);

        assert!(!is_preferable_trade_route_proposal(
            &proposal, &config, &mapping
        ));
    }
}
