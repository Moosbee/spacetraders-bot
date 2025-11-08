use std::collections::HashMap;

use async_graphql::Object;
use ship::MyShip;

use crate::utils::ConductorContext;

type Result<T> = std::result::Result<T, GraphiQLError>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn me(&self) -> &str {
        "Hello world!"
    }
    async fn ship<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<MyShip> {
        let context = ctx.data::<ConductorContext>()?;
        let ship = context
            .ship_manager
            .get_clone(&symbol)
            .ok_or(GraphiQLError::NotFound)?;
        Ok(ship)
    }
    async fn ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<MyShip>> {
        let context = ctx.data::<ConductorContext>()?;
        let ships = context
            .ship_manager
            .get_all_clone()
            .await
            .into_values()
            .collect();
        Ok(ships)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GraphiQLError {
    #[error("Not found")]
    NotFound,
    #[error("Graphql error: {:?}", 0)]
    GraphiQLError(async_graphql::Error),
}

impl From<async_graphql::Error> for GraphiQLError {
    fn from(value: async_graphql::Error) -> Self {
        GraphiQLError::GraphiQLError(value)
    }
}
