use std::{collections::HashMap, env, str::FromStr, time::Duration};

use database::{DatabaseConnectorAsync, DbPool, PaginatedQuery};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

pub async fn export_waypoints(system: Option<&str>) -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let waypoints = if let Some(system) = system {
        database::Waypoint::get_by_system(&database_pool, system, PaginatedQuery::unpaged()).await?
    } else {
        database::Waypoint::get_all(&database_pool, PaginatedQuery::unpaged()).await?
    };

    print_json(&waypoints.items);

    Ok(())
}

pub async fn export_systems() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let systems = database::System::get_all(&database_pool, PaginatedQuery::unpaged()).await?;

    print_json(&systems.items);

    Ok(())
}

pub async fn export_jump_connections() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let connections = ship::autopilot::generate_all_connections(&database_pool).await?;

    let output = json!({
        "connections":connections.0,
        "system_to_gate_mapping":connections.1
    });

    print_json(&output);

    Ok(())
}

pub async fn export_routes() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    eprintln!("Fetching routes...");

    let routes = database::Route::get_all(&database_pool, PaginatedQuery::unpaged()).await?;

    let ship_state_ids = routes
        .items
        .iter()
        .flat_map(|r| [r.ship_info_before, r.ship_info_after])
        .filter_map(|f| f)
        .collect::<Vec<i64>>();
    eprintln!("Fetching ship states... {}", ship_state_ids.len());
    let ship_states = database::ShipState::get_by_ids(&database_pool, &ship_state_ids)
        .await?
        .into_iter()
        .map(|f| (f.id, f))
        .collect::<HashMap<i64, database::ShipState>>();

    // we will print it as a CSV with semi-colons as delimiters
    let mut output = String::new();
    output.push_str( // ;ShipSymbol;From;To;CreatedAt;
        "ID; Distance; NavMode; EngineSpeed; CalcTravelTime; RealTravelTime; TimeDiff; TimeDiffPercent; CalcFuelCost; RealFuelCost; EngineConditionBefore; FrameConditionBefore; ReactorConditionBefore; EngineConditionAfter; FrameConditionAfter; ReactorConditionAfter; Incident;\n"
    );
    for route in routes.items {
        let ship_state_before = if let Some(id) = route.ship_info_before {
            ship_states.get(&id)
        } else {
            None
        };
        let ship_state_after = if let Some(id) = route.ship_info_after {
            ship_states.get(&id)
        } else {
            None
        };
        if ship_state_before.is_none() || ship_state_after.is_none() {
            continue;
        }

        let ship_state_before = ship_state_before.unwrap();
        let ship_state_after = ship_state_after.unwrap();

        let distance = route.distance;
        let nav_mode = space_traders_client::models::ShipNavFlightMode::from_str(&route.nav_mode);
        if let Err(e) = nav_mode {
            eprintln!("Failed to parse nav mode: {} - {:?}", e, route.nav_mode);
            continue;
        };
        let nav_mode = nav_mode.unwrap();
        let engine_speed = ship_state_before.engine_speed;

        let travel_stats = ship::autopilot::get_travel_stats(
            engine_speed,
            nav_mode,
            ship_state_before.engine_condition,
            distance,
        );

        let time_diff = travel_stats.travel_time - route.travel_time;
        let time_diff_percent = if route.travel_time > 0.0 {
            (time_diff / travel_stats.travel_time) * 100.0
        } else {
            0.0
        };

        let incident = if ship_state_before.engine_condition != ship_state_after.engine_condition
            || ship_state_before.frame_condition != ship_state_after.frame_condition
            || ship_state_before.reactor_condition != ship_state_after.reactor_condition
        {
            1
        } else {
            0
        };

        output.push_str(&get_csv_line(&[
            &route.id,
            // &route.ship_symbol,
            // &route.from,
            // &route.to,
            // &route.created_at,
            &distance,
            &nav_mode,
            &engine_speed,
            &travel_stats.travel_time,
            &route.travel_time,
            &time_diff,
            &time_diff_percent,
            &travel_stats.fuel_cost,
            &route.fuel_cost,
            &ship_state_before.engine_condition,
            &ship_state_before.frame_condition,
            &ship_state_before.reactor_condition,
            &ship_state_after.engine_condition,
            &ship_state_after.frame_condition,
            &ship_state_after.reactor_condition,
            &incident,
        ]))
    }
    println!("{}", output);

    Ok(())
}

async fn connect() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL").unwrap();

    let database_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(120))
        .connect(&database_url)
        .await?;

    Ok(DbPool::new(database_pool, None))
}

fn get_csv_line(values: &[&dyn std::fmt::Display]) -> String {
    let text = values
        .iter()
        .map(|f| format!("{}", f))
        .collect::<Vec<String>>()
        .join("; ");
    format!("{};\n", text)
}

fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

pub(crate) async fn generate_graphql() -> anyhow::Result<()> {
    let schema = async_graphql::Schema::build(
        spacetraders::control_api::QueryRoot,
        spacetraders::control_api::MutationRoot,
        async_graphql::EmptySubscription,
    )
    .finish();

    println!("{}", schema.sdl());
    tokio::fs::write("schema.graphql", schema.sdl()).await?;

    Ok(())
}
