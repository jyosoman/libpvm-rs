use neo4j::cypher::CypherStream;

use packstream::values::ValueCast;

use std::collections::HashMap;


use data::Node;
use trace::{Uuid5, TraceEvent};
use invbloom::InvBloom;

pub enum Transact {
    ProcCheck {
        uuid: Uuid5,
        pid: i32,
        cmdline: String,
    },
    Exec { uuid: Uuid5, cmdline: String },
    Fork {
        par_uuid: Uuid5,
        ch_uuid: Uuid5,
        ch_pid: i32,
    },
    Noop,
}

pub fn cache_check(tr: &TraceEvent, proc_cache: &InvBloom) -> Result<Transact, &'static str> {
    if !proc_cache.check(&tr.subjprocuuid) {
        Ok(Transact::ProcCheck {
            uuid: tr.subjprocuuid.clone(),
            pid: tr.pid,
            cmdline: tr.exec.clone().ok_or("other missing exec")?,
        })
    } else {
        Ok(Transact::Noop)
    }
}

pub fn parse_trace(tr: &TraceEvent, proc_cache: &InvBloom) -> Result<Vec<Transact>, &'static str> {
    let mut ret = Vec::with_capacity(2);
    ret.push(cache_check(tr, proc_cache)?);
    match &tr.event[..] {
        "audit:event:aue_execve:" => {
            ret.push(Transact::Exec {
                uuid: tr.subjprocuuid.clone(),
                cmdline: tr.cmdline.clone().ok_or("exec missing cmdline")?,
            });
        }
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => {
            let ret_objuuid1 = tr.ret_objuuid1.clone().ok_or("fork missing ret_objuuid1")?;
            proc_cache.check(&ret_objuuid1);
            ret.push(Transact::Fork {
                par_uuid: tr.subjprocuuid.clone(),
                ch_uuid: ret_objuuid1,
                ch_pid: tr.retval,
            });
        }
        _ => {}
    }
    Ok(ret)
}

pub fn execute(cypher: &mut CypherStream, tr: &Transact) -> Result<(), String> {
    match *tr {
        Transact::ProcCheck {
            ref uuid,
            pid,
            ref cmdline,
        } => proc_check(cypher, &uuid, pid, &cmdline[..]),
        Transact::Exec {
            ref uuid,
            ref cmdline,
        } => run_exec(cypher, &uuid, &cmdline[..]),
        Transact::Fork {
            ref par_uuid,
            ref ch_uuid,
            ch_pid,
        } => run_fork(cypher, &par_uuid, &ch_uuid, ch_pid),
        Transact::Noop => Ok(()),
    }
}

pub fn persist_node(cypher: &mut CypherStream, node: &Node) -> Result<(), String> {
    let result = cypher.run(
        "MERGE (p:Process {db_id: {db_id}})
         SET p.uuid = {uuid}
         SET p.cmdline = {cmdline}
         SET p.pid = {pid}
         SET p.thin = {thin}",
        node.get_props(),
    );
    match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

pub fn proc_check(
    cypher: &mut CypherStream,
    uuid: &Uuid5,
    pid: i32,
    cmdline: &str,
) -> Result<(), String> {
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
    match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

pub fn run_exec(cypher: &mut CypherStream, uuid: &Uuid5, cmdline: &str) -> Result<(), String> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    props.insert("cmdline", cmdline.from());

    let result = cypher.run(
        "MATCH (p:Process {uuid: {uuid},
                           thin: false})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         CREATE (q:Process {uuid: p.uuid,
                            pid: p.pid,
                            cmdline: {cmdline},
                            thin: false})
         CREATE (p)-[:INF {class: 'next'}]->(q)
         RETURN 0
         UNION
         MATCH (p:Process {uuid: {uuid},
                           thin: true})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         SET p.cmdline = {cmdline}
         SET p.thin = false
         RETURN 0",
        props,
    );
    match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

pub fn run_fork(
    cypher: &mut CypherStream,
    par_uuid: &Uuid5,
    ch_uuid: &Uuid5,
    ch_pid: i32,
) -> Result<(), String> {
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
    match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}
