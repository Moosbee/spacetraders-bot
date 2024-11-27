#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use space_traders_client::models;

    use crate::ship::{
        ship_models::MyShip,
        nav::{
            nav_models::{NavMode, RouteConnection},
            stats::get_travel_stats,
            utils::get_route,
        },
    };

    #[test]
    fn get_inner_system_travel_stats_test() {
        let erg = get_travel_stats(10, models::ShipNavFlightMode::Cruise, (0, 0), (3, 4));

        assert!(
            erg.distance == 5.0
                && erg.fuel_cost == 5
                && erg.travel_time == chrono::Duration::milliseconds((27.5 * 1000.0) as i64),
            "erg != 5.0, 5, 27.5 was {:?}",
            erg
        );
    }

    #[test]
    fn dijkstra_test() {
        let waypoints = crate::tests::tests::get_waypoints();

        let mut realroute = serde_json::from_str::<Vec<RouteConnection>>(
            r#"
            [{"end_symbol":"X1-KC3-D42","distance":0,"start_symbol":"","flight_mode":"DRIFT","re_cost":0,"cost":0},{"end_symbol":"X1-KC3-D43","distance":0,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":1},{"end_symbol":"X1-KC3-H51","distance":51.478150704935004,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":26.739075352467502},{"end_symbol":"X1-KC3-H52","distance":51.478150704935004,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":26.739075352467502},{"end_symbol":"X1-KC3-H53","distance":51.478150704935004,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":26.739075352467502},{"end_symbol":"X1-KC3-H54","distance":51.478150704935004,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":26.739075352467502},{"end_symbol":"X1-KC3-BC5X","distance":57.8013840664737,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":29.90069203323685},{"end_symbol":"X1-KC3-A1","distance":81.39410298049853,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":41.697051490249265},{"end_symbol":"X1-KC3-A3","distance":81.39410298049853,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":41.697051490249265},{"end_symbol":"X1-KC3-A2","distance":81.39410298049853,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":41.697051490249265},{"end_symbol":"X1-KC3-A4","distance":81.39410298049853,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":41.697051490249265},{"end_symbol":"X1-KC3-F46","distance":124.8118584109699,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":63.40592920548495},{"end_symbol":"X1-KC3-F47","distance":124.8118584109699,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":63.40592920548495},{"end_symbol":"X1-KC3-F49","distance":124.8118584109699,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":63.40592920548495},{"end_symbol":"X1-KC3-F48","distance":124.8118584109699,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":63.40592920548495},{"end_symbol":"X1-KC3-G50","distance":137.32079230764728,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":69.66039615382364},{"end_symbol":"X1-KC3-E44","distance":142.8355697996826,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":72.4177848998413},{"end_symbol":"X1-KC3-E45","distance":142.8355697996826,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":72.4177848998413},{"end_symbol":"X1-KC3-C41","distance":160.58953888718904,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":81.29476944359452},{"end_symbol":"X1-KC3-B6","distance":182.36227680087788,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":92.18113840043894},{"end_symbol":"X1-KC3-K82","distance":185.28356645962967,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":93.64178322981483},{"end_symbol":"X1-KC3-K81","distance":185.28356645962967,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":93.64178322981483},{"end_symbol":"X1-KC3-C40","distance":194.06442229321684,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":98.03221114660842},{"end_symbol":"X1-KC3-C39","distance":194.06442229321684,"start_symbol":"X1-KC3-D42","flight_mode":"BURN","re_cost":0,"cost":98.03221114660842},{"end_symbol":"X1-KC3-I56","distance":171.96511274092776,"start_symbol":"X1-KC3-E44","flight_mode":"BURN","re_cost":0,"cost":159.40034127030518},{"end_symbol":"X1-KC3-B35","distance":156.18578680533003,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":171.27403180310395},{"end_symbol":"X1-KC3-B38","distance":157.68956845650888,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":172.02592262869337},{"end_symbol":"X1-KC3-B8","distance":164.1584600317632,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":175.26036841632055},{"end_symbol":"X1-KC3-B9","distance":179.96944185055418,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":183.16585932571604},{"end_symbol":"X1-KC3-B7","distance":180,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":183.18113840043895},{"end_symbol":"X1-KC3-B37","distance":182.17573932881405,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":184.26900806484596},{"end_symbol":"X1-KC3-B33","distance":184.94593804677083,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":185.65410742382437},{"end_symbol":"X1-KC3-B10","distance":187.20042734993956,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":186.78135207540873},{"end_symbol":"X1-KC3-B36","distance":190.5465822312224,"start_symbol":"X1-KC3-B6","flight_mode":"BURN","re_cost":0,"cost":188.45442951605014},{"end_symbol":"X1-KC3-B17","distance":184.09780009549272,"start_symbol":"X1-KC3-C40","flight_mode":"BURN","re_cost":0,"cost":191.08111119435478},{"end_symbol":"X1-KC3-B19","distance":194.98717906570164,"start_symbol":"X1-KC3-C40","flight_mode":"BURN","re_cost":0,"cost":196.52580067945922},{"end_symbol":"X1-KC3-B28","distance":106.47065323364932,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":213.63566788712984},{"end_symbol":"X1-KC3-B30","distance":128.4445405612866,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":224.62261155094848},{"end_symbol":"X1-KC3-B26","distance":142.22517358048822,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":231.51292806054929},{"end_symbol":"X1-KC3-B27","distance":146.23952953972466,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":233.5201060401675},{"end_symbol":"X1-KC3-B12","distance":250.79872407968904,"start_symbol":"X1-KC3-D42","flight_mode":"CRUISE","re_cost":0,"cost":251.79872407968904},{"end_symbol":"X1-KC3-B25","distance":186.10212250267324,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":253.4514025216418},{"end_symbol":"X1-KC3-B16","distance":254.01771591761076,"start_symbol":"X1-KC3-D42","flight_mode":"CRUISE","re_cost":0,"cost":255.01771591761076},{"end_symbol":"X1-KC3-B14","distance":254.12595302329905,"start_symbol":"X1-KC3-D42","flight_mode":"CRUISE","re_cost":0,"cost":255.12595302329905},{"end_symbol":"X1-KC3-B29","distance":195.04102132628407,"start_symbol":"X1-KC3-I56","flight_mode":"BURN","re_cost":0,"cost":257.92085193344724},{"end_symbol":"X1-KC3-B13","distance":259.67864756271354,"start_symbol":"X1-KC3-D42","flight_mode":"CRUISE","re_cost":0,"cost":260.67864756271354},{"end_symbol":"X1-KC3-B15","distance":270.1851217221259,"start_symbol":"X1-KC3-D42","flight_mode":"CRUISE","re_cost":0,"cost":271.1851217221259},{"end_symbol":"X1-KC3-B11","distance":177.0706073858674,"start_symbol":"X1-KC3-B7","flight_mode":"BURN","re_cost":0,"cost":272.7164420933726},{"end_symbol":"X1-KC3-B21","distance":203.2141727340886,"start_symbol":"X1-KC3-C40","flight_mode":"CRUISE","re_cost":0,"cost":302.246383880697},{"end_symbol":"X1-KC3-B34","distance":212.13203435596427,"start_symbol":"X1-KC3-B6","flight_mode":"CRUISE","re_cost":0,"cost":305.3131727564032},{"end_symbol":"X1-KC3-B20","distance":212.00235847744713,"start_symbol":"X1-KC3-C40","flight_mode":"CRUISE","re_cost":0,"cost":311.03456962405556},{"end_symbol":"X1-KC3-B32","distance":218.0022935659164,"start_symbol":"X1-KC3-B6","flight_mode":"CRUISE","re_cost":0,"cost":311.18343196635533},{"end_symbol":"X1-KC3-B18","distance":219.3855054464629,"start_symbol":"X1-KC3-C40","flight_mode":"CRUISE","re_cost":0,"cost":318.4177165930713},{"end_symbol":"X1-KC3-B23","distance":231.36551169091732,"start_symbol":"X1-KC3-C40","flight_mode":"CRUISE","re_cost":0,"cost":330.39772283752575},{"end_symbol":"X1-KC3-B24","distance":248.02016047087784,"start_symbol":"X1-KC3-C40","flight_mode":"CRUISE","re_cost":0,"cost":347.05237161748624},{"end_symbol":"X1-KC3-B22","distance":278.0071941515183,"start_symbol":"X1-KC3-G50","flight_mode":"CRUISE","re_cost":0,"cost":348.66759030534195},{"end_symbol":"X1-KC3-B31","distance":273.28007611240156,"start_symbol":"X1-KC3-K81","flight_mode":"CRUISE","re_cost":0,"cost":367.9218593422164},{"end_symbol":"X1-KC3-I55","distance":221.93016919743022,"start_symbol":"X1-KC3-I56","flight_mode":"CRUISE","re_cost":0,"cost":382.3305104677354},{"end_symbol":"X1-KC3-J57","distance":149.5125412799876,"start_symbol":"X1-KC3-I55","flight_mode":"BURN","re_cost":0,"cost":458.08678110772917},{"end_symbol":"X1-KC3-J58","distance":119.85407794480753,"start_symbol":"X1-KC3-J57","flight_mode":"BURN","re_cost":0,"cost":519.013820080133},{"end_symbol":"X1-KC3-J68","distance":136.8941196691808,"start_symbol":"X1-KC3-J57","flight_mode":"BURN","re_cost":0,"cost":527.5338409423196},{"end_symbol":"X1-KC3-J67","distance":181.41664752717708,"start_symbol":"X1-KC3-J57","flight_mode":"BURN","re_cost":0,"cost":549.7951048713177},{"end_symbol":"X1-KC3-J74","distance":381.16269492173546,"start_symbol":"X1-KC3-B7","flight_mode":"CRUISE","re_cost":0,"cost":565.3438333221744},{"end_symbol":"X1-KC3-J70","distance":230.09780529157595,"start_symbol":"X1-KC3-J57","flight_mode":"CRUISE","re_cost":0,"cost":689.1845863993051},{"end_symbol":"X1-KC3-J72","distance":334.37852801877096,"start_symbol":"X1-KC3-J57","flight_mode":"CRUISE","re_cost":0,"cost":793.4653091265002},{"end_symbol":"X1-KC3-J66","distance":384.94804844290354,"start_symbol":"X1-KC3-J57","flight_mode":"CRUISE","re_cost":0,"cost":844.0348295506327},{"end_symbol":"X1-KC3-J69","distance":399.0112780360976,"start_symbol":"X1-KC3-J58","flight_mode":"CRUISE","re_cost":0,"cost":919.0250981162305}]
        "#,
        )
        .unwrap();

        let wayps: HashMap<String, models::Waypoint> = waypoints
            .clone()
            .iter()
            .map(|w| (w.symbol.clone(), w.clone()))
            .collect();

        let mut routes = super::path_finding_tests::get_full_dijkstra(
            &wayps,
            "X1-KC3-D42".to_string(),
            400,
            NavMode::BurnAndCruise,
            true,
        )
        .unwrap()
        .iter()
        .map(|f| f.1.clone())
        .collect::<Vec<_>>();

        routes.sort_by(|a, b| a.end_symbol.cmp(&b.end_symbol));
        realroute.sort_by(|a, b| a.end_symbol.cmp(&b.end_symbol));

        routes.sort_by(|a, b| a.start_symbol.cmp(&b.start_symbol));
        realroute.sort_by(|a, b| a.start_symbol.cmp(&b.start_symbol));

        routes.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        realroute.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // println!("Routes: {:?}, \n\n\n Real Route: {:?}", routes, realroute);

        let zib_routes = routes.iter().zip(realroute.iter()).collect::<Vec<_>>();

        for (route, real) in zib_routes {
            println!(
                " Route: {:?}\nRRoute: {:?}\nis same: {}",
                route,
                real,
                route == real,
            );

            if route != real {
                println!("is equal: {}", is_equal_route(&wayps, route, real));

                assert!(is_equal_route(&wayps, route, real));
            }
        }

        // println!(
        //     "Route: {:?}",
        //     routes.iter().zip(realroute.iter()).collect::<Vec<_>>()
        // );

        // assert!(routes == realroute);
    }

    #[test]
    fn test_get_routes() {
        let waypoints = crate::tests::tests::get_waypoints();
        let wayps: HashMap<String, models::Waypoint> = waypoints
            .clone()
            .iter()
            .map(|w| (w.symbol.clone(), w.clone()))
            .collect();
        println!("start;end;dj;star;ship;");
        for _i in 0..200 {
            let index_s = (rand::random::<f32>() * waypoints.len() as f32).floor() as usize;
            let index_e = (rand::random::<f32>() * waypoints.len() as f32).floor() as usize;
            // let st = waypoints.choose(&mut rand::thread_rng());
            // let en = waypoints.choose(&mut rand::thread_rng());
            let start = waypoints[index_s].symbol.clone();
            let end = waypoints[index_e].symbol.clone();

            // println!("\n\nstart: {}, end: {}", start, end);

            let start_time = tokio::time::Instant::now();

            let all_routes = crate::ship::nav::tests::path_finding_tests::get_full_dijkstra(
                &wayps,
                start.clone(),
                400,
                NavMode::BurnAndCruiseAndDrift,
                false,
            )
            .unwrap();

            let dj_route = get_route(all_routes, start.clone(), end.clone()).unwrap();

            let elapsed_dj = start_time.elapsed();

            // println!("dj_route {:?}", dj_route);

            let start_time = tokio::time::Instant::now();

            let star_route = crate::ship::nav::tests::path_finding_tests::get_route_a_star(
                &wayps,
                start.clone(),
                end.clone(),
                400,
                NavMode::BurnAndCruiseAndDrift,
                false,
            )
            .unwrap();

            let elapsed_star = start_time.elapsed();

            // println!("star_route {:?}", star_route);

            let mut my_ship = MyShip::default();

            my_ship.fuel.capacity = 400;

            let start_time = tokio::time::Instant::now();

            let ship_route = my_ship
                .find_route(
                    &wayps,
                    start.clone(),
                    end.clone(),
                    &NavMode::BurnAndCruiseAndDrift,
                    false,
                )
                .unwrap();

            let elapsed_ship = start_time.elapsed();

            // println!("ship_route {:?}", ship_route);

            // println!(
            //     "Time dj: {:?}, star: {:?}, ship: {:?}",
            //     elapsed_dj, elapsed_star, elapsed_ship
            // );

            println!(
                "{};{};{};{};{};",
                start,
                end,
                elapsed_dj.as_nanos(),
                elapsed_star.as_nanos(),
                elapsed_ship.as_nanos()
            );

            // assert!(dj_route == star_route);
            assert!(are_equal_routes(
                &wayps,
                dj_route.clone(),
                star_route.clone()
            ));

            assert!(are_equal_routes(
                &wayps,
                dj_route.clone(),
                ship_route.clone()
            ));

            assert!(are_equal_routes(
                &wayps,
                star_route.clone(),
                ship_route.clone()
            ));
        }
        // assert!(false)
    }

    pub fn are_equal_routes(
        waypoints: &HashMap<String, models::Waypoint>,
        a: Vec<RouteConnection>,
        b: Vec<RouteConnection>,
    ) -> bool {
        if a.len() != b.len() {
            return false;
        }
        for i in 0..a.len() {
            if !is_equal_route(waypoints, &a[i], &b[i]) {
                return false;
            }
        }
        true
    }

    pub fn is_equal_route(
        waypoints: &HashMap<String, models::Waypoint>,
        a: &RouteConnection,
        b: &RouteConnection,
    ) -> bool {
        let start_wp_a = waypoints
            .get(&a.start_symbol)
            .unwrap_or_else(|| panic!("Did not find {:?}", a.start_symbol));
        let end_wp_a = waypoints
            .get(&a.end_symbol)
            .unwrap_or_else(|| panic!("Did not find {:?}", a.end_symbol));

        let start_wp_b = waypoints
            .get(&b.start_symbol)
            .unwrap_or_else(|| panic!("Did not find {:?}", b.start_symbol));
        let end_wp_b = waypoints
            .get(&b.end_symbol)
            .unwrap_or_else(|| panic!("Did not find {:?}", b.end_symbol));

        start_wp_a.x == start_wp_b.x
            && start_wp_a.y == start_wp_b.y
            && end_wp_a.x == end_wp_b.x
            && end_wp_a.y == end_wp_b.y
    }
}

#[cfg(test)]
mod path_finding_tests {
    use std::{cmp::Reverse, collections::HashMap};

    use priority_queue::PriorityQueue;
    use space_traders_client::models;

    use crate::{
        ship::nav::{
            nav_models::{NavMode, RouteConnection},
            utils::{distance_between_waypoints, get_nearby_waypoints, get_route},
        },
        IsMarketplace,
    };

    pub fn get_route_a_star(
        waypoints: &HashMap<String, models::Waypoint>,
        start_symbol: String,
        end_symbol: String,
        max_fuel: i32,
        nav_mode: NavMode,
        only_markets: bool,
    ) -> Result<Vec<RouteConnection>, anyhow::Error> {
        let mut unvisited = waypoints.clone();
        let mut visited = HashMap::new();
        let mut to_visit = PriorityQueue::new();

        let start = unvisited
            .get(&start_symbol)
            .ok_or_else(|| anyhow::anyhow!("Could not find start waypoint"))?;
        let end_waypoint = unvisited
            .get(&end_symbol)
            .ok_or_else(|| anyhow::anyhow!("Could not find end waypoint: {}", end_symbol))?
            .clone();

        let modes = nav_mode.get_flight_modes(max_fuel);
        let start_route = RouteConnection {
            start_symbol: "".to_string(),
            end_symbol: start.symbol.clone(),
            distance: 0.0,
            cost: 0.0,
            flight_mode: models::ShipNavFlightMode::Drift,
            re_cost: 0.0,
        };
        to_visit.push(start_route, Reverse(0));

        while let Some((current_route, _)) = to_visit.pop() {
            to_visit = to_visit
                .into_iter()
                .filter(|(c, _)| c.end_symbol != current_route.end_symbol)
                .collect();
            visited.insert(current_route.end_symbol.clone(), current_route.clone());
            let current = unvisited
                .remove(&current_route.end_symbol)
                .ok_or_else(|| anyhow::anyhow!("Could not remove from queue"))?;

            if current.symbol == end_symbol {
                break;
            }

            if !only_markets || current.is_marketplace() {
                for mode in &modes {
                    let next_waypoints =
                        get_nearby_waypoints(&unvisited, (current.x, current.y), mode.radius);
                    for waypoint in next_waypoints {
                        let distance = distance_between_waypoints(
                            (current.x, current.y),
                            (waypoint.x, waypoint.y),
                        );
                        let heuristic_cost = distance_between_waypoints(
                            (waypoint.x, waypoint.y),
                            (end_waypoint.x, end_waypoint.y),
                        ) * 0.4;
                        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;
                        let re_cost = cost + heuristic_cost;
                        let next_route = RouteConnection {
                            start_symbol: current.symbol.clone(),
                            end_symbol: waypoint.symbol.clone(),
                            distance,
                            cost,
                            flight_mode: mode.mode,
                            re_cost,
                        };
                        to_visit.push_increase(next_route, Reverse((re_cost * 1_000_000.0) as i64));
                    }
                }
            }
        }

        get_route(visited, start_symbol, end_symbol)
    }

    #[cfg(test)]
    pub fn get_full_dijkstra(
        waypoints: &HashMap<String, models::Waypoint>,
        start_symbol: String,
        max_fuel: i32,
        nav_mode: NavMode,
        only_markets: bool,
    ) -> Result<HashMap<String, RouteConnection>, anyhow::Error> {
        let mut unvisited: HashMap<String, models::Waypoint> = waypoints.clone();
        // .clone()
        // .iter()
        // .map(|w| (w.symbol.clone(), w.clone()))
        // .collect();
        let mut visited: HashMap<String, RouteConnection> = HashMap::new();
        let mut to_visit: PriorityQueue<RouteConnection, Reverse<i64>> = PriorityQueue::new();

        let start = match unvisited.get(&start_symbol) {
            Some(it) => it,
            None => return Err(anyhow::anyhow!("Could not find start waypoint")),
        };

        let modes = nav_mode.get_flight_modes(max_fuel);

        to_visit.push(
            RouteConnection {
                start_symbol: "".to_string(),
                end_symbol: start.symbol.clone(),
                distance: 0.0,
                cost: 0.0,
                flight_mode: models::ShipNavFlightMode::Drift,
                re_cost: 0.0,
            },
            Reverse(0),
        );

        while !to_visit.is_empty() {
            let (current_route, _) = to_visit
                .pop()
                .ok_or(anyhow::anyhow!("Could not pop from queue"))?;
            to_visit = to_visit
                .into_iter()
                .filter(|(c, _)| current_route.end_symbol != c.end_symbol)
                .collect();
            visited.insert(current_route.end_symbol.clone(), current_route.clone());
            let current = unvisited
                .remove(&current_route.end_symbol)
                .ok_or(anyhow::anyhow!("Could not remove from queue"))?;

            if !only_markets || current.is_marketplace() {
                for mode in &modes {
                    let next_waypoints =
                        get_nearby_waypoints(&unvisited, (current.x, current.y), mode.radius);

                    // depends on luck what waypoints are chosen first on same cost
                    // next_waypoints.sort_by(|a, b| a.symbol.cmp(&b.symbol));
                    // next_waypoints.sort_by(|a, b| b.symbol.cmp(&a.symbol));
                    // next_waypoints.reverse();

                    for waypoint in next_waypoints.iter() {
                        let distance = distance_between_waypoints(
                            (current.x, current.y),
                            (waypoint.x, waypoint.y),
                        );
                        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;
                        to_visit.push_increase(
                            RouteConnection {
                                start_symbol: current.symbol.clone(),
                                end_symbol: waypoint.symbol.clone(),
                                distance,
                                cost,
                                flight_mode: mode.mode,
                                re_cost: cost,
                            },
                            Reverse((cost * 1000000.0) as i64),
                        );
                    }
                }
            }
        }
        Ok(visited)
    }
}
