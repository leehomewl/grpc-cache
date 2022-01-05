#[macro_use]
extern crate lazy_static;

use std::time::Instant;
use left_right::{WriteHandle, ReadHandle};
use rand::Rng;
use rand::prelude::ThreadRng;
use tokio;
use tokio::time::{sleep, Duration};

mod lrcache;
use lrcache::{Cache, Result, AddOpp};

mod settings;
use settings::*;

mod metrics;
use metrics::Metrics;


struct CacheWriter(WriteHandle<Cache<i32, i32>, AddOpp<i32, i32>>);
impl CacheWriter {
    pub fn insert(&mut self, k: i32, v: i32) {
        self.0.append(AddOpp(k, v));
    }

    pub fn flush(&mut self) {
        self.0.publish();
    }
}

struct CacheReader(ReadHandle<Cache<i32, i32>>);
impl CacheReader {
    pub fn get(&self, key: &i32) -> Option<i32> {
        self.0.enter().map(|guard| guard.get(key)).unwrap_or(None)
    }
}

fn main() {
    let (write, read) = left_right::new::<Cache<i32, i32>, AddOpp<i32, i32>>();
    let mut w = CacheWriter(write);
    let r = CacheReader(read);

    println!("None={:?}", r.get(&1));
    w.insert(1, 100);
    println!("None={:?}", r.get(&1));

    w.flush();
    println!("Some(100)={:?}", r.get(&1));

    w.insert(2, 200);
    w.insert(3, 300);
    println!("None={:?}", r.get(&2));
    println!("None={:?}", r.get(&3));

    w.flush();
    println!("Some(100)={:?}", r.get(&1));
    println!("Some(200)={:?}", r.get(&2));
    println!("Some(300)={:?}", r.get(&3));

}

// struct Service {
//     cache: GreenBlueCache<i32, i32>,
// }

// impl Default for Service {
//     fn default() -> Self {
//         Self {
//             cache: GreenBlueCache::with_capacity(WRITE_ITERS as usize),
//         }
//     }
// }

// lazy_static! {
//     static ref SERVICE: Service = Service::default();
// }

// thread_local! {
//     static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {

//     // let t0 = tokio::spawn(async {
//     //     writer(&SERVICE.cache, Duration::ZERO).await
//     // });

//     // t0.await?;

//     println!(">>>>>>> SPAWN READERS....");
//     let ts: Vec<JoinHandle<()>> = (0..READERS).into_iter()
//         .map(|i| tokio::spawn(async move {
//             reader(&SERVICE.cache, i).await;
//         }))
//         .collect();

//     println!(">>>>>>> SPAWN WRITER1....");
//     let t0 = tokio::spawn(async {
//         writer(&SERVICE.cache, WRITE_THROTTLE).await
//     });

//     sleep(Duration::from_millis(20000)).await;

//     println!(">>>>>>> SPAWN WRITER2....");
//     let t1 = tokio::spawn(async {
//         writer(&SERVICE.cache, WRITE_THROTTLE).await
//     });

//     for t in ts {
//         t.await?;
//     };

//     t0.await?;
//     t1.await?;

//     Ok(())
// }

// async fn writer(cache: &GreenBlueCache<i32, i32>, throttle: Duration) -> gbcache::Result<()> {
//     for i in 1..=WRITE_ITERS {
//         cache.put(i, 100 * i as i32)?;
//         if !throttle.is_zero() {
//             sleep(throttle).await;
//         }
//         if i % WRITE_FLUSH == 0 {
//              cache.flush()?;
//         }
//     }

//     cache.flush()?;
//     cache.status();

//     Ok(())
// }

// async fn reader(cache: &GreenBlueCache<i32, i32>, reader: usize) -> gbcache::Result<()> {
//     let mut metrics = Metrics::default();
//     for i in 1..=READ_ITERS {
//         let start = Instant::now();

//         let keys: Vec<i32> = (0..BATCH_SIZE).into_iter()
//             .map(|_| RNG.with(
//                 |rng| rng.borrow_mut().gen_range(1i32..=WRITE_ITERS as i32)
//             ))
//             .collect();
        
//         let vs = cache.get(keys.as_slice());
//         metrics.put(BATCH_SIZE, start.elapsed(), READ_TIMEOUT);
//         if !READ_THROTTLE.is_zero() {
//             sleep(READ_THROTTLE).await;
//         }
//         if i % READ_REPORT == 0 { // } || v.is_none() {
//             cache.status();
//             println!("Reader {} i: {} Got {}:{:?} {:?}", reader, i, keys[0], vs[0], metrics);
//         }
//     }

//     Ok(())
// }