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
    CreateRel { src: i64, dst: i64, class: String },
    UpdateNode { id: i64, pid: i32, cmdline: String },
}

pub fn execute_loop(mut db: CypherStream, recv: Receiver<DBTr>) {
    let mut trs = 0;
    db.begin_transaction(None);
    for tr in recv {
        if let Err(e) = execute(&mut db, tr) {
            println!("{}", e);
        }
        trs += 1;
    }
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
    }
}
