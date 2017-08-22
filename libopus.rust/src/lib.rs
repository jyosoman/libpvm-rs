#![feature(try_from)]
#![feature(i128_type)]
#![feature(slice_patterns)]

extern crate futures_cpupool;
extern crate libc;
extern crate neo4j;
extern crate num_cpus;
extern crate packstream;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

pub use c_api::*;

pub mod ingest;
pub mod iostream;
pub mod c_api;
pub mod trace;
pub mod data;
pub mod persist;
pub mod query;
pub mod value_as;
pub mod invbloom;
pub mod uuid;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
