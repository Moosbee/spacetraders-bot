pub mod server;

/// **DO NOT USE THIS**
///
/// **The most insecure way to clone a `DashMap`**
///
/// **use with caution**
///
/// Clones a `DashMap` into a `HashMap` without locking the map.
fn unsafe_clone<K: Clone + Eq + std::hash::Hash, V: Clone>(
    map: &dashmap::DashMap<K, V>,
) -> std::collections::HashMap<K, V> {
    let mut cloned_map = std::collections::HashMap::new();

    let shards = &*map.shards();

    for shard in shards {
        // Directly access the data without locking.
        let guard = unsafe { shard.make_read_guard_unchecked() };

        // Clone each key-value pair into the new HashMap.
        unsafe {
            for kv in guard.iter() {
                let kkv = kv.as_ref();
                cloned_map.insert(kkv.0.clone(), kkv.1.clone().into_inner());
            }
        }
    }

    cloned_map
}
