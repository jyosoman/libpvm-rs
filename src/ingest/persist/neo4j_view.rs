use neo4j::{Neo4jDB, Neo4jOperations, Value};

use std::{collections::{HashMap, hash_map::Entry}, sync::{Arc, mpsc::Receiver}};

use data::{NodeID, HasID, HasUUID, node_types::EnumNode};

use super::{DBTr, View};

const BATCH_SIZE: usize = 1000;
const TR_SIZE: usize = 100_000;

pub struct Neo4JView {
    db: Neo4jDB,
}

impl Neo4JView {
    pub fn new(db: Neo4jDB) -> Box<Self> {
        Box::new(Neo4JView { db })
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

        self.db
            .run_unchecked("CREATE INDEX ON :Node(db_id)", HashMap::new());
        self.db
            .run_unchecked("CREATE INDEX ON :Process(uuid)", HashMap::new());
        self.db
            .run_unchecked("CREATE INDEX ON :File(uuid)", HashMap::new());
        self.db
            .run_unchecked("CREATE INDEX ON :EditSession(uuid)", HashMap::new());

        self.db
            .run_unchecked("MERGE (:DBInfo {pvm_version: 2})", hashmap!());

        let mut tr = self.db.transaction();
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

pub trait ToDB: HasID + HasUUID {
    fn get_labels(&self) -> Vec<&'static str>;
    fn get_props(&self) -> HashMap<&'static str, Value>;
    fn to_db(&self) -> (NodeID, Vec<&'static str>, HashMap<&'static str, Value>) {
        let mut props = self.get_props();
        props.insert("db_id", self.get_db_id().into());
        props.insert("uuid", self.get_uuid().into());
        (self.get_db_id(), self.get_labels(), props)
    }
}

impl ToDB for EnumNode {
    fn get_labels(&self) -> Vec<&'static str> {
        match *self {
            EnumNode::EditSession(_) => vec!["Node", "EditSession"],
            EnumNode::File(_) => vec!["Node", "File"],
            EnumNode::Pipe(_) => vec!["Node", "Pipe"],
            EnumNode::Proc(_) => vec!["Node", "Process"],
            EnumNode::Socket(_) => vec!["Node", "Socket"],
        }
    }
    fn get_props(&self) -> HashMap<&'static str, Value> {
        match *self {
            EnumNode::EditSession(ref e) => hashmap!("name"  => Value::from(e.name.clone())),
            EnumNode::File(ref f) => hashmap!("name"  => Value::from(f.name.clone())),
            EnumNode::Pipe(ref p) => hashmap!("fd"    => Value::from(p.fd)),
            EnumNode::Proc(ref p) => hashmap!("cmdline" => Value::from(p.cmdline.clone()),
                                              "pid"     => Value::from(p.pid),
                                              "thin"    => Value::from(p.thin)),
            EnumNode::Socket(ref s) => hashmap!("class"  => Value::from(s.class as i64),
                                                "path" => Value::from(s.path.clone()),
                                                "ip" => Value::from(s.ip.clone()),
                                                "port" => Value::from(s.port)),
        }
    }
}
