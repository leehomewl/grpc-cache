#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use std::time::Instant;
use rand::Rng;
use rand::prelude::ThreadRng;
use tokio;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod rwcache;
use rwcache::RwCache;

mod settings;
use settings::*;

mod metrics;
use metrics::Metrics;

struct Service {
    cache: RwCache<i32, i32>,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            cache: RwCache::default(),
        }
    }
}

lazy_static! {
    static ref SERVICE: Service = Service::default();
}

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let t0 = tokio::spawn(async {
        writer(&SERVICE.cache, Duration::ZERO).await
    });
    t0.await?;

    let ts: Vec<JoinHandle<()>> = (0..READERS).into_iter()
        .map(|i| tokio::spawn(async move {
            reader(&SERVICE.cache, i).await;
        }))
        .collect();

    sleep(Duration::from_millis(2000)).await;
    let t0 = tokio::spawn(async {
        writer(&SERVICE.cache, WRITE_THROTTLE).await
    });

    for t in ts {
        t.await?;
    };

    t0.await?;

    Ok(())
}

async fn writer(cache: &RwCache<i32, i32>, throttle: Duration) -> rwcache::Result<()> {
    for i in 1..=WRITE_ITERS {
        cache.put(i, 100 * i)?;
        if !throttle.is_zero() {
            sleep(throttle).await;
        }
    }

    cache.status();

    Ok(())
}

async fn reader(cache: &RwCache<i32, i32>, reader: usize) -> rwcache::Result<()> {
    let mut metrics = Metrics::default();
    for i in 1..=READ_ITERS {
        let start = Instant::now();
        let k = RNG.with(
            |rng| rng.borrow_mut().gen_range(1..=WRITE_ITERS)
        );
        let v = cache.get(&k);
        metrics.put(1, start.elapsed(), READ_TIMEOUT);
        if i % READ_REPORT == 0 { // } || v.is_none() {
            cache.status();
            println!("Reader {} i: {} Got {}:{:?} {:?}", reader, i, k, v, metrics);
            if !READ_THROTTLE.is_zero() {
                sleep(READ_THROTTLE).await;
            }
        }
    }

    Ok(())
}