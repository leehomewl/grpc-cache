/// Green-Blue Cache
///
///
use dashmap::DashMap;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use tokio::time::Duration;

pub type Result<T> = std::result::Result<T, CacheError>;

const THROTTLE: Duration = Duration::from_nanos(1);

#[derive(Debug)]
pub struct GreenBlueCache<K, V>
where
    K: Eq + Hash + Sized,
{
    caches: [Arc<DashMap<K, V>>; 2],
    current: RwLock<usize>,
    pending: RwLock<Vec<(K, V)>>,
    nowrite_lock: Mutex<()>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    NotFound,
    CannotSwitch,
    CannotWrite,
}

impl std::fmt::Display for CacheError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_str(&format!("{:?}", self))?;
        Ok(())
    }
}

impl std::error::Error for CacheError {}

impl<K, V> GreenBlueCache<K, V>
where
    K: Eq + Hash + Sized + Clone + Display,
    V: Clone + Display,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            caches: [
                Arc::new(DashMap::with_capacity(capacity)),
                Arc::new(DashMap::with_capacity(capacity)),
            ],
            current: std::sync::RwLock::new(0),
            pending: RwLock::new(Vec::with_capacity(capacity)),
            nowrite_lock: Mutex::new(()),
        }
    }

    pub fn put(&self, key: K, value: V) -> Result<()> {
        // println!("** put {}: {}", &key, &value);
        let i = 1 - *self.current.read().unwrap();
        let mut pending = self.pending.write().unwrap();
        let cache = self.caches[i].clone();
        pending.push((key.clone(), value.clone()));
        cache.insert(key, value);
        // sleep(THROTTLE).await;
        Ok(())
    }

    pub fn get(&self, keys: &[K]) -> Vec<Option<V>> {
        let i = *self.current.read().unwrap();
        let cache = self.caches[i].clone();
        keys.iter()
            .map(|k| cache.get(k).map(|v| v.clone()))
            .collect()
    }

    pub fn flush(&self) -> Result<()> {
        let nowrite_lock = self
            .nowrite_lock
            .try_lock()
            .map_err(|_| CacheError::CannotSwitch)?;
        let i = {
            let mut current = self.current.write().unwrap();
            let i = *current;
            *current = 1 - i;
            i
        };

        while Arc::strong_count(&self.caches[i]) > 1 {
            println!(
                "** flush: wait readers rc={:?}",
                Arc::strong_count(&self.caches[i])
            );
            std::thread::sleep(THROTTLE);
        }
        // assert_eq!(Arc::strong_count(&self.caches[i]), 1);
        //println!("** flush: DONE wait readers");
        // TODO

        // Insert pending items in inactive cache
        let pending = self.pending.read().unwrap();
        let cache = self.caches[i].clone();
        println!("*** {:?} Flushing...", std::thread::current().id());
        for (k, v) in pending.iter() {
            cache.insert(k.clone(), v.clone());
        }
        drop(pending);
        let mut pending = self.pending.write().unwrap();
        pending.clear();
        println!("*** Flush DONE.");
        drop(nowrite_lock);
        Ok(())
    }

    pub fn status(&self) {
        println!("Thread {:?} ************ Green: {}_items {}_shards {}_readers // Blue: {}_items {}_shards {}_readers // Pending: {} Current: {}",
            std::thread::current().id(),
            self.caches[0].len(),
            self.caches[0].shards().len(),
            Arc::strong_count(&self.caches[0]),
            self.caches[1].len(),
            self.caches[1].shards().len(),
            Arc::strong_count(&self.caches[1]),
            self.pending.read().unwrap().len(),
            *self.current.read().unwrap(),
        );
    }
}
