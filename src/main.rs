#![recursion_limit = "256"]
mod tests;

use core::panic;
use std::{env, num::NonZeroU32};

use chrono::{DateTime, Utc};

use opentelemetry::{
    global::{self},
    sdk::propagation::TraceContextPropagator,
};
use rsntp::AsyncSntpClient;

use ::utils::get_random_faction;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use tracing_subscriber::{fmt::format, layer::SubscriberExt};

use spacetraders::db_administration::{create_database_pool, export_database, reset_database};
use spacetraders::open_telemetry;
use spacetraders::reset_runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _erg = dotenvy::dotenv();

    setup_logging();

    info!("SpaceTraders starting up");

    check_time().await;

    let account_token = env::var("ACCOUNT_TOKEN").unwrap();
    let database_url = env::var("DATABASE_URL").unwrap();
    let agent_symbol = env::var("AGENT_SYMBOL").unwrap_or("MOOSBEE".to_string());
    let readyset_url = env::var("READYSET_URL").ok();
    let socket_address = env::var("SOCKET_ADDRESS")
        .ok()
        .unwrap_or("0.0.0.0:8780".to_string());

    let global_cancel_token = CancellationToken::new();

    let mut reset_cycle = 0u64;

    loop {
        reset_cycle += 1;

        info!("Starting reset cycle {}", reset_cycle);

        let database_pool = create_database_pool(&database_url, readyset_url.as_ref()).await?;

        info!("Running database migrations");
        let migrate_result = sqlx::migrate!().run(&database_pool.database_pool).await;

        if let Err(e) = migrate_result {
            error!(error = ?e, "Error running database migrations");
            match e {
                sqlx::migrate::MigrateError::VersionMismatch(_) => {
                    info!("Database migrations already applied")
                }
                _ => return Err(e.into()),
            }
        }

        wait_for_api().await?;

        // check db if already has an agent, if not create agent
        let agent_token = {
            let db_agent_token = database::Configuration::get_agent_token(&database_pool).await?;

            if let Some(db_agent_token) = db_agent_token {
                info!("Using existing agent token from database");
                db_agent_token
            } else {
                info!(%agent_symbol, "Registering new agent");
                let account_api =
                    space_traders_client::Api::new(None, 500, NonZeroU32::new(2).unwrap());

                let faction = get_random_faction();

                let agent_token_response = account_api
                    .register(agent_symbol.clone(), faction, account_token.clone())
                    .await?;

                let agent_token = agent_token_response.token;

                database::Configuration::set_agent_token(&database_pool, &agent_token).await?;
                info!(
                    agent = ?*agent_token_response.agent,
                    "Agent registered and token saved"
                );
                agent_token
            }
        };

        let reset_info = reset_runner::run_reset(
            &agent_token,
            database_pool.clone(),
            global_cancel_token.clone(),
            socket_address.clone(),
        )
        .await?;

        info!(reset_info=?reset_info, "run finished");

        let filename = format!(
            "spacetraders_reset_{}_UTC_to_{}_UTC_{}_{}",
            reset_info.start_date.format("%Y-%m-%d_%H-%M-%S"),
            reset_info.end_date.format("%Y-%m-%d_%H-%M-%S"),
            reset_info.version,
            reset_info.agent_symbol
        );

        if global_cancel_token.is_cancelled() {
            info!("Global cancel token triggered, shutting down");
            break;
        }

        export_database(&database_url, &filename).await?;

        reset_database(database_pool, &database_url).await?;

        info!("Reset cycle {} completed", reset_cycle);
    }

    info!("SpaceTraders shutting down");
    Ok(())
}

async fn wait_for_api() -> Result<(), anyhow::Error> {
    let waiting_api = space_traders_client::Api::new(None, 500, NonZeroU32::new(2).unwrap());

    loop {
        let status = waiting_api.get_status().await;

        let working = match status {
            Ok(status) => status.status == "SpaceTraders is currently online and available to play",
            Err(status_error) => {
                error!(?status_error, "Failed to get status");
                false
            }
        };

        if working {
            info!("API is working");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}

fn setup_logging() {
    // console_subscriber::init();

    let otel_endpoint = env::var("OTEL_ENDPOINT").ok();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let telemetry = if let Some(otel_endpoint) = otel_endpoint {
        let tracer = open_telemetry::init_trace(otel_endpoint).unwrap();
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        Some(telemetry)
    } else {
        None
    };
    let fmt_tracer = tracing_subscriber::fmt::Layer::new()
        .with_span_events(format::FmtSpan::NEW | format::FmtSpan::CLOSE)
        .json();

    // let tracing_tracy = tracing_tracy::TracyLayer::default();

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with(fmt_tracer);
    // .with(tracing_tracy);

    if let Some(telemetry) = telemetry {
        let subscriber = subscriber.with(telemetry);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    // let env = Env::default()
    //     .filter_or("RUST_LOG", "info")
    //     .write_style_or("RUST_LOG_STYLE", "always");

    // env_logger::Builder::from_env(env)
    //     .target(Target::Stdout)
    //     .init();
}

#[tracing::instrument]
async fn check_time() {
    let client = AsyncSntpClient::new();
    let result = client.synchronize("pool.ntp.org").await.unwrap();
    let local_time: DateTime<Utc> = result.datetime().into_chrono_datetime().unwrap();
    let time_diff = (local_time - Utc::now()).abs();

    info!(current_time = %Utc::now(), expected_time = %local_time, time_diff_ms = ?time_diff.num_milliseconds(), "Checked local time against NTP");

    if time_diff > chrono::Duration::milliseconds(1000) {
        error!(
            time_diff_ms = time_diff.num_milliseconds(),
            "The time is not correct"
        );
        panic!(
            "The time is not correct, off by: {:?}",
            time_diff.to_std().unwrap()
        );
    }
}
