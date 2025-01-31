use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct Cache<K, V> {
    data: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, (value, Instant::now()));
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() <= self.ttl {
                Some(value.clone())
            } else {
                None
            }
        })
    }
    pub fn remove_expired(&mut self) {
        self.data
            .retain(|_, (_, timestamp)| timestamp.elapsed() <= self.ttl);
    }
}


// We can also create functions that work with our cache
// fn process_cached_data<K, V, F>(cache: &mut Cache<K, V>, key: K, generator: F)
// where
//     K: Hash + Eq + Clone,
//     V: Clone,
//     F: FnOnce() -> V,
// {
//     if cache.get(&key).is_none() {
//         let value = generator();
//         cache.insert(key, value);
//     }
// }