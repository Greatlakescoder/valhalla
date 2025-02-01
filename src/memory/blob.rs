use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Range;
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

pub fn get_cached_data<K, V>(cache: &Cache<K, V>) -> Vec<V>
where
    K: Hash + Eq + Clone,
    V: Clone + Debug,
{
    let keys: Vec<_> = cache.data.keys().cloned().collect();
    let mut values = Vec::new();
    
    
    for i in 0..=5 {
        if i < keys.len() {
            if let Some(value) = cache.get(&keys[i]) {
                values.push(value);
            }
        }
    }
    values
}


