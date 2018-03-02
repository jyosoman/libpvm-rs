use packstream::values::Value;

use neo4j::bolt::BoltSummary;
use neo4j::cypher::CypherStream;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

pub enum DBTr {
    CreateNode(Value, Value),
    CreateNodes(Value),
    CreateRel(Value),
    CreateRels(Value),
    UpdateNode(Value),
    UpdateNodes(Value),
}

pub fn execute_loop(mut db: CypherStream, recv: Receiver<DBTr>) {
    let mut ups = 0;
    let mut qrs = 0;
    let mut trs = 0;
    let mut nodes: Vec<Value> = Vec::new();
    let mut edges: Vec<Value> = Vec::new();
    let mut update: Vec<Value> = Vec::new();

    const BATCH_SIZE: usize = 1000;
    const TR_SIZE: usize = 100000;

    db.begin_transaction(None);
    for tr in recv {
        match tr {
            DBTr::CreateNode(labs, props) => {
                let mut prs: HashMap<&'static str, Value> = HashMap::new();
                prs.insert("labels", labs);
                prs.insert("props", props);
                nodes.push(prs.into());
            }
            DBTr::CreateRel(props) => {
                edges.push(props);
            }
            DBTr::UpdateNode(props) => {
                update.push(props);
            }
            _ => {}
        }
        ups += 1;
        if ups >= qrs * BATCH_SIZE {
            execute(&mut db, DBTr::CreateNodes(nodes.clone().into()));
            nodes.clear();
            execute(&mut db, DBTr::CreateRels(edges.clone().into()));
            edges.clear();
            execute(&mut db, DBTr::UpdateNodes(update.clone().into()));
            update.clear();
            qrs += 1;
        }
        if ups > trs * TR_SIZE {
            match db.commit_transaction() {
                Some(s) => match s {
                    BoltSummary::Failure(m) => println!("Error: Commit failed due to {:?}", m),
                    BoltSummary::Ignored(_) => unreachable!(),
                    BoltSummary::Success(_) => {}
                },
                None => println!("Error: Database commit failed to produce a summary."),
            };
            db.begin_transaction(None);
            trs += 1;
        }
    }
    execute(&mut db, DBTr::CreateNodes(nodes.into()));
    execute(&mut db, DBTr::CreateRels(edges.into()));
    execute(&mut db, DBTr::UpdateNodes(update.into()));
    qrs += 1;
    println!("Final Commit");
    match db.commit_transaction() {
        Some(s) => match s {
            BoltSummary::Failure(m) => println!("Error: Commit failed due to {:?}", m),
            BoltSummary::Ignored(_) => unreachable!(),
            BoltSummary::Success(_) => {}
        },
        None => println!("Error: Database commit failed to produce a summary."),
    };
    trs += 1;
    println!("Neo4J Updates Issued: {}", ups);
    println!("Neo4J Queries Issued: {}", qrs * 3);
    println!("Neo4J Transactions Issued: {}", trs);
}

fn execute(cypher: &mut CypherStream, tr: DBTr) {
    let mut props = HashMap::new();
    match tr {
        DBTr::CreateNodes(val) => {
            props.insert("nodes", val);
            cypher.run_unchecked(
                "UNWIND $nodes AS props
                 CALL apoc.create.node(props.labels, props.props) YIELD node
                 RETURN 0",
                props,
            );
        }
        DBTr::CreateRels(val) => {
            props.insert("rels", val);
            cypher.run_unchecked(
                "UNWIND $rels AS props
                 MATCH (s:Node {db_id: props.src}),
                       (d:Node {db_id: props.dst})
                 CREATE (s)-[:INF {class: props.class}]->(d)",
                props,
            );
        }
        DBTr::UpdateNodes(val) => {
            props.insert("upds", val);
            cypher.run_unchecked(
                "UNWIND $upds AS props
                 MATCH (p:Node {db_id: props.db_id})
                 SET p += props",
                props,
            );
        }
        _ => unreachable!(),
    }
}
