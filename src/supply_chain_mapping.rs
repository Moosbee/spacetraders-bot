use std::collections::HashMap;

use space_traders_client::models;

pub struct SupplyChainMapping {
    import_mapping: HashMap<models::TradeSymbol, Vec<models::TradeSymbol>>,
    export_mapping: HashMap<models::TradeSymbol, Vec<models::TradeSymbol>>,
}

impl SupplyChainMapping {
    pub fn new(supply_chain_mapping: &[database::ExportImportMapping]) -> Self {
        let mut import_mapping = HashMap::new();
        let mut export_mapping = HashMap::new();

        for mapping in supply_chain_mapping.iter() {
            import_mapping
                .entry(mapping.export_good.clone())
                .or_insert(vec![])
                .push(mapping.import_good.clone());
            export_mapping
                .entry(mapping.import_good.clone())
                .or_insert(vec![])
                .push(mapping.export_good.clone());
        }

        Self {
            import_mapping,
            export_mapping,
        }
    }

    /// gets the necessary imports to create the given export
    pub fn get_import_mapping(&self, export_good: models::TradeSymbol) -> &[models::TradeSymbol] {
        self.import_mapping.get(&export_good).unwrap_or(&vec![])
    }

    /// returns the exports that need this import
    pub fn get_export_mapping(&self, import_good: models::TradeSymbol) -> &[models::TradeSymbol] {
        self.export_mapping.get(&import_good).unwrap_or(&vec![])
    }
}
