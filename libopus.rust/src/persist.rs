use neo4j::cypher::CypherStream;

use packstream::values::ValueCast;

use std::collections::HashMap;
use std::sync::mpsc;

use data::Node;
use trace::TraceEvent;
use invbloom::InvBloom;
use uuid::Uuid5;

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
}

pub fn cache_check(
    tr: &TraceEvent,
    send: &mut mpsc::SyncSender<Transact>,
    proc_cache: &InvBloom,
) -> Result<(), &'static str> {
    if !proc_cache.check(&tr.subjprocuuid) {
        send.send(Transact::ProcCheck {
            uuid: tr.subjprocuuid.clone(),
            pid: tr.pid,
            cmdline: tr.exec.clone().ok_or("other missing exec")?,
        }).map_err(|_| "Database worker closed queue unexpectadly")
    } else {
        Ok(())
    }
}

pub fn parse_trace(
    tr: &TraceEvent,
    send: &mut mpsc::SyncSender<Transact>,
    proc_cache: &InvBloom,
) -> Result<(), &'static str> {
    cache_check(tr, send, proc_cache)?;
    match &tr.event[..] {
        "audit:event:aue_execve:" => {
            send.send(Transact::Exec {
                uuid: tr.subjprocuuid.clone(),
                cmdline: tr.cmdline.clone().ok_or("exec missing cmdline")?,
            }).map_err(|_| "Database worker closed queue unexpectadly")
        }
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => {
            let ret_objuuid1 = tr.ret_objuuid1.clone().ok_or("fork missing ret_objuuid1")?;
            proc_cache.check(&ret_objuuid1);
            send.send(Transact::Fork {
                par_uuid: tr.subjprocuuid.clone(),
                ch_uuid: ret_objuuid1,
                ch_pid: tr.retval,
            }).map_err(|_| "Database worker closed queue unexpectadly")
        }
        _ => Ok(()),
    }
}

pub fn execute(cypher: &mut CypherStream, tr: &Transact) -> Result<(), String> {
    match *tr {
        Transact::ProcCheck {
            ref uuid,
            pid,
            ref cmdline,
        } => proc_check(cypher, uuid, pid, &cmdline[..]),
        Transact::Exec {
            ref uuid,
            ref cmdline,
        } => run_exec(cypher, uuid, &cmdline[..]),
        Transact::Fork {
            ref par_uuid,
            ref ch_uuid,
            ch_pid,
        } => run_fork(cypher, par_uuid, ch_uuid, ch_pid),
    }
}

pub fn persist_node(cypher: &mut CypherStream, node: &Node) -> Result<(), String> {
    cypher.run_unchecked(
        "MERGE (p:Process {db_id: {db_id}})
         SET p.uuid = {uuid}
         SET p.cmdline = {cmdline}
         SET p.pid = {pid}
         SET p.thin = {thin}",
        node.get_props(),
    );
    /*match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }*/
    Ok(())
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

    cypher.run_unchecked(
        "MERGE (p:Process {uuid: {uuid}})
          ON CREATE SET p.pid = {pid}
          ON CREATE SET p.cmdline = {cmdline}
          ON CREATE SET p.thin = true
         RETURN 0",
        props,
    );
    /*match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }*/
    Ok(())
}

pub fn run_exec(cypher: &mut CypherStream, uuid: &Uuid5, cmdline: &str) -> Result<(), String> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    props.insert("cmdline", cmdline.from());

    cypher.run_unchecked(
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
    /*match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }*/
    Ok(())
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

    cypher.run_unchecked(
        "MATCH (p:Process {uuid: {par_uuid}})
         WHERE NOT (p)-[:INF {class: 'next'}]->()
         CREATE (c:Process {uuid: {ch_uuid},
                            pid: {ch_pid},
                            cmdline: p.cmdline,
                            thin: true})
         CREATE (p)-[:INF {class: 'child'}]->(c)
         RETURN 0",
        props,
    );
    /*match result {
        Ok(res) => {
            cypher.fetch_summary(&res);
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }*/
    Ok(())
}
