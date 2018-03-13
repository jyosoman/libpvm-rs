use data::node_types::EnumNode;

use neo_wrap::{Neo4j, Neo4jOperations};

use uuid::Uuid5;

use value_as::CastValue;

pub fn nodes_by_uuid(cypher: &mut Neo4j, uuid: Uuid5) -> Vec<EnumNode> {
    cypher
        .run(
            "MATCH (n {uuid: {uuid}})
              RETURN n",
            hashmap!("uuid" => uuid.into()),
        )
        .unwrap()
        .first()
        .map(|data| EnumNode::from_db(data).unwrap())
        .collect()
}

pub fn count_processes(cypher: &mut Neo4j) -> i64 {
    cypher
        .run(
            "MATCH (n:Process)
              RETURN count(n)",
            hashmap!(),
        )
        .unwrap()
        .first()
        .map(|data| data.into_int().unwrap())
        .next()
        .unwrap()
}
