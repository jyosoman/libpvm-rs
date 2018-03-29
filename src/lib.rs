#![feature(try_from)]
#![feature(slice_patterns)]
#![feature(box_patterns)]

extern crate futures_cpupool;
extern crate libc;
#[macro_use]
extern crate maplit;
extern crate neo4j;
extern crate num_cpus;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

macro_rules! timeit {
    ($E:expr) => {
        {
            use std::time::Instant;
            let now = Instant::now();
            $E;
            let dur = now.elapsed();
            eprintln!("{} took {:.3}",
                stringify!($E),
                dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) * 1e-9);
        }
    }
}

pub use c_api::*;

pub mod ingest;
pub mod iostream;
pub mod c_api;
pub mod trace;
pub mod data;
pub mod query;
pub mod invbloom;
pub mod uuid;
pub mod checking_store;
