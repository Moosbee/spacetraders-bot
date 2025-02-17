use space_traders_client::models;

use crate::ship;

pub struct TransferResult {
    pub ship_symbol: String,
    pub available_space: i32,
}

pub struct ShipInventory {
    pub ship_symbol: String,
    pub amount: i32,
}

#[derive(Debug)]
pub struct ShipInventoryManager;

impl ShipInventoryManager {
    pub fn new() -> Self {
        Self
    }

    pub fn determine_most_abundant_cargo(
        &self,
        ships: &[ship::MyShip],
    ) -> Option<models::TradeSymbol> {
        let cargo_totals = ships
            .iter()
            .map(|f| f.cargo.inventory.clone())
            .reduce(|acc, inventory| {
                acc.into_iter()
                    .map(|(symbol, amount)| (symbol, amount + inventory.get(&symbol).unwrap_or(&0)))
                    .collect()
            })
            .unwrap_or_default();

        cargo_totals
            .into_iter()
            .max_by_key(|(_, amount)| *amount)
            .map(|(symbol, _)| symbol)
    }

    pub fn find_best_extractor(
        &self,
        ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Option<ShipInventory> {
        ships
            .iter()
            .max_by_key(|ship| ship.cargo.get_amount(trade_symbol))
            .map(|ship| ShipInventory {
                ship_symbol: ship.symbol.clone(),
                amount: ship.cargo.get_amount(trade_symbol),
            })
    }

    pub fn find_best_transporter(
        &self,
        ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Option<TransferResult> {
        // First try to find a ship that already has some of the cargo
        let transporter = ships
            .iter()
            .filter(|ship| ship.cargo.units < ship.cargo.capacity && ship.cargo.has(trade_symbol))
            .max_by_key(|ship| ship.cargo.get_amount(trade_symbol));

        // If none found, pick the one with most available space
        let ship = transporter.or_else(|| {
            ships
                .iter()
                .min_by_key(|ship| ship.cargo.capacity - ship.cargo.units)
        });

        ship.map(|ship| TransferResult {
            ship_symbol: ship.symbol.clone(),
            available_space: ship.cargo.capacity - ship.cargo.units,
        })
    }
}
