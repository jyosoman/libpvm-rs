use std::{collections::HashMap, io::{BufWriter, Write}, sync::{Arc, mpsc::Receiver}};

use neo4j::Value;

use super::{DBTr, View};

use data::ToDB;

const TR_SIZE: usize = 10_000;

pub struct CypherView<O: Write> {
    out: BufWriter<O>,
}

impl<O: Write> CypherView<O> {
    pub fn new(stream: O) -> Box<Self> {
        Box::new(CypherView {
            out: BufWriter::new(stream),
        })
    }
}

impl<O: Write> View for CypherView<O> {
    fn run(&mut self, stream: Receiver<Arc<DBTr>>) {
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
                        "{{src: {}, dst: {}, type: \"{}\", props: {}}}",
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
            .map(|(_k, (l, p))| format!("{{labels: {}, props: {}}}", l, p))
            .collect::<Vec<String>>();
        for chunk in props.chunks(TR_SIZE) {
            writeln!(
                self.out,
                ":begin
                 UNWIND [{}] as n
                 CALL apoc.create.node(n.labels, n.props) YIELD node
                 RETURN 0;
                 :commit",
                chunk.join(", ")
            ).unwrap();
        }
        writeln!(
            self.out,
            "CREATE INDEX ON :Node(db_id);
             CREATE INDEX ON :Process(uuid);
             CREATE INDEX ON :File(uuid);
             CREATE INDEX ON :EditSession(uuid);
             MERGE (:DBInfo {{pvm_version: 2}});
             CALL db.awaitIndexes();"
        ).unwrap();

        for chunk in rels.chunks(TR_SIZE) {
            writeln!(
                self.out,
                ":begin
                 UNWIND [{}] AS r
                 MATCH (s:Node {{db_id: r.src}}), (d:Node {{db_id: r.dst}})
                 CALL apoc.create.relationship(s, r.type, r.props, d) YIELD rel
                 RETURN 0;
                 :commit",
                chunk.join(", ")
            ).unwrap();
        }
        self.out.flush().unwrap();
    }
}

fn render_labs(labs: &[&str]) -> String {
    format!("[\"{}\"]", labs.join("\", \""))
}

fn render_props(props: &HashMap<&str, Value>) -> String {
    let p: Vec<String> = props.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
    format!("{{{}}}", &p[..].join(", "))
}
