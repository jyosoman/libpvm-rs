use std::collections::HashMap;

use neo4j::cypher::CypherStream;
use packstream::values::{Value, ValueCast};

use ::data::Process;

pub trait Persistable {
    fn get_props(&self) -> HashMap<&str, Value>;
}

impl Persistable for Process {
    fn get_props(&self) -> HashMap<&str, Value> {
        let mut props = HashMap::new();
        props.insert("db_id", self.db_id.0.from());
        props.insert("uuid", self.uuid.from());
        props.insert("cmdline", self.cmdline.from());
        props.insert("pid", self.pid.from());
        props.insert("thin", self.thin.from());
        props
    }
}

pub fn persist_node<T: Persistable>(
    cypher: &mut CypherStream,
    node: &T,
) -> Result<(), &'static str> {
    let result = cypher.run(
        "MERGE (p:Process {db_id: {db_id}})
         SET p.uuid = {uuid}
         SET p.cmdline = {cmdline}
         SET p.pid = {pid}
         SET p.thin = {thin}",
        node.get_props(),
    );
    cypher.fetch_summary(&result);
    Ok(())
}
