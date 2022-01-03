#[macro_use]
extern crate lazy_static;

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

mod gbcache;
use gbcache::GreenBlueCache;

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

#[tokio::main(worker_threads=100)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let t0 = tokio::spawn(async {
        writer(&SERVICE.cache).await
    });

    sleep(Duration::from_millis(10)).await;

    let ts: Vec<JoinHandle<()>> = (0..100).into_iter()
        .map(|i| tokio::spawn(async move {
            reader(&SERVICE.cache, i).await;
        }))
        .collect();

    for t in ts {
        t.await?;
    };
    t0.await?;

    Ok(())
}

async fn writer(cache: &GreenBlueCache<i32, i32>) -> gbcache::Result<()> {
    for i in 1..=100_000 {
        cache.put(i, 100 * i).await?;

        if i % 1000 == 0 {
             cache.status().await;
             cache.flush().await?;
             cache.status().await;
        }
    }

    cache.status().await;
    cache.flush().await?;
    cache.status().await;

    Ok(())
}

async fn reader(cache: &GreenBlueCache<i32, i32>, reader: usize) -> gbcache::Result<()> {
    for i in 1..=1_000_000 {
        let k = i % 100_000 + 1;
        let v = cache.get(&k).await;

        if i % 10_000 == 0 { // } || v.is_none() {
            println!("Reader {} i: {} Got {}:{:?}", reader, i, k, v);
        }
    }

    Ok(())
}