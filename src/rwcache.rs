use std::borrow::{BorrowMut, Borrow};
/// Locking Single Cache
/// 
/// 
use dashmap::DashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub type Result<T> = std::result::Result<T, CacheError>;

const THROTTLE: Duration = Duration::from_nanos(1);

#[derive(Debug)]
pub struct RwCache<K, V> 
where K: Eq + Hash + Sized {
    cache: Arc<DashMap<K, V>>,
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

impl<K, V> Default for RwCache<K, V>
where K: Eq + Hash + Sized {
        fn default() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }
}


impl<K, V> RwCache<K, V>
where 
    K: Eq + Hash + Clone + Display,
    V: Clone + Display {

    pub fn put(&self, key: K, value: V) -> Result<()> {
        // println!("** put {}: {}", &key, &value);
        let cache = &self.cache.clone();
        cache.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let cache = self.cache.clone();
        // println!("** get: current {}, readers {:?}", &key, Arc::strong_count(&rc));
        let result = cache.get(key).map(|v| v.clone());
        result
    }

    pub fn status(&self) {
        println!("************ Cache: {}_items {}_readers {}_shards",
            self.cache.len(),
            Arc::strong_count(&self.cache),
            self.cache.shards().len()
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
