use neo4j::cypher::CypherStream;

use data::Node;

pub fn persist_node(cypher: &mut CypherStream, node: &Node) -> Result<(), &'static str> {
    let result = cypher.run(
        "MERGE (p:Process {db_id: {db_id}})
         SET p.uuid = {uuid}
         SET p.cmdline = {cmdline}
         SET p.pid = {pid}
         SET p.thin = {thin}
         WITH p
         FOREACH (ch IN {chs} |
             MERGE (q:Process {db_id: ch.id})
             MERGE (p)-[e:INF]->(q)
             SET e.class = ch.class)",
        node.get_props(),
    );
    cypher.fetch_summary(&result);
    Ok(())
}
