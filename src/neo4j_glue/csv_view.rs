use std::{collections::HashMap,
          fs::File,
          io::Write,
          sync::{mpsc::Receiver, Arc},
          thread};

use zip::{write::FileOptions, ZipWriter};

use data::{node_types::EnumNode, HasID, HasUUID, Rel};
use ingest::persist::{DBTr, View, ViewInst};

use engine::Config;

const HYDRATE_SH: &'static str =
r#"#! /bin/bash
export NEO4J_USER=neo4j
export NEO4J_PASS=opus
echo "Preparing to hydrate database"
read -p "Ensure neo4j is stopped and that any database files have been removed. Then press enter."
echo "Importing data"
neo4j-admin import --nodes proc.csv --nodes file.csv --nodes es.csv --nodes pipe.csv --nodes socket.csv --nodes dbinfo.csv --relationships rel.csv --id-type=INTEGER
echo "Data import complete"
read -p "Now start neo4j, wait for it to come up, then press enter."
echo -n "Building indexes..."
cypher-shell -u$NEO4J_USER -p$NEO4J_PASS >/dev/null <<EOF
CREATE INDEX ON :Node(db_id);
CREATE INDEX ON :Process(uuid);
CREATE INDEX ON :File(uuid);
CREATE INDEX ON :EditSession(uuid);
CREATE INDEX ON :Pipe(uuid);
CREATE INDEX ON :Socket(uuid);
EOF
echo "Done"
echo "Database hydrated"
"#;

#[derive(Debug)]
pub struct CSVView {
    id: usize,
}

impl View for CSVView {
    fn new(id: usize) -> CSVView {
        CSVView { id }
    }
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &'static str {
        "CSVView"
    }
    fn desc(&self) -> &'static str {
        "View for writing a static csv files for later consumption."
    }
    fn params(&self) -> HashMap<&'static str, &'static str> {
        hashmap!("path" => "The file to write the csv data to.")
    }
    fn create(
        &self,
        id: usize,
        params: HashMap<String, String>,
        _cfg: &Config,
        stream: Receiver<Arc<DBTr>>,
    ) -> ViewInst {
        let mut out = ZipWriter::new(File::create(&params["path"]).unwrap());
        let thr = thread::spawn(move || {
            out.start_file("hydrate.sh", FileOptions::default().unix_permissions(0o755))
                .unwrap();
            writeln!(out, "{}", HYDRATE_SH).unwrap();

            out.start_file("dbinfo.csv", FileOptions::default())
                .unwrap();
            writeln!(out, ":LABEL,pvm_version:int,source").unwrap();
            writeln!(out, "DBInfo,2,libPVM-{}", ::VERSION).unwrap();

            let mut procs = HashMap::new();
            let mut files = HashMap::new();
            let mut es = HashMap::new();
            let mut pipes = HashMap::new();
            let mut sockets = HashMap::new();
            let mut rels = HashMap::new();

            for evt in stream {
                match *evt {
                    DBTr::CreateNode(ref node) => match *node {
                        EnumNode::Proc(ref p) => {
                            procs.insert(p.get_db_id(), p.clone());
                        }
                        EnumNode::File(ref f) => {
                            files.insert(f.get_db_id(), f.clone());
                        }
                        EnumNode::EditSession(ref e) => {
                            es.insert(e.get_db_id(), e.clone());
                        }
                        EnumNode::Pipe(ref p) => {
                            pipes.insert(p.get_db_id(), p.clone());
                        }
                        EnumNode::Socket(ref s) => {
                            sockets.insert(s.get_db_id(), s.clone());
                        }
                    },
                    DBTr::CreateRel(ref rel) => {
                        rels.insert(rel.get_db_id(), (*rel).clone());
                    }
                    DBTr::UpdateNode(ref node) => match *node {
                        EnumNode::Proc(ref p) => {
                            procs.insert(p.get_db_id(), p.clone());
                        }
                        EnumNode::File(ref f) => {
                            files.insert(f.get_db_id(), f.clone());
                        }
                        EnumNode::EditSession(ref e) => {
                            es.insert(e.get_db_id(), e.clone());
                        }
                        EnumNode::Pipe(ref p) => {
                            pipes.insert(p.get_db_id(), p.clone());
                        }
                        EnumNode::Socket(ref s) => {
                            sockets.insert(s.get_db_id(), s.clone());
                        }
                    },
                }
            }
            out.start_file("inf.csv", FileOptions::default()).unwrap();
            writeln!(
                out,
                "db_id:ID,:START_ID,:END_ID,:TYPE,pvm_op,generating_call,byte_count:int"
            ).unwrap();
            for (_, rel) in rels {
                match rel {
                    Rel::Inf {
                        id,
                        src,
                        dst,
                        pvm_op,
                        generating_call,
                        byte_count,
                    } => writeln!(
                        out,
                        "{},{},{},INF,{:?},\"{}\",{}",
                        id, src, dst, pvm_op, generating_call, byte_count
                    ).unwrap(),
                }
            }
            out.start_file("proc.csv", FileOptions::default()).unwrap();
            writeln!(out, "db_id:ID,:LABEL,uuid,cmdline,pid:int,thin:boolean").unwrap();
            for (_, v) in procs {
                writeln!(
                    out,
                    "{},Node;Process,{},\"{}\",{},{}",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.cmdline.replace("\"", "\"\""),
                    v.pid,
                    v.thin
                ).unwrap();
            }
            out.start_file("file.csv", FileOptions::default()).unwrap();
            writeln!(out, "db_id:ID,:LABEL,uuid,name").unwrap();
            for (_, v) in files {
                writeln!(
                    out,
                    "{},Node;File,{},\"{}\"",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.name
                ).unwrap();
            }
            out.start_file("es.csv", FileOptions::default()).unwrap();
            writeln!(out, "db_id:ID,:LABEL,uuid,name").unwrap();
            for (_, v) in es {
                writeln!(
                    out,
                    "{},Node;EditSession,{},\"{}\"",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.name
                ).unwrap();
            }
            out.start_file("pipe.csv", FileOptions::default()).unwrap();
            writeln!(out, "db_id:ID,:LABEL,uuid,fd:int").unwrap();
            for (_, v) in pipes {
                writeln!(out, "{},Node;Pipe,{},{}", v.get_db_id(), v.get_uuid(), v.fd).unwrap();
            }
            out.start_file("socket.csv", FileOptions::default())
                .unwrap();
            writeln!(out, "db_id:ID,:LABEL,uuid,class:int,path,ip,port:int").unwrap();
            for (_, v) in sockets {
                writeln!(
                    out,
                    "{},Node;Socket,{},{},\"{}\",\"{}\",{}",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.class as i64,
                    v.path,
                    v.ip,
                    v.port
                ).unwrap();
            }
            out.finish().unwrap();
        });
        ViewInst {
            id,
            vtype: self.id,
            params,
            handle: thr,
        }
    }
}
