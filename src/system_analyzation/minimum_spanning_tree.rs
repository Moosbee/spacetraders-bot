use itertools::Itertools;
use utils::{DisjointSet, WaypointCan, distance_between_waypoints};

#[derive(Debug)]
pub struct SpanningTreeEdge<'a> {
    pub from: &'a database::Waypoint,
    pub to: &'a database::Waypoint,
    pub distance: f64,
}

impl<'a> Eq for SpanningTreeEdge<'a> {}

impl<'a> PartialEq for SpanningTreeEdge<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
            || self.from == other.to && self.to == other.from
    }
}

pub fn gen_minimum_spanning_tree<'a>(
    waypoints: &'a [database::Waypoint],
    only_markets: bool,
) -> Vec<SpanningTreeEdge<'a>> {
    let valid_waypoints = waypoints
        .iter()
        .filter(|w| !only_markets || w.is_marketplace())
        .collect::<Vec<_>>();
    if valid_waypoints.len() < 2 {
        return vec![];
    }
    let all_connections = valid_waypoints
        .iter()
        .enumerate()
        .flat_map(|(i, wp)| {
            valid_waypoints[i + 1..].iter().map(|w| SpanningTreeEdge {
                from: wp,
                to: w,
                distance: distance_between_waypoints((*wp).into(), (*w).into()),
            })
        })
        .sorted_by(|a, b| a.distance.total_cmp(&b.distance))
        .collect::<Vec<_>>();

    let mut disjoint_set = DisjointSet::new();

    let mut spanning_tree = Vec::with_capacity(valid_waypoints.len() - 1);

    for connection in all_connections.into_iter() {
        let was_loop = !disjoint_set.union(&connection.from.symbol, &connection.to.symbol);

        if was_loop {
            continue;
        }

        spanning_tree.push(connection);

        if spanning_tree.len() == valid_waypoints.len() - 1 {
            break;
        }
    }

    spanning_tree
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet, VecDeque};

    use utils::WaypointCan;

    use super::{SpanningTreeEdge, gen_minimum_spanning_tree};

    fn load_waypoints() -> Vec<database::Waypoint> {
        utils::tests::get_waypoints()
    }

    fn waypoint_symbols(waypoints: &[database::Waypoint]) -> HashSet<String> {
        waypoints.iter().map(|w| w.symbol.clone()).collect()
    }

    fn is_connected(symbols: &HashSet<String>, edges: &[SpanningTreeEdge]) -> bool {
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            adjacency
                .entry(edge.from.symbol.clone())
                .or_default()
                .push(edge.to.symbol.clone());
            adjacency
                .entry(edge.to.symbol.clone())
                .or_default()
                .push(edge.from.symbol.clone());
        }

        let start = symbols.iter().next().expect("no waypoints").clone();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([start.clone()]);
        visited.insert(start);

        while let Some(symbol) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(&symbol) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        visited.len() == symbols.len()
    }

    #[test]
    fn mst_spans_all_waypoints() {
        let waypoints = load_waypoints();
        let all_symbols = waypoint_symbols(&waypoints);

        let tree = gen_minimum_spanning_tree(&waypoints, false);

        assert_eq!(
            tree.len(),
            all_symbols.len() - 1,
            "spanning tree over {} waypoints should have {} edges",
            all_symbols.len(),
            all_symbols.len() - 1
        );

        assert!(is_connected(&all_symbols, &tree));

        let mut seen_edges = HashSet::new();
        let mut previous_distance = f64::NEG_INFINITY;

        for edge in &tree {
            let from = &edge.from.symbol;
            let to = &edge.to.symbol;

            assert_ne!(from, to, "self-loop found in spanning tree");
            assert!(all_symbols.contains(from));
            assert!(all_symbols.contains(to));

            let key = if from < to {
                (from.clone(), to.clone())
            } else {
                (to.clone(), from.clone())
            };
            assert!(seen_edges.insert(key), "duplicate edge {from} <-> {to}");

            assert!(
                edge.distance.is_finite() && edge.distance >= 0.0,
                "unexpected distance {}",
                edge.distance
            );

            let expected_distance =
                utils::distance_between_waypoints(edge.from.into(), edge.to.into());
            assert_eq!(edge.distance, expected_distance);

            assert!(
                edge.distance >= previous_distance,
                "edges should be produced in non-decreasing distance order"
            );
            previous_distance = edge.distance;
        }
    }

    #[test]
    fn mst_only_markets_uses_only_market_waypoints() {
        let waypoints = load_waypoints();
        let market_symbols: HashSet<String> = waypoints
            .iter()
            .filter(|w| w.is_marketplace())
            .map(|w| w.symbol.clone())
            .collect();

        let tree = gen_minimum_spanning_tree(&waypoints, true);

        assert_eq!(tree.len(), market_symbols.len() - 1);
        assert!(is_connected(&market_symbols, &tree));

        for edge in &tree {
            assert!(market_symbols.contains(&edge.from.symbol));
            assert!(market_symbols.contains(&edge.to.symbol));
            assert!(edge.distance <= 300.0);
        }
    }

    #[test]
    fn mst_no_waypoints() {
        let waypoints = Vec::new();
        let tree = gen_minimum_spanning_tree(&waypoints, false);
        assert!(tree.is_empty());
    }
}
