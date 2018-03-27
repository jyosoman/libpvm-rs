use neo4j::{Neo4jDB, Neo4jOperations, Value};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use data::NodeID;

use super::{View, DBTr};

const BATCH_SIZE: usize = 1000;
const TR_SIZE: usize = 100_000;

pub struct Neo4JView {
    db: Neo4jDB,
}

impl Neo4JView {
    pub fn new(mut db: Neo4jDB) -> Self {
        db.run_unchecked("CREATE INDEX ON :Node(db_id)", HashMap::new());
        db.run_unchecked("CREATE INDEX ON :Process(uuid)", HashMap::new());
        db.run_unchecked("CREATE INDEX ON :File(uuid)", HashMap::new());
        db.run_unchecked("CREATE INDEX ON :EditSession(uuid)", HashMap::new());

        db.run_unchecked("MERGE (:DBInfo {pvm_version: 2})", hashmap!());

        Neo4JView{
            db,
        }
    }
}

impl View for Neo4JView {
    fn run(&mut self, stream: Receiver<Arc<DBTr>>) {
        let mut nodes = CreateNodes::new();
        let mut edges = CreateRels::new();
        let mut updates = UpdateNodes::new();
        let mut ups = 0;
        let mut btc = 0;
        let mut trs = 0;
        let mut tr = self.db.transaction();
        for evt in stream {
            match (*evt).clone() {
                DBTr::CreateNode {
                    id, labs, props
                } => {
                    nodes.add(id, hashmap!("labels" => labs.into(), "props"  => props.into()));
                    ups += 1;
                }
                DBTr::CreateRel { src, dst, ty, mut props } => {
                    props.insert("src", src.into());
                    props.insert("dst", dst.into());
                    props.insert("type", ty.into());
                    edges.add(props.into());
                    ups += 1;
                }
                DBTr::UpdateNode { id, props } => {
                    if let Some(props) = nodes.update(id, props.into()) {
                        updates.add(props);
                        ups += 1;
                    }
                }
            }
            if ups > (btc + 1) * BATCH_SIZE {
                nodes.execute(&mut tr);
                edges.execute(&mut tr);
                updates.execute(&mut tr);
                btc += 1;
            }
            if ups > (trs + 1) * TR_SIZE {
                tr.commit_and_refresh().unwrap();
                trs += 1;
            }
        }
        nodes.execute(&mut tr);
        edges.execute(&mut tr);
        updates.execute(&mut tr);
        println!("Final Commit");
        tr.commit().unwrap();
        trs += 1;
        println!("Neo4J Updates Issued: {}", ups);
        println!("Neo4J Batches Issued: {}", btc*3);
        println!("Neo4J Transactions Issued: {}", trs);
    }
}


struct CreateNodes {
    nodes: HashMap<NodeID, HashMap<&'static str, Value>>,
}

impl CreateNodes {
    fn new() -> Self {
        CreateNodes {
            nodes: HashMap::new(),
        }
    }
    fn execute<T: Neo4jOperations>(&mut self, db: &mut T) {
        let nodes: Value = self.nodes.drain().map(|(_k, v)| v).collect();
        db.run_unchecked(
            "UNWIND $nodes AS props
             CALL apoc.create.node(props.labels, props.props) YIELD node
             RETURN 0",
            hashmap!("nodes" => nodes),
        );
    }
    fn add(&mut self, id: NodeID, data: HashMap<&'static str, Value>) {
        self.nodes.insert(id, data);
    }
    fn update(&mut self, id: NodeID, props: Value) -> Option<Value> {
        match self.nodes.entry(id) {
            Entry::Occupied(mut ent) => {
                ent.get_mut().insert("props", props);
                None
            }
            Entry::Vacant(_) => Some(props),
        }
    }
}

struct CreateRels {
    rels: Vec<Value>,
}

impl CreateRels {
    fn new() -> Self {
        CreateRels { rels: Vec::new() }
    }
    fn execute<T: Neo4jOperations>(&mut self, db: &mut T) {
        db.run_unchecked(
            "UNWIND $rels AS props
             MATCH (s:Node {db_id: props.src}),
                   (d:Node {db_id: props.dst})
             CALL apoc.create.relationship(s, props.type, {class: props.class}, d) YIELD rel
             RETURN 0",
            hashmap!("rels" => self.rels.drain(..).collect()),
        );
    }
    fn add(&mut self, value: Value) {
        self.rels.push(value);
    }
}

struct UpdateNodes {
    props: Vec<Value>,
}

impl UpdateNodes {
    fn new() -> Self {
        UpdateNodes { props: Vec::new() }
    }
    fn execute<T: Neo4jOperations>(&mut self, db: &mut T) {
        db.run_unchecked(
            "UNWIND $upds AS props
             MATCH (p:Node {db_id: props.db_id})
             SET p += props",
            hashmap!("upds" => self.props.drain(..).collect()),
        );
    }
    fn add(&mut self, value: Value) {
        self.props.push(value);
    }
}