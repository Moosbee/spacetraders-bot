use std::{collections::HashMap, hash::Hash};

pub struct DisjointSet<'a, T: Eq + Hash + std::cmp::PartialEq> {
    nodes_to_parents: HashMap<&'a T, &'a T>,
    nodes_child_count: HashMap<&'a T, u32>,
}

impl<'a, T: Eq + Hash + std::cmp::PartialEq> DisjointSet<'a, T> {
    pub fn new() -> Self {
        Self {
            nodes_to_parents: HashMap::new(),
            nodes_child_count: HashMap::new(),
        }
    }

    pub fn make_set(&mut self, node: &'a T) {
        if !self.nodes_to_parents.contains_key(node) {
            self.nodes_to_parents.insert(node, node);
            self.nodes_child_count.insert(node, 1);
        }
    }

    pub fn find_root<'b>(&'b mut self, node: &'a T) -> Option<&'a T> {
        // let mut current_node = node;
        // loop {
        //     let parent = self.nodes_to_parents.get(current_node)?;
        //     if current_node == *parent {
        //         return Some(*parent);
        //     }
        //     current_node = *parent;
        // }

        let parent = *self.nodes_to_parents.get(node)?;

        if parent != node {
            let parent_root = self.find_root(parent)?;

            self.nodes_to_parents.insert(node, parent_root);
        }

        Some(*self.nodes_to_parents.get(node)?)
    }

    pub fn union(&mut self, node_a: &'a T, node_b: &'a T) -> bool {
        self.make_set(node_a);
        self.make_set(node_b);

        let parent_a = self.find_root(node_a).unwrap();
        let parent_b = self.find_root(node_b).unwrap();

        if parent_a == parent_b {
            return false;
        }

        let parent_a_count = self.nodes_child_count.get(parent_a).unwrap();
        let parent_b_count = self.nodes_child_count.get(parent_b).unwrap();

        if parent_a_count < parent_b_count {
            // add parent a to b

            self.nodes_to_parents.insert(parent_a, parent_b);
            self.nodes_child_count
                .insert(parent_b, parent_a_count + parent_b_count);

            return true;
        } else {
            // add parent b to a

            self.nodes_to_parents.insert(parent_b, parent_a);
            self.nodes_child_count
                .insert(parent_a, parent_a_count + parent_b_count);

            return true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DisjointSet;

    #[test]
    fn test_disjoint_set_union() {
        let values = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let v = |i: usize| &values[i];

        let mut dsu = DisjointSet::new();

        dsu.union(v(1), v(2));
        dsu.union(v(2), v(3));
        dsu.union(v(1), v(9));
        dsu.union(v(4), v(5));
        dsu.union(v(7), v(8));
        dsu.union(v(4), v(8));
        dsu.union(v(6), v(9));

        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(2)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(3)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(6)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(9)));

        assert_eq!(dsu.find_root(v(4)), dsu.find_root(v(5)));
        assert_eq!(dsu.find_root(v(4)), dsu.find_root(v(7)));
        assert_eq!(dsu.find_root(v(4)), dsu.find_root(v(8)));

        assert_ne!(dsu.find_root(v(1)), dsu.find_root(v(10)));
        assert_ne!(dsu.find_root(v(4)), dsu.find_root(v(10)));

        dsu.union(v(3), v(4));

        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(2)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(3)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(6)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(9)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(4)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(5)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(7)));
        assert_eq!(dsu.find_root(v(1)), dsu.find_root(v(8)));

        assert_ne!(dsu.find_root(v(1)), dsu.find_root(v(10)));

        dsu.union(v(10), v(1));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(1)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(2)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(3)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(4)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(5)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(6)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(7)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(8)));
        assert_eq!(dsu.find_root(v(10)), dsu.find_root(v(9)));
    }
}
