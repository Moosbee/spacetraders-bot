/*
exchanges should be valued very low as in 1 or 2 exchanges will impact it very little and the more exchanges their are the less a single one will make a difference.

imports should be logarythmic, a few will have a high impact but the more their are the less the influence, but they don't stagnate with their influence

exports should be somewhat linear, the more exports their are the better

less valuable=longer time until next scrap
     */

use chrono::DateTime;
use space_traders_client::models;

use crate::error::Result;

const EXPORT_WEIGHT: f64 = 0.15;
const IMPORT_WEIGHT: f64 = 0.15;
const EXCHANGE_WEIGHT: f64 = 0.3;

pub fn get_waypoint_time(
    // waypoint: &database::Waypoint,
    market_trade: &[database::MarketTrade],
    // market_trade_goods: &[database::MarketTradeGood],
    max_update_interval: i32, // in seconds
) -> Result<chrono::DateTime<chrono::Utc>> {
    get_waypoint_time_with_weights(
        market_trade,
        max_update_interval,
        EXPORT_WEIGHT,
        IMPORT_WEIGHT,
        EXCHANGE_WEIGHT,
    )
}

pub fn get_waypoint_time_with_weights(
    // waypoint: &database::Waypoint,
    market_trade: &[database::MarketTrade],
    // market_trade_goods: &[database::MarketTradeGood],
    max_update_interval: i32, // in seconds
    export_weight: f64,
    import_weight: f64,
    exchange_weight: f64,
) -> Result<chrono::DateTime<chrono::Utc>> {
    let last_date = market_trade
        .iter()
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .map(|f| f.created_at)
        .unwrap_or(DateTime::<chrono::Utc>::MIN_UTC);

    let num_exports = market_trade
        .iter()
        .filter(|f| f.r#type == models::market_trade_good::Type::Export)
        .count() as f64;

    let num_imports = market_trade
        .iter()
        .filter(|f| f.r#type == models::market_trade_good::Type::Import)
        .count() as f64;

    let num_exchanges = market_trade
        .iter()
        .filter(|f| f.r#type == models::market_trade_good::Type::Exchange)
        .count() as f64;

    // let interval_percent = calc_interval_c(
    //     export_weight,
    //     import_weight,
    //     exchange_weight,
    //     num_exports,
    //     num_imports,
    //     num_exchanges,
    // );
    let interval_percent = calc_interval_a(
        export_weight,
        import_weight,
        exchange_weight,
        num_exports,
        num_imports,
        num_exchanges,
    );

    let interval_duration =
        chrono::Duration::seconds((interval_percent * max_update_interval as f64) as i64);

    let nextscrap = last_date + interval_duration;

    Ok(nextscrap)
}

fn calc_interval_a(
    export_weight: f64,
    import_weight: f64,
    exchange_weight: f64,
    num_exports: f64,
    num_imports: f64,
    num_exchanges: f64,
) -> f64 {
    // Export contribution (somewhat linear)
    let export_factor = 1.0 + (export_weight * num_exports);

    // Import contribution (logarithmic decay)
    let import_factor = 1.0 + (import_weight * f64::ln_1p(num_imports));

    // Exchange contribution (very low, rapidly diminishing)
    let exchange_factor = 1.0 + (exchange_weight * f64::ln_1p(num_exchanges) / f64::ln(10.0));

    // Calculate update interval modification
    let interval_modifier = export_factor * import_factor * exchange_factor;

    1.0 / interval_modifier
}

fn calc_interval_c(
    export_weight: f64,   // Weight for exports
    import_weight: f64,   // Weight for imports
    exchange_weight: f64, // Weight for exchanges
    num_exports: f64,
    num_imports: f64,
    num_exchanges: f64,
) -> f64 {
    // Calculate factor for exchanges (minimal and diminishing impact)
    let exchanges_factor = 1.0 - (num_exchanges + 1.0).log10() * exchange_weight;

    // Calculate factor for imports (logarithmic influence)
    let imports_factor = 1.0 - (num_imports + 1.0).log2() * import_weight;

    // Calculate factor for exports (somewhat linear influence)
    let exports_factor = 1.0 / (1.0 + (num_exports + 1.0).log2() * export_weight);

    // Combine factors multiplicatively
    exchanges_factor * imports_factor * exports_factor
}
