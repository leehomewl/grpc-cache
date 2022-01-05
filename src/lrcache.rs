use std::collections::{HashMap, hash_map::Iter};
use std::hash::Hash;
use std::sync::Arc;

use dashmap::DashMap;
use left_right::{Absorb, ReadHandle, WriteHandle};

enum CacheError {

}

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug, Default)]
pub struct Cache<K, V> {
    data: HashMap<K, V>
}

impl<K, V> Cache<K, V> 
where 
    K: Eq + Hash + Clone,
    V: Clone 
{
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).map(|v| v.clone())
    }

    fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }

    fn extend(&mut self, from: &Self) {
        for (k, v) in from.data.iter() {
            self.insert(k.clone(), v.clone());
        }
    }
}

pub struct AddOpp<K, V>(pub K, pub V);

impl<K, V> Absorb<AddOpp<K, V>> for Cache<K, V> 
where 
    K: Eq + Hash + Clone,
    V: Clone 
{
    fn absorb_first(&mut self, operation: &mut AddOpp<K, V>, _: &Self) {
        self.insert(operation.0.clone(), operation.1.clone());
    }

    fn absorb_second(&mut self, operation: AddOpp<K, V>, _: &Self) {
        self.insert(operation.0.clone(), operation.1.clone());
    }

    fn drop_first(self: Box<Self>) {}

    fn sync_with(&mut self, first: &Self) {
        println!("Initial sync!!!!");
        self.extend(first);
    }
}

pub struct CacheWriter<K: Eq + Hash + Clone, V: Clone>(WriteHandle<Cache<K, V>, AddOpp<K, V>>);
impl<K, V> CacheWriter<K, V>
where 
    K: Eq + Hash + Clone,
    V: Clone 
{
    pub fn put(&mut self, k: K, v: V) {
        self.0.append(AddOpp(k, v));
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

pub struct CacheReader<K: Eq + Hash + Clone, V: Clone>(ReadHandle<Cache<K, V>>);
impl<K, V> CacheReader<K, V>
where 
    K: Eq + Hash + Clone,
    V: Clone 
{
    pub fn get(&self, key: &K) -> Option<V> {
        self.0.enter().map(|guard| guard.get(key)).unwrap_or(None)
    }
}

pub fn new<K, V>() -> (CacheWriter<K, V>, CacheReader<K, V>)
where 
    K: Default + Eq + Hash + Clone,
    V: Default + Clone 
{
    let (write, read) = left_right::new::<Cache<K, V>, AddOpp<K, V>>();
    let w = CacheWriter(write);
    let r = CacheReader(read);
    (w, r)
}

#[cfg(test)]
mod tests {
     use super::*;

     #[test]
     fn test_write_read() {
        let (mut w, r) = new();

        println!(">> Empty");
        assert_eq!(None, r.get(&1));

        println!(">> Insert");
        w.put(1, 100);
        assert_eq!(None, r.get(&1));
    
        println!(">> Flush");
        w.flush();
        assert_eq!(Some(100), r.get(&1));
    
        println!(">> Insert");
        w.put(2, 200);
        w.put(3, 300);
        assert_eq!(None, r.get(&2));
        assert_eq!(None, r.get(&3));
    
        println!(">> Flush");
        w.flush();
        assert_eq!(Some(100), r.get(&1));
        assert_eq!(Some(200), r.get(&2));
        assert_eq!(Some(300), r.get(&3));

        println!(">> Insert");
        w.put(4, 400);
        w.put(5, 500);
        assert_eq!(None, r.get(&4));
        assert_eq!(None, r.get(&5));
    
        println!(">> Flush");
        w.flush();
        assert_eq!(Some(400), r.get(&4));
        assert_eq!(Some(500), r.get(&5));

        println!(">> Update");
        w.put(1, 1000);
        w.put(2, 2000);
        assert_eq!(Some(100), r.get(&1));
        assert_eq!(Some(200), r.get(&2));
    
        println!(">> Flush");
        w.flush();
        assert_eq!(Some(1000), r.get(&1));
        assert_eq!(Some(2000), r.get(&2));

        println!(">> Flush");
        w.flush();
        assert_eq!(Some(1000), r.get(&1));
        assert_eq!(Some(2000), r.get(&2));
        assert_eq!(Some(300), r.get(&3));
        assert_eq!(Some(400), r.get(&4));
        assert_eq!(Some(500), r.get(&5));
     }

}