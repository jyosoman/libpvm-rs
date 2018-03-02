use std::collections::{HashMap, VecDeque};

use packstream::values::Data;
use neo4j::cypher::CypherStream;

use data::node_types::EnumNode;

pub fn nodes_by_uuid(cypher: &mut CypherStream, uuid: &str) -> Vec<EnumNode> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.into());
    let result = cypher
        .run(
            "MATCH (n {uuid: {uuid}})
              RETURN n",
            props,
        )
        .unwrap();
    let mut records: VecDeque<Data> = VecDeque::new();
    while cypher.fetch(&result, &mut records) > 0 {}
    let _ = cypher.fetch_summary(&result);

    let mut ret = Vec::with_capacity(records.len());
    for rec in records.drain(..) {
        match rec {
            Data::Record(mut v) => {
                ret.push(EnumNode::from_db(v.remove(0)).unwrap());
            }
        }
    }
    ret
}
