#[macro_use]
extern crate lazy_static;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::time::Instant;
use rand::Rng;
use rand::prelude::ThreadRng;
use tokio;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod gbcache;
use gbcache::GreenBlueCache;

const THROTTLE: Duration = Duration::from_nanos(1);

struct Service {
    cache: GreenBlueCache<i32, i32>,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            cache: GreenBlueCache::default(),
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
        writer(&SERVICE.cache).await
    });


    let ts: Vec<JoinHandle<()>> = (0..10).into_iter()
        .map(|i| tokio::spawn(async move {
            reader(&SERVICE.cache, i).await;
        }))
        .collect();

    for t in ts {
        t.await?;
    };

    t0.await?;
    //sleep(Duration::from_millis(1000)).await;

    Ok(())
}

async fn writer(cache: &GreenBlueCache<i32, i32>) -> gbcache::Result<()> {
    for i in 1..=100_000 {
        cache.put(i, 100 * i)?;
        sleep(THROTTLE).await;

        if i % 999 == 0 {
            cache.status().await;
            cache.flush()?;
            cache.status().await;
        }
    }

    cache.status().await;
    cache.flush()?;
    cache.status().await;

    Ok(())
}

async fn reader(cache: &GreenBlueCache<i32, i32>, reader: usize) -> gbcache::Result<()> {
    let mut start = Instant::now();
    for i in 1..=50_000_000 {
        let k = RNG.with(
            |rng| rng.borrow_mut().gen_range(1..=100_000)
        );
        let v = cache.get(&k);

        if i % 1_000_000 == 0 { // } || v.is_none() {
            println!("Reader {} i: {} lat: {:?} Got {}:{:?}", reader, i, start.elapsed(), k, v);
            start = Instant::now();
        }
    }

    Ok(())
}