use std::str::FromStr;

use space_traders_client::models;
use tracing::instrument;

use super::{
    run_paginated_query, DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult,
};

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

    pub async fn get_imports_for_export(
        database_pool: &DbPool,
        export_symbol: models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<models::TradeSymbol>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping
                        WHERE export_symbol = $1
                        LIMIT $2 OFFSET $3;
                    "#,
                    export_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items.into_iter().map(|item| item.import_symbol).collect())
            },
            || async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping
                        WHERE export_symbol = $1;
                    "#,
                    export_symbol as models::TradeSymbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items.into_iter().map(|item| item.import_symbol).collect())
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ExportImportMapping
                    WHERE export_symbol = $1
                    "#,
                    export_symbol as models::TradeSymbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    pub async fn get_exports_for_import(
        database_pool: &DbPool,
        import_symbol: models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<models::TradeSymbol>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping
                        WHERE import_symbol = $1
                        LIMIT $2 OFFSET $3;
                    "#,
                    import_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items.into_iter().map(|item| item.export_symbol).collect())
            },
            || async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping
                        WHERE import_symbol = $1;
                    "#,
                    import_symbol as models::TradeSymbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items.into_iter().map(|item| item.export_symbol).collect())
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ExportImportMapping
                    WHERE import_symbol = $1
                    "#,
                    import_symbol as models::TradeSymbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for ExportImportMapping {
    type ID = (models::TradeSymbol, models::TradeSymbol);

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &DbPool,
        item: &ExportImportMapping,
    ) -> crate::Result<Self::ID> {
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

        Ok((item.export_symbol, item.import_symbol))
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ExportImportMapping) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ExportImportMapping) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
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

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ExportImportMapping>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping
                        LIMIT $1 OFFSET $2;
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ExportImportMapping,
                    r#"
                        SELECT
                            export_symbol as "export_symbol: models::TradeSymbol",
                            import_symbol as "import_symbol: models::TradeSymbol"
                        FROM ExportImportMapping;
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ExportImportMapping
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            ExportImportMapping,
            r#"
                SELECT
                    export_symbol as "export_symbol: models::TradeSymbol",
                    import_symbol as "import_symbol: models::TradeSymbol"
                FROM ExportImportMapping
                WHERE export_symbol = $1 AND import_symbol = $2
                LIMIT 1;
            "#,
            id.0 as models::TradeSymbol,
            id.1 as models::TradeSymbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM ExportImportMapping
                WHERE export_symbol = $1 AND import_symbol = $2;
            "#,
            id.0 as models::TradeSymbol,
            id.1 as models::TradeSymbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.export_symbol = id.0;
        self.import_symbol = id.1;
    }
}
