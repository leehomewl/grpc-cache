/// Green-Blue Cache
/// 
/// 
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug)]
pub struct GreenBlueCache<K, V> {
    caches: [HashMap<K, V>; 2],
    readers: Arc<()>,
    reading: usize,
    pending: Vec<(K, V)>,
    write_lock: Mutex<()>,
}

#[derive(Debug, Clone, PartialEq)]
enum CacheError {
    NotFound,
    CannotSwitch,
    CannotWrite,
}

impl<K, V> Default for GreenBlueCache<K, V> {
    fn default() -> Self {
        Self {
            caches: [
                HashMap::new(),
                HashMap::new(),
            ],
            readers: Arc::new(()),
            reading: 0,
            pending: Vec::new(),
            write_lock: Mutex::new(()),
        }
    }
}


impl<K, V> GreenBlueCache<K, V>
where 
    K: Eq + Hash + Clone + Display,
    V: Clone {

    pub fn put(&mut self, key: K, value: V) -> Result<()> {
        if let Ok(_lock) = self.write_lock.lock() {
            println!("** put {}", &key);
            self.pending.push((key.clone(), value.clone()));
            self.caches[1 - self.reading].insert(key, value);
            Ok(())
        } else {
            Err(CacheError::CannotWrite)
        }
    }

    pub fn get(&mut self, key: &K) -> Result<V> {
        let rc = self.readers.clone();
        println!("** get: reading {}, rc {:?}", &key, Arc::strong_count(&rc));
        self.caches[self.reading].get(key)
            .map_or_else(
                || Err(CacheError::NotFound),
                |v| Ok(v.clone())
            )
    }

    pub fn flush(&mut self) -> Result<()> {
        println!("** flush");
        if let Ok(_lock) = self.write_lock.lock() {
            // Switch reading pointer
            let updating = self.reading;
            self.reading = 1 - self.reading;
            // From now on new readers will use the new cache

            // Wait for readers on the old map to finish
            let active_readers = Arc::strong_count(&self.readers);
            println!("** switch: readers {:?}", active_readers);
            assert_eq!(active_readers, 1);
            // TODO

            // Insert pending items in inactive cache
            for (k, v) in self.pending.iter() {
                self.caches[updating].insert(k.clone(), v.clone());
            }
            self.pending.clear();

            Ok(())
        } else {
            Err(CacheError::CannotSwitch)
        }
        
    }

}

#[cfg(test)]
mod tests {
    use super::GreenBlueCache;

    #[test]
    fn test_write_read() {

        let mut cache = GreenBlueCache::default();
        println!("{:?}", cache);

        assert_eq!(cache.put(1, 100), Ok(()));
        println!("{:?}", cache);
        assert_eq!(cache.flush(), Ok(()));
        println!("{:?}", cache);


        assert_eq!(cache.get(&1), Ok(100));

        assert_eq!(cache.put(2, 200), Ok(()));
        println!("{:?}", cache);
        assert_eq!(cache.flush(), Ok(()));
        println!("{:?}", cache);

        assert_eq!(cache.get(&2), Ok(200));        

        assert_eq!(cache.get(&1), Ok(100));

    }
}
