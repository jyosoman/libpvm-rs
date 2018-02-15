use packstream::values::Value;

use neo4j::bolt::BoltSummary;
use neo4j::cypher::CypherStream;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use uuid::Uuid5;

pub enum DBTr {
    CreateNode {
        id: i64,
        uuid: Uuid5,
        pid: i32,
        cmdline: String,
    },
    CreateNodes(Value),
    CreateRel { src: i64, dst: i64, class: String },
    CreateRels(Value),
    UpdateNode { id: i64, pid: i32, cmdline: String },
    UpdateNodes(Value),
}

pub fn execute_loop(mut db: CypherStream, recv: Receiver<DBTr>) {
    let mut trs = 0;
    let mut nodes: Vec<HashMap<&str, Value>> = Vec::new();
    let mut edges: Vec<HashMap<&str, Value>> = Vec::new();
    let mut update: Vec<HashMap<&str, Value>> = Vec::new();

    const BATCH_SIZE: usize = 1000;

    db.begin_transaction(None);
    for tr in recv {
        let mut props = HashMap::new();
        match tr {
            DBTr::CreateNode { id, uuid, pid, cmdline } => {
                props.insert("db_id", id.into());
                props.insert("uuid", uuid.into());
                props.insert("pid", pid.into());
                props.insert("cmdline", cmdline.into());
                nodes.push(props.into());
            }
            DBTr::CreateRel { src, dst, class } => {
                props.insert("src", src.into());
                props.insert("dst", dst.into());
                props.insert("class", class.into());
                edges.push(props.into());
            }
            DBTr::UpdateNode { id, pid, cmdline } => {
                props.insert("db_id", id.into());
                props.insert("pid", pid.into());
                props.insert("cmdline", cmdline.into());
                update.push(props.into());
            }
            _ => {}
        }
        if nodes.len() >= BATCH_SIZE {
            execute(&mut db, DBTr::CreateNodes(nodes.clone().into())).unwrap();
            nodes.clear();
        }
        if edges.len() >= BATCH_SIZE {
            execute(&mut db, DBTr::CreateRels(edges.clone().into())).unwrap();
            edges.clear();
        }
        if update.len() >= BATCH_SIZE {
            execute(&mut db, DBTr::UpdateNodes(update.clone().into())).unwrap();
            update.clear();
        }
        trs += 1;
   }
    execute(&mut db, DBTr::CreateNodes(nodes.into())).unwrap();
    execute(&mut db, DBTr::CreateRels(edges.into())).unwrap();
    execute(&mut db, DBTr::UpdateNodes(update.into())).unwrap();
    println!("Final Commit");
    match db.commit_transaction() {
        Some(s) => {
            match s {
                BoltSummary::Failure(m) => println!("Error: Commit failed due to {:?}", m),
                BoltSummary::Ignored(_) => unreachable!(),
                BoltSummary::Success(_) => {}
            }
        }
        None => println!("Error: Database commit failed to produce a summary."),
    };
    println!("Neo4J Queries Issued: {}", trs);
}

fn execute(cypher: &mut CypherStream, tr: DBTr) -> Result<(), String> {
    let mut props = HashMap::new();
    match tr {
        DBTr::CreateNode {
            id,
            uuid,
            pid,
            cmdline,
        } => {
            props.insert("db_id", id.into());
            props.insert("uuid", uuid.into());
            props.insert("pid", pid.into());
            props.insert("cmdline", cmdline.into());
            Ok(cypher.run_unchecked(
                "CREATE (p:Process {db_id: $db_id,
                                    uuid: $uuid,
                                    pid: $pid,
                                    cmdline: $cmdline})",
                props,
            ))
        }
        DBTr::CreateNodes(val) => {
            props.insert("nodes", val);
            Ok(cypher.run_unchecked(
                "UNWIND $nodes AS props
                 CREATE (n: Process) SET n = props",
                props,
            ))
        }
        DBTr::CreateRel { src, dst, class } => {
            props.insert("src", src.into());
            props.insert("dst", dst.into());
            props.insert("class", class.into());
            Ok(cypher.run_unchecked(
                "MATCH (s:Process {db_id: $src}),
                       (d:Process {db_id: $dst})
                 CREATE (s)-[:INF {class: $class}]->(d)",
                props,
            ))
        }
        DBTr::CreateRels(val) => {
            props.insert("rels", val);
            Ok(cypher.run_unchecked(
                "UNWIND $rels AS props
                 MATCH (s:Process {db_id: props.src}),
                       (d:Process {db_id: props.dst})
                 CREATE (s)-[:INF {class: props.class}]->(d)",
                props,
            ))
        }
        DBTr::UpdateNode { id, pid, cmdline } => {
            props.insert("db_id", id.into());
            props.insert("pid", pid.into());
            props.insert("cmdline", cmdline.into());
            Ok(cypher.run_unchecked(
                "MATCH (p:Process {db_id: $db_id})
                 SET p.pid = $pid
                 SET p.cmdline = $cmdline",
                props,
            ))
        }
        DBTr::UpdateNodes(val) => {
            props.insert("upds", val);
            Ok(cypher.run_unchecked(
                "UNWIND $upds AS props
                 MATCH (p:Process {db_id: props.db_id})
                 SET p += props",
                props,
            ))
        }
    }
}
