#![feature(try_from)]

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

mod ingest {}
pub mod iostream;
pub mod c_api;
pub mod trace;
pub mod data;
pub mod persist;
pub mod query;
pub mod value_as;

#[cfg(test)]
mod tests {
    use super::*;
    use super::query::low;
    use neo4j::cypher::CypherStream;
    use std::time::Instant;

    const NANO: f64 = 1e9_f64;

    #[test]
    fn it_works() {}

    #[test]
    fn test_cypher() {
        let p = data::Node::Process(data::ProcessNode {
            db_id: data::NodeID(0),
            uuid: String::from("0000-0000-0000"),
            cmdline: String::from("./foo"),
            pid: 2,
            thin: false,
        });
        let q = data::Node::Process(data::ProcessNode {
            db_id: data::NodeID(1),
            uuid: String::from("0000-0000-0000"),
            cmdline: String::from("./bar"),
            pid: 1,
            thin: false,
        });
        let mut cypher = CypherStream::connect("localhost:7687", "neo4j", "opus").unwrap();
        let i = Instant::now();
        persist::persist_node(&mut cypher, &q);
        persist::persist_node(&mut cypher, &p);
        println!("{:?}", (i.elapsed().subsec_nanos() as f64) / NANO);
        let foo = low::nodes_by_uuid(&mut cypher, "0000-0000-0000");
        println!("{:?}", foo);
    }
}
