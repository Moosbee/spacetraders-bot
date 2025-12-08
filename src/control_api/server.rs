use std::time::Duration;

use futures::FutureExt;
use std::convert::Infallible;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use async_graphql::{
    http::{GraphQLPlaygroundConfig, GraphiQLSource},
    EmptySubscription, Schema,
};
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};
use warp::{http::Response as HttpResponse, Filter, Rejection};

use crate::{
    control_api::graphql::{mutations::MutationRoot, QueryRoot},
    manager::Manager,
    utils::ConductorContext,
};

pub struct ControlApiServer {
    context: ConductorContext,
    cancellation_token: CancellationToken,
    ship_rx: Option<tokio::sync::broadcast::Receiver<ship::MyShip>>,
    ship_cancellation_token: CancellationToken,
}

impl ControlApiServer {
    pub fn new(
        context: ConductorContext,
        ship_rx: tokio::sync::broadcast::Receiver<ship::MyShip>,
        cancellation_token: CancellationToken,
        ship_cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            context,
            cancellation_token,
            ship_rx: Some(ship_rx),
            ship_cancellation_token,
        }
    }

    #[instrument(
        level = "info",
        name = "spacetraders::control_api::run_server",
        skip(self)
    )]
    async fn run_server(&mut self) -> anyhow::Result<()> {
        let config = { self.context.config.read().await.clone() };
        if !config.control_active {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(config.control_start_sleep)).await;
        let context = self.context.clone();
        let database_pool = self.context.database_pool.clone();

        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .data(context)
            .data(database_pool)
            .finish();

        tokio::fs::write("schema.graphql", schema.sdl()).await?;

        tracing::info!(socket_address = %config.socket_address, "GraphiQL IDE available at address");

        let graphql_post = async_graphql_warp::graphql(schema).and_then(
            |(schema, request): (
                Schema<QueryRoot, MutationRoot, EmptySubscription>,
                async_graphql::Request,
            )| async move {
                Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
            },
        );

        let graphiql = warp::path::end().and(warp::get()).map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(GraphiQLSource::build().endpoint("/").finish())
        });

        let playground = warp::path("playground").and(warp::get()).map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(async_graphql::http::playground_source(
                    GraphQLPlaygroundConfig::new("/"),
                ))
        });

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec![
                "Access-Control-Allow-Origin",
                "Origin",
                "Accept",
                "X-Requested-With",
                "Content-Type",
            ])
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST]);

        let routes = graphiql.or(playground).or(graphql_post).with(cors).recover(
            |err: Rejection| async move {
                if let Some(GraphQLBadRequest(err)) = err.find() {
                    return Ok::<_, Infallible>(warp::reply::with_status(
                        err.to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    ));
                }

                Ok(warp::reply::with_status(
                    "INTERNAL_SERVER_ERROR".to_string(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            },
        );

        let socket_address: std::net::SocketAddr = config
            .socket_address
            .parse()
            .expect("Invalid socket address");

        tokio::select! {
            _ = self.cancellation_token.cancelled() => {
                tracing::info!("Server is shutting down due to cancellation");
            },
            _ = warp::serve(routes).run(socket_address).fuse() => {
                tracing::info!("Server shutdown completed");
            }
        }

        Ok(())
    }
}

impl Manager for ControlApiServer {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send + '_>>
    {
        Box::pin(async move {
            self.run_server()
                .await
                .map_err(|e| crate::error::Error::General(e.to_string()))
        })
    }

    fn get_name(&self) -> &str {
        "ControlApiServer"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancellation_token
    }
}
