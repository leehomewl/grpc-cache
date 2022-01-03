#[macro_use]
extern crate lazy_static;

mod gbcache;
use gbcache::GreenBlueCache;

lazy_static! {
    static ref CACHE: GreenBlueCache<String, String> = GreenBlueCache::default();
}


fn main() {
    println!("Hello, world!");
}
