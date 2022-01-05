#[macro_use]
extern crate lazy_static;

use left_right::{ReadHandle, WriteHandle};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::time::Instant;
use tokio;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod lrcache;
use lrcache::*;

mod settings;
use settings::*;

mod metrics;
use metrics::Metrics;

// fn main() {
//     let (mut w, r) = lrcache::new::<i32, i32>();

//     println!("None={:?}", r.get(&1));
//     w.put(1, 100);
//     println!("None={:?}", r.get(&1));

//     w.flush();
//     println!("Some(100)={:?}", r.get(&1));

//     w.put(2, 200);
//     w.put(3, 300);
//     println!("None={:?}", r.get(&2));
//     println!("None={:?}", r.get(&3));

//     w.flush();
//     println!("Some(100)={:?}", r.get(&1));
//     println!("Some(200)={:?}", r.get(&2));
//     println!("Some(300)={:?}", r.get(&3));
// }

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug)]
enum CacheError {}

// lazy_static! {
//     static ref SERVICE: Service = Service::default();
// }

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (mut write, read) = lrcache::new::<i32, i32>();
    let write_ref = Arc::new(Mutex::new(write));

    let keep_alive = write_ref.clone();
    // let t0 = tokio::spawn(async move { writer(&mut write, Duration::ZERO).await });

    // t0.await?;

    // let r = read.clone();
    // let tr = tokio::spawn(async move {
    //     reader(r, 999).await
    // });

    // tr.await?;

    println!(">>>>>>> SPAWN READERS....");
    let ts: Vec<JoinHandle<()>> = (0..READERS)
        .into_iter()
        .map(|i| {
            let cache = read.clone();
            tokio::spawn(async move {
                reader(cache, i).await;
            })
        })
        .collect();

    let w = write_ref.clone();
    let t0 = tokio::spawn(async { writer(w, Duration::ZERO) });

    t0.await?;

    // sleep(Duration::from_millis(20000)).await;

    // println!(">>>>>>> SPAWN WRITER2....");
    // let t1 = tokio::spawn(async {
    //     writer(&service.write.clone(), WRITE_THROTTLE).await
    // });

    for t in ts {
        t.await?;
    }

    // t0.await?;
    // t1.await?;

    drop(keep_alive);

    Ok(())
}

fn writer(cache: Arc<Mutex<CacheWriter<i32, i32>>>, throttle: Duration) -> Result<()> {
    let cache = cache.clone();
    let mut cache = cache.lock().unwrap();
    for i in 1..=WRITE_ITERS {
        cache.put(i, 100 * i as i32);
        // if !throttle.is_zero() {
        //     sleep(throttle).await;
        // }
        if i % WRITE_FLUSH == 0 {
            if i % 1_000_000 == 0 {
                println!("Flushing...");
            }
            cache.flush();
            if i % 1_000_000 == 0 {
                println!("Flush DONE.");
            }
        }
    }

    println!("Flushing...");
    cache.flush();
    println!("Flush DONE.");
    // cache.status();

    Ok(())
}

async fn reader(cache: CacheReader<i32, i32>, reader: usize) -> Result<()> {
    let mut metrics = Metrics::default();
    for i in 1..=READ_ITERS {
        let start = Instant::now();

        let keys: Vec<i32> = (0..BATCH_SIZE)
            .into_iter()
            .map(|_| RNG.with(|rng| rng.borrow_mut().gen_range(1i32..=WRITE_ITERS as i32)))
            .collect();

        let vs = cache.get(keys.as_slice());
        metrics.put(BATCH_SIZE, start.elapsed(), READ_TIMEOUT);
        if !READ_THROTTLE.is_zero() {
            sleep(READ_THROTTLE).await;
        }
        if i % READ_REPORT == 0 {
            // } || v.is_none() {
            // cache.status();
            println!(
                "Reader {} i: {} Got {}:{:?} {:?}",
                reader, i, keys[0], vs[0], metrics
            );
        }
    }

    Ok(())
}
