use std::borrow::{BorrowMut, Borrow};
/// Green-Blue Cache
/// 
/// 
use dashmap::DashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::time::Duration;

pub type Result<T> = std::result::Result<T, CacheError>;

const THROTTLE: Duration = Duration::from_nanos(1);

#[derive(Debug)]
pub struct GreenBlueCache<K, V>
where K: Eq + Hash + Sized {
    green: Arc<DashMap<K, V>>,
    blue:  Arc<DashMap<K, V>>,
    pending: Arc<RwLock<Vec<(K, V)>>>,
    refs: Arc<RwLock<ReadWriteRef<K, V>>>,
}

#[derive(Debug)]
struct ReadWriteRef<K, V> 
where K: Eq + Hash + Sized {
    read: Arc<DashMap<K, V>>,
    write: Arc<DashMap<K, V>>,
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

impl<K, V> Default for GreenBlueCache<K, V> 
where K: Eq + Hash + Sized {
    fn default() -> Self {
        let green = Arc::new(DashMap::new());
        let blue = Arc::new(DashMap::new());
        let read = green.clone();
        let write = blue.clone();
        Self {
            green,
            blue,
            refs: Arc::new(RwLock::new(ReadWriteRef {
                read,
                write,
            })),
            pending: Arc::new(RwLock::new(Vec::new())),
        }
    }
}


impl<K, V> GreenBlueCache<K, V>
where 
    K: Eq + Hash + Sized + Clone + Display,
    V: Clone + Display {

    pub fn put(&self, key: K, value: V) -> Result<()> {
        let mut pending = self.pending.write().unwrap();
        let cache = self.refs.clone().read().unwrap().write.clone();
        pending.push((key.clone(), value.clone()));
        cache.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let cache = self.refs.clone().read().unwrap().read.clone();
        cache.get(key).map(|v| v.clone())
    }

    pub fn flush(&self) -> Result<()> {
        let mut pending = self.pending.write().unwrap();
        {
            let rc = self.refs.clone();
            let mut refs = rc.write().unwrap();
            let read = refs.read.clone();
            let write = refs.write.clone();
            refs.read = write;
            refs.write = read;
        }
        // From now on new readers will use the new cache

        // Wait for readers on the old map to finish
        //println!("** flush: wait readers rc={:?}", Arc::strong_count(&refs.write));
        let cache = self.refs.clone().read().unwrap().write.clone();
        while Arc::strong_count(&cache) > 3 {
            std::thread::sleep(THROTTLE);
        }
        // assert_eq!(Arc::strong_count(&self.caches[i]), 1);
        //println!("** flush: DONE wait readers");
        // TODO

        // Insert pending items in inactive cache
        for (k, v) in pending.iter() {
             cache.insert(k.clone(), v.clone());
        }
        pending.clear();

        Ok(())
    }

    pub fn status(&self) {
        println!("************ Green: {}_items {}_readers // Blue: {}_items {}_readers // Pending: {} Read: {}",
            self.green.len(),
            Arc::strong_count(&self.green),
            self.blue.len(),
            Arc::strong_count(&self.blue),
            self.pending.read().unwrap().len(),
            self.refs.read().unwrap().read.len(),
        );
    }

}