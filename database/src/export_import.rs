use std::str::FromStr;

use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnector, DbPool};

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct ExportImportMapping {
    /// The symbol of the exported good.
    pub export_symbol: models::TradeSymbol,
    /// The symbols of the goods that importers of the exported good are looking for.
    pub import_symbol: models::TradeSymbol,
}

impl ExportImportMapping {
    pub fn generate_mapping(
        mapping: models::GetSupplyChain200ResponseData,
    ) -> crate::Result<Vec<ExportImportMapping>> {
        let mut result = Vec::new();
        for export_to_import in mapping.export_to_import_map {
            for import in export_to_import.1 {
                let export_symbol =
                    models::TradeSymbol::from_str(&export_to_import.0).map_err(|_err| {
                        crate::Error::InvalidTradeSymbol(export_to_import.0.to_string())
                    })?;
                let import_symbol = models::TradeSymbol::from_str(&import)
                    .map_err(|_err| crate::Error::InvalidTradeSymbol(import.to_string()))?;
                result.push(ExportImportMapping {
                    export_symbol,
                    import_symbol,
                });
            }
        }
        Ok(result)
    }
}

impl DatabaseConnector<ExportImportMapping> for ExportImportMapping {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &ExportImportMapping) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ExportImportMapping (
                    export_symbol,
                    import_symbol
                )
                VALUES ($1, $2)
                ON CONFLICT (export_symbol, import_symbol) DO NOTHING;
            "#,
            item.export_symbol as models::TradeSymbol,
            item.import_symbol as models::TradeSymbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[ExportImportMapping],
    ) -> crate::Result<()> {
        let (export_symbols, import_symbols): (Vec<_>, Vec<_>) = itertools::multiunzip(
            items
                .iter()
                .map(|item| (item.export_symbol, item.import_symbol)),
        );

        sqlx::query!(
            r#"
                INSERT INTO ExportImportMapping (
                    export_symbol,
                    import_symbol
                )
                SELECT export_sym, import_sym FROM UNNEST(
                    $1::trade_symbol[],
                    $2::trade_symbol[]
                ) AS t(export_sym, import_sym)
                ON CONFLICT (export_symbol, import_symbol) DO NOTHING;
            "#,
            &export_symbols as &[models::TradeSymbol],
            &import_symbols as &[models::TradeSymbol]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<ExportImportMapping>> {
        let items = sqlx::query_as!(
            ExportImportMapping,
            r#"
                SELECT
                    export_symbol as "export_symbol: models::TradeSymbol",
                    import_symbol as "import_symbol: models::TradeSymbol"
                FROM ExportImportMapping
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
    }
}
