use std::borrow::{BorrowMut, Borrow};
/// Green-Blue Cache
/// 
/// 
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub type Result<T> = std::result::Result<T, CacheError>;

const THROTTLE: Duration = Duration::from_nanos(1);

#[derive(Debug)]
pub struct GreenBlueCache<K, V> {
    caches: [Arc<RwLock<HashMap<K, V>>>; 2],
    current: RwLock<usize>,
    pending: RwLock<Vec<(K, V)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    NotFound,
    CannotSwitch,
    CannotWrite,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         formatter.write_str(&format!("{:?}", self))?;
         Ok(())
    }
}

impl std::error::Error for CacheError {}

impl<K, V> Default for GreenBlueCache<K, V> {
    fn default() -> Self {
        Self {
            caches: [
                Arc::new(RwLock::new(HashMap::new())),
                Arc::new(RwLock::new(HashMap::new())),
            ],
            current: RwLock::new(0),
            pending: RwLock::new(Vec::new()),
        }
    }
}


impl<K, V> GreenBlueCache<K, V>
where 
    K: Eq + Hash + Clone + Display,
    V: Clone + Display {

    pub async fn put(&self, key: K, value: V) -> Result<()> {
        // println!("** put {}: {}", &key, &value);
        let i = 1 - *self.current.read().await;
        let mut pending = self.pending.write().await;
        let rc = self.caches[i].clone();
        let mut cache = rc.write().await;
        pending.push((key.clone(), value.clone()));
        cache.insert(key, value);
        // sleep(THROTTLE).await;
        Ok(())
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        // let current = self.current.read().await;
        // let i = *current;
        // drop(current);

        let i = *self.current.read().await;
        let rc = self.caches[i].clone();
        let cache = rc.read().await;
        // println!("** get: current {}, readers {:?}", &key, Arc::strong_count(&rc));
        let result = cache.get(key).map(|v| v.clone());
        // drop(rc); // Ensures rc lives until here
        result
    }

    pub async fn flush(&self) -> Result<()> {
        let mut pending = self.pending.write().await;
        let i = {
            let mut current = self.current.write().await;
            let i = *current;
            *current = 1 - i;
            i
        };
        // From now on new readers will use the new cache

        // Wait for readers on the old map to finish
        let rc = self.caches[i].clone();
        println!("** flush: wait readers rc={:?}", Arc::strong_count(&self.caches[i]));
        while Arc::strong_count(&self.caches[i]) > 2 {
            sleep(THROTTLE).await;
        }
        assert_eq!(Arc::strong_count(&self.caches[i]), 2);
        println!("** flush: DONE wait readers");
        // TODO

        // Insert pending items in inactive cache
        let mut cache = rc.write().await;
        for (k, v) in pending.iter() {
            cache.insert(k.clone(), v.clone());
        }
        pending.clear();

        Ok(())
    }

    pub async fn status(&self) {
        println!("Green: {}_items {}_readers // Blue: {}_items {}_readers // Pending: {} Current: {}",
            self.caches[0].read().await.len(),
            Arc::strong_count(&self.caches[0]),
            self.caches[1].read().await.len(),
            Arc::strong_count(&self.caches[1]),
            self.pending.read().await.len(),
            *self.current.read().await,
        );
    }

}

// #[cfg(test)]
// mod tests {
//     use super::GreenBlueCache;

//     #[test]
//     fn test_write_read() {

//         let mut cache = GreenBlueCache::default();
//         println!("{:?}", cache);

//         assert_eq!(cache.put(1, 100), Ok(()));
//         println!("{:?}", cache);
//         assert_eq!(cache.flush(), Ok(()));
//         println!("{:?}", cache);


//         assert_eq!(cache.get(&1), Some(100));

//         assert_eq!(cache.put(2, 200), Ok(()));
//         assert_eq!(cache.put(3, 300), Ok(()));
//         println!("{:?}", cache);
//         assert_eq!(cache.flush(), Ok(()));
//         println!("{:?}", cache);

//         assert_eq!(cache.get(&3), Some(300));        
//         assert_eq!(cache.get(&2), Some(200));        
//         assert_eq!(cache.get(&1), Some(100));

//     }
// }
