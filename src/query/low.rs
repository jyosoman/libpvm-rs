use data::node_types::EnumNode;

use neo4j::{Neo4jDB, Neo4jOperations};

use neo4j_glue::{FromDB, UUID2Val};

use uuid::Uuid;

pub fn nodes_by_uuid(cypher: &mut Neo4jDB, uuid: Uuid) -> Vec<EnumNode> {
    cypher
        .run(
            "MATCH (n {uuid: {uuid}})
              RETURN n",
            hashmap!("uuid" => uuid.into_val()),
        )
        .unwrap()
        .first()
        .map(|data| EnumNode::from_value(data).unwrap())
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
