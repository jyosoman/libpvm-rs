use packstream::values::Value;

use neo4j::bolt::BoltSummary;
use neo4j::cypher::CypherStream;

use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use data::NodeID;

pub enum DBTr {
    CreateNode(NodeID, Value, Value),
    CreateRel(Value),
    UpdateNode(NodeID, Value),
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
    fn execute(&mut self, db: &mut CypherStream) {
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
    fn execute(&mut self, db: &mut CypherStream) {
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
    fn execute(&mut self, db: &mut CypherStream) {
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

pub fn execute_loop(mut db: CypherStream, recv: Receiver<DBTr>) {
    let mut ups = 0;
    let mut qrs = 0;
    let mut trs = 0;
    let mut nodes = CreateNodes::new();
    let mut edges = CreateRels::new();
    let mut update = UpdateNodes::new();

    const BATCH_SIZE: usize = 1000;
    const TR_SIZE: usize = 100000;

    db.run_unchecked("CREATE INDEX ON :Node(db_id)", HashMap::new());
    db.run_unchecked("CREATE INDEX ON :Process(uuid)", HashMap::new());
    db.run_unchecked("CREATE INDEX ON :File(uuid)", HashMap::new());
    db.run_unchecked("CREATE INDEX ON :EditSession(uuid)", HashMap::new());

    db.begin_transaction(None);
    for tr in recv {
        match tr {
            DBTr::CreateNode(id, labs, props) => {
                nodes.add(id, hashmap!("labels" => labs, "props"  => props));
                ups += 1;
            }
            DBTr::CreateRel(props) => {
                edges.add(props);
                ups += 1;
            }
            DBTr::UpdateNode(id, props) => {
                if let Some(props) = nodes.update(id, props) {
                    update.add(props);
                    ups += 1;
                }
            }
        }
        if ups >= qrs * BATCH_SIZE {
            nodes.execute(&mut db);
            edges.execute(&mut db);
            update.execute(&mut db);
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
    nodes.execute(&mut db);
    edges.execute(&mut db);
    update.execute(&mut db);
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
