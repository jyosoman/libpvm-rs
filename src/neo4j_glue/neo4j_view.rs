use neo4j::{Neo4jDB, Neo4jOperations, Value};

use std::{thread, collections::{HashMap, hash_map::Entry}, sync::{Arc, mpsc::Receiver}};

use data::NodeID;

use ingest::persist::{DBTr, View, ViewInst};
use neo4j_glue::ToDB;

const BATCH_SIZE: usize = 1000;
const TR_SIZE: usize = 100_000;

#[derive(Debug)]
pub struct Neo4JView {
    id: usize,
}

impl View for Neo4JView {
    fn new(id: usize) -> Neo4JView {
        Neo4JView { id }
    }
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &'static str {
        "Neo4jView"
    }
    fn desc(&self) -> &'static str {
        "View for streaming data to a Neo4j database instance."
    }
    fn params(&self) -> HashMap<&'static str, &'static str> {
        hashmap!("addr" => "The Neo4j server address to connect to.",
                 "user" => "The username to auth with.",
                 "pass" => "The password to auth with.")
    }
    fn create(
        &self,
        id: usize,
        params: HashMap<String, String>,
        stream: Receiver<Arc<DBTr>>,
    ) -> ViewInst {
        let mut db = Neo4jDB::connect(&params["addr"], &params["user"], &params["pass"]).unwrap();
        let thr = thread::spawn(move || {
            let mut nodes = CreateNodes::new();
            let mut edges = CreateRels::new();
            let mut updates = UpdateNodes::new();
            let mut ups = 0;
            let mut btc = 0;
            let mut trs = 0;

            db.run_unchecked("CREATE INDEX ON :Node(db_id)", HashMap::new());
            db.run_unchecked("CREATE INDEX ON :Process(uuid)", HashMap::new());
            db.run_unchecked("CREATE INDEX ON :File(uuid)", HashMap::new());
            db.run_unchecked("CREATE INDEX ON :EditSession(uuid)", HashMap::new());
            db.run_unchecked("CREATE INDEX ON :Pipe(uuid)", HashMap::new());
            db.run_unchecked("CREATE INDEX ON :Socket(uuid)", HashMap::new());

            db.run_unchecked("MERGE (:DBInfo {pvm_version: 2})", hashmap!());

            let mut tr = db.transaction();
            for evt in stream {
                match *evt {
                    DBTr::CreateNode(ref node) => {
                        let (id, labs, props) = node.to_db();
                        nodes.add(
                            id,
                            hashmap!("labels" => labs.into(), "props"  => props.into()),
                        );
                        ups += 1;
                    }
                    DBTr::CreateRel {
                        src,
                        dst,
                        ty,
                        ref props,
                    } => {
                        let rel: HashMap<&str, Value> = hashmap!("src" => src.into(),
                                                                 "dst" => dst.into(),
                                                                 "type" => ty.into(),
                                                                 "props" => props.clone().into());
                        edges.add(rel.into());
                        ups += 1;
                    }
                    DBTr::UpdateNode(ref node) => {
                        let (id, _, props) = node.to_db();
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
            println!("Neo4J Batches Issued: {}", btc * 3);
            println!("Neo4J Transactions Issued: {}", trs);
        });
        ViewInst {
            id,
            vtype: self.id,
            params,
            handle: thr,
        }
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
            "UNWIND $nodes AS n
             CALL apoc.create.node(n.labels, n.props) YIELD node
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
            "UNWIND $rels AS r
             MATCH (s:Node {db_id: r.src}),
                   (d:Node {db_id: r.dst})
             CALL apoc.create.relationship(s, r.type, r.props, d) YIELD rel
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
