use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::Loader;
use tracing::instrument;

use crate::manager::trade_manager::trade_route_calculator::{
    TradeRouteCandidate, TradeRouteProposal, gen_trade_route_proposal,
};

#[derive(Debug, Clone)]
pub struct TradeRouteProposalConfig {
    pub trade_route_candidate: TradeRouteCandidate,
    pub ship_stats: ship::autopilot::ShipNavStats,
    pub purchase_multiplier: f64,
}

impl std::hash::Hash for TradeRouteProposalConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.trade_route_candidate.hash(state);
        self.ship_stats.hash(state);
        // self.purchase_multiplier.hash(state);
    }
}

impl PartialEq for TradeRouteProposalConfig {
    fn eq(&self, other: &Self) -> bool {
        self.trade_route_candidate == other.trade_route_candidate
            && self.ship_stats == other.ship_stats
        // && self.purchase_multiplier == other.purchase_multiplier
    }
}

impl Eq for TradeRouteProposalConfig {}

pub struct TradeRouteProposalLoader {
    context: crate::utils::ConductorContext,
}

impl TradeRouteProposalLoader {
    pub fn new(context: crate::utils::ConductorContext) -> Self {
        Self { context }
    }
}

impl Loader<TradeRouteProposalConfig> for TradeRouteProposalLoader {
    type Value = TradeRouteProposal;
    type Error = Arc<crate::error::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(
        &self,
        keys: &[TradeRouteProposalConfig],
    ) -> Result<HashMap<TradeRouteProposalConfig, Self::Value>, Self::Error> {
        let mut navigator_cache = ship::autopilot::NavigatorCache::default();
        let mut travel_price_cache =
            ship::autopilot::TravelPriceCache::new(self.context.database_pool.clone());
        let trade_systems = keys
            .iter()
            .flat_map(|x| {
                [
                    x.trade_route_candidate.purchase.waypoint_symbol.clone(),
                    x.trade_route_candidate.sell.waypoint_symbol.clone(),
                ]
            })
            .collect::<Vec<String>>();
        for system_symbol in trade_systems.iter() {
            navigator_cache
                .preload_system_routers(&self.context.database_pool, system_symbol)
                .await
                .map_err(|e| Arc::new(e.into()))?;
            travel_price_cache
                .preload_system_prices(system_symbol)
                .await
                .map_err(|e| Arc::new(e.into()))?;
        }
        if trade_systems.len() > 1 {
            navigator_cache
                .preload_jump_gate_router(&self.context.database_pool)
                .await
                .map_err(|e| Arc::new(e.into()))?;
        }

        let config = self.context.config.read().await.clone();
        let fallback_purchase_price = config.default_purchase_price;
        let fallback_sell_price = config.default_sell_price;

        let mut proposals = HashMap::with_capacity(keys.len());

        for trade_route_candidate_config in keys {
            let proposal = gen_trade_route_proposal(
                &self.context.database_pool,
                trade_route_candidate_config.trade_route_candidate.clone(),
                &trade_route_candidate_config.ship_stats,
                trade_route_candidate_config.purchase_multiplier,
                fallback_purchase_price,
                fallback_sell_price,
                &mut navigator_cache,
                &mut travel_price_cache,
            )
            .await?;
            if let Some(proposal) = proposal {
                proposals.insert(trade_route_candidate_config.clone(), proposal);
            }
        }

        Ok(proposals)
    }
}
