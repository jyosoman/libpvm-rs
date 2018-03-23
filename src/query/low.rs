use data::node_types::EnumNode;

use neo4j::{Neo4jDB, Neo4jOperations};

use uuid::Uuid5;

pub fn nodes_by_uuid(cypher: &mut Neo4jDB, uuid: Uuid5) -> Vec<EnumNode> {
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

pub fn count_processes(cypher: &mut Neo4jDB) -> i64 {
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
