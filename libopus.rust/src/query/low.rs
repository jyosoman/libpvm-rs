use std::collections::{HashMap, VecDeque};

use packstream::values::{Data, ValueCast};
use neo4j::cypher::CypherStream;

use data::Node;


pub fn nodes_by_uuid(cypher: &mut CypherStream, uuid: &str) -> Vec<Node> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    let result = cypher.run(
        "MATCH (n {uuid: {uuid}})
         WITH n
         OPTIONAL MATCH (n)-[e]->(m)
         RETURN n, collect(e)",
        props,
    );
    let mut records: VecDeque<Data> = VecDeque::new();
    while cypher.fetch(&result, &mut records) > 0 {}
    let _ = cypher.fetch_summary(&result);

    let mut ret = Vec::with_capacity(records.len());
    for rec in records.drain(..) {
        match rec {
            Data::Record(mut v) => ret.push(Node::from_value(v.remove(0), v.remove(0)).unwrap()),
        }
    }
    ret
}
