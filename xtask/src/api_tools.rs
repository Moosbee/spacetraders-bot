use utils::distance_between_waypoints;

pub(crate) async fn check_waypoints() -> Result<(), spacetraders::error::Error> {
    let api: space_traders_client::Api =
        space_traders_client::Api::new(None, 500, std::num::NonZeroU32::new(2).unwrap());

    let systems = api.get_all_systems(20).await?;
    for system in systems {
        for waypoint in system
            .waypoints
            .into_iter()
            .map(|f| database::Waypoint::from(f))
            .collect::<Vec<_>>()
        {
            let valid = spacetraders::system_analyzation::check_waypoint(&waypoint);
            if !valid {
                println!(
                    "Invalid waypoint: {:?} distance: {}",
                    waypoint,
                    distance_between_waypoints((0, 0), (waypoint.x, waypoint.y))
                );
            }
        }
    }

    Ok(())
}
