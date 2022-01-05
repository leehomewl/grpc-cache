use std::collections::HashMap;
use std::hash::Hash;

use left_right::{Absorb, ReadHandle, WriteHandle};

struct AddOpp<K, V>(pub K, pub V);

impl<K, V> Absorb<AddOpp<K, V>> for HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
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
        self.extend(first.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
}

pub struct CacheWriter<K: Eq + Hash + Clone, V: Clone>(WriteHandle<HashMap<K, V>, AddOpp<K, V>>);
impl<K, V> CacheWriter<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn put(&mut self, k: K, v: V) {
        self.0.append(AddOpp(k, v));
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

#[derive(Clone)]
pub struct CacheReader<K: Eq + Hash + Clone, V: Clone>(ReadHandle<HashMap<K, V>>);
impl<K, V> CacheReader<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn get(&self, keys: &[K]) -> Vec<Option<V>> {
        if let Some(guard) = self.0.enter() {
            keys.iter()
                .map(|k| guard.get(k).map(|v| v.clone()))
                .collect()
        } else {
            //TODO: Return err result
            keys.iter().map(|_| None).collect()
        }
    }
}

pub fn new<K, V>() -> (CacheWriter<K, V>, CacheReader<K, V>)
where
    K: Default + Eq + Hash + Clone,
    V: Default + Clone,
{
    let (write, read) = left_right::new::<HashMap<K, V>, AddOpp<K, V>>();
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
        assert_eq!(vec![None], r.get(&[1]));

        println!(">> Insert");
        w.put(1, 100);
        assert_eq!(vec![None], r.get(&[1]));

        println!(">> Flush");
        w.flush();
        assert_eq!(vec![Some(100)], r.get(&[1]));

        println!(">> Insert");
        w.put(2, 200);
        w.put(3, 300);
        assert_eq!(vec![None, None], r.get(&[2, 3]));

        println!(">> Flush");
        w.flush();
        assert_eq!(vec![Some(100), Some(200), Some(300)], r.get(&[1, 2, 3]));

        println!(">> Insert");
        w.put(4, 400);
        w.put(5, 500);
        assert_eq!(vec![None, None], r.get(&[4, 5]));

        println!(">> Flush");
        w.flush();
        assert_eq!(vec![Some(400), Some(500)], r.get(&[4, 5]));

        println!(">> Update");
        w.put(1, 1000);
        w.put(2, 2000);
        assert_eq!(vec![Some(100), Some(200)], r.get(&[1, 2]));

        println!(">> Flush");
        w.flush();
        assert_eq!(vec![Some(1000), Some(2000)], r.get(&[1, 2]));

        println!(">> Flush");
        w.flush();
        assert_eq!(
            vec![Some(1000), Some(2000), Some(300), Some(400), Some(500)],
            r.get(&[1, 2, 3, 4, 5])
        );

        //Data vanishes when writer is dropped
        drop(w);
        assert_eq!(vec![None; 5], r.get(&[1, 2, 3, 4, 5]));
    }
}
