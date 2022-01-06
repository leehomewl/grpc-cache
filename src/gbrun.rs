#[macro_use]
extern crate lazy_static;

use rand::prelude::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::time::Instant;
use tokio;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod gbcache;
use gbcache::GreenBlueCache;

mod settings;
use settings::*;

mod metrics;
use metrics::Metrics;

struct Service {
    cache: GreenBlueCache<String, String>,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            cache: GreenBlueCache::with_capacity(WRITE_ITERS as usize),
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
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let t0 = tokio::spawn(
        async { writer(&SERVICE.cache, Duration::ZERO).await }
    );
    t0.await?;

    println!(">>>>>>> SPAWN READERS....");
    let ts: Vec<JoinHandle<()>> = (0..READERS)
        .into_iter()
        .map(|i| {
            tokio::spawn(async move {
                reader(&SERVICE.cache, i).await;
            })
        })
        .collect();

    println!(">>>>>>> START WRITE SCHEDULE....");
    for _ in 0..3 {
        sleep(Duration::from_secs(5)).await;

        let t0 = tokio::spawn(async { 
            writer(&SERVICE.cache, WRITE_THROTTLE).await
        });

        t0.await?;
    }

    for t in ts {
        t.await?;
    }

    Ok(())
}


async fn writer(cache: &GreenBlueCache<String, String>, throttle: Duration) -> gbcache::Result<()> {
    println!(">>>>>>>>>>>>>>>>>>>>>> WRITING INITIATED!!");
    for i in 1..=WRITE_ITERS {
        cache.put(format!("{}", i), format!("@{}", 100 * i as i32))?;
        if !throttle.is_zero() {
            sleep(throttle).await;
        }
        if i % WRITE_FLUSH == 0 {
            cache.flush()?;
        }
    }

    cache.flush()?;
    cache.status();
    println!("<<<<<<<<<<<<<<<<<<<<< WRITE DONE!!");

    Ok(())
}

async fn reader(cache: &GreenBlueCache<String, String>, reader: usize) -> gbcache::Result<()> {
    let mut metrics = Metrics::default();
    for i in 1..=READ_ITERS {
        let start = Instant::now();

        let keys: Vec<String> = (0..BATCH_SIZE)
            .into_iter()
            .map(|_| RNG.with(|rng| format!(
                "{}", rng.borrow_mut().gen_range(1i32..=WRITE_ITERS as i32)))
            )
            .collect();

        let vs = cache.get(keys.as_slice());
        metrics.put(BATCH_SIZE, start.elapsed(), READ_TIMEOUT);
        if !READ_THROTTLE.is_zero() {
            sleep(READ_THROTTLE).await;
        }
        if i % READ_REPORT == 0 {
            // } || v.is_none() {
            cache.status();
            println!(
                "Reader {} i: {} Got {}:{:?} {:?}",
                reader, i, keys[0], vs[0], metrics
            );
        }
    }

    Ok(())
}
