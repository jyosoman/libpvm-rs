use neo4j::cypher::CypherStream;

use packstream::values::ValueCast;

use std::collections::HashMap;

use data::Node;

pub enum Transact {
    ProcCheck {
        uuid: String,
        pid: i32,
        cmdline: String,
    },
    Exec { uuid: String, cmdline: String },
    Fork {
        par_uuid: String,
        ch_uuid: String,
        ch_pid: i32,
    },
}


pub fn execute(cypher: &mut CypherStream, tr: Transact) -> Result<(), &'static str> {
    match tr {
        Transact::ProcCheck { uuid, pid, cmdline } => {
            proc_check(cypher, &uuid[..], pid, &cmdline[..])
        }
        Transact::Exec { uuid, cmdline } => run_exec(cypher, &uuid[..], &cmdline[..]),
        Transact::Fork {
            par_uuid,
            ch_uuid,
            ch_pid,
        } => run_fork(cypher, &par_uuid[..], &ch_uuid[..], ch_pid),
    }
}

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

pub fn proc_check(
    cypher: &mut CypherStream,
    uuid: &str,
    pid: i32,
    cmdline: &str,
) -> Result<(), &'static str> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    props.insert("pid", pid.from());
    props.insert("cmdline", cmdline.from());

    let result = cypher.run(
        "MERGE (p:Process {uuid: {uuid}})
          ON CREATE SET p.pid = {pid}
          ON CREATE SET p.cmdline = {cmdline}
          ON CREATE SET p.thin = true",
        props,
    );
    cypher.fetch_summary(&result);
    Ok(())
}

pub fn run_exec(cypher: &mut CypherStream, uuid: &str, cmdline: &str) -> Result<(), &'static str> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    props.insert("cmdline", cmdline.from());

    let result = cypher.run(
        "MATCH (p:Process {uuid: {uuid},
                           thin: true})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         SET p.cmdline = {cmdline}
         SET p.thin = false
         UNION
         MATCH (p:Process {uuid: {uuid},
                           thin: false})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         CREATE (q:Process {uuid: p.uuid,
                            pid: p.pid,
                            cmdline: {cmdline},
                            thin: false})
         CREATE (p)-[:INF {class: 'next'}]->(q)",
        props,
    );
    cypher.fetch_summary(&result);
    Ok(())
}

pub fn run_fork(
    cypher: &mut CypherStream,
    par_uuid: &str,
    ch_uuid: &str,
    ch_pid: i32,
) -> Result<(), &'static str> {
    let mut props = HashMap::new();
    props.insert("par_uuid", par_uuid.from());
    props.insert("ch_uuid", ch_uuid.from());
    props.insert("ch_pid", ch_pid.from());

    let result = cypher.run(
        "MATCH (p:Process {uuid: {par_uuid}})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         CREATE (c:Process {uuid: {ch_uuid},
                            pid: {ch_pid},
                            cmdline: p.cmdline,
                            thin: true})
         CREATE (p)-[:INF {class: 'child'}]->(c)",
        props,
    );
    cypher.fetch_summary(&result);
    Ok(())
}
