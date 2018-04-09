use std::{thread, collections::HashMap, fs::File, io::{BufWriter, Write},
          sync::{Arc, mpsc::Receiver}};

use neo4j::Value;

use ingest::persist::{DBTr, View, ViewInst};

use engine::Config;

use neo4j_glue::ToDB;

const TR_SIZE: usize = 10_000;

#[derive(Debug)]
pub struct CypherView {
    id: usize,
}

impl View for CypherView {
    fn new(id: usize) -> CypherView {
        CypherView { id }
    }
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &'static str {
        "CypherView"
    }
    fn desc(&self) -> &'static str {
        "View for writing a static cypher file for later consumption."
    }
    fn params(&self) -> HashMap<&'static str, &'static str> {
        hashmap!("path" => "The file to write the cypher data to.")
    }
    fn create(
        &self,
        id: usize,
        params: HashMap<String, String>,
        _cfg: &Config,
        stream: Receiver<Arc<DBTr>>,
    ) -> ViewInst {
        let mut out = BufWriter::new(File::create(&params["path"]).unwrap());
        let thr = thread::spawn(move || {
            let mut nodes = HashMap::new();
            let mut rels = Vec::new();
            for evt in stream {
                match *evt {
                    DBTr::CreateNode(ref node) => {
                        let (id, labs, props) = node.to_db();
                        nodes.insert(id, (render_labs(&labs), render_props(&props)));
                    }
                    DBTr::CreateRel {
                        src,
                        dst,
                        ty,
                        ref props,
                    } => {
                        rels.push(format!(
                            "MATCH (s:Node {{db_id: {}}}), (d:Node {{db_id: {}}}) CREATE (s)-[:{} {}]->(d);",
                            src,
                            dst,
                            ty,
                            render_props(props)
                        ));
                    }
                    DBTr::UpdateNode(ref node) => {
                        let (id, _, props) = node.to_db();
                        let (l, _) = nodes.remove(&id).unwrap();
                        nodes.insert(id, (l, render_props(&props)));
                    }
                }
            }
            let props = nodes
                .into_iter()
                .map(|(_k, (l, p))| format!("CREATE (n:{} {});", l, p))
                .collect::<Vec<String>>();
            for chunk in props.chunks(TR_SIZE) {
                writeln!(
                    out,
                    ":begin
                     {}
                     :commit",
                    chunk.join("\n")
                ).unwrap();
            }
            writeln!(
                out,
                "CREATE INDEX ON :Node(db_id);
                 CREATE INDEX ON :Process(uuid);
                 CREATE INDEX ON :File(uuid);
                 CREATE INDEX ON :EditSession(uuid);
                 MERGE (:DBInfo {{pvm_version: 2}});
                 CALL db.awaitIndexes();"
            ).unwrap();

            for chunk in rels.chunks(TR_SIZE) {
                writeln!(
                    out,
                    ":begin
                     {}
                     :commit",
                    chunk.join("\n")
                ).unwrap();
            }
            out.flush().unwrap();
        });
        ViewInst {
            id,
            vtype: self.id,
            params,
            handle: thr,
        }
    }
}

fn render_labs(labs: &[&str]) -> String {
    format!("{}", labs.join(":"))
}

fn render_props(props: &HashMap<&str, Value>) -> String {
    let p: Vec<String> = props.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
    format!("{{{}}}", &p[..].join(", "))
}
