extern crate neo4j;
extern crate packstream;

pub use c_api::*;

mod ingest {}
pub mod c_api;
pub mod data;
pub mod persist;
pub mod query;

#[cfg(test)]
mod tests {
    use super::*;
    use super::query::low;
    use neo4j::cypher::CypherStream;

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
            rel: Vec::new(),
        });
        let mut cypher = CypherStream::connect("localhost:7687", "neo4j", "opus").unwrap();
        persist::persist_node(&mut cypher, &p);
        let foo = low::nodes_by_uuid(&mut cypher, "0000-0000-0000");
        println!("{:?}", foo);
    }
}
