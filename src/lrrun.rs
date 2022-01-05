#[macro_use]
extern crate lazy_static;

use left_right::{ReadHandle, WriteHandle};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::time::Instant;
use tokio;
use tokio::time::{sleep, Duration};

mod lrcache;
use lrcache::*;

mod settings;
use settings::*;

mod metrics;
use metrics::Metrics;

fn main() {
    let (mut w, r) = lrcache::new::<i32, i32>();

    println!("None={:?}", r.get(&1));
    w.put(1, 100);
    println!("None={:?}", r.get(&1));

    w.flush();
    println!("Some(100)={:?}", r.get(&1));

    w.put(2, 200);
    w.put(3, 300);
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
