use std::{
    collections::HashMap, fs::File, io::Write, sync::{mpsc::Receiver, Arc}, thread,
};

use zip::{write::FileOptions, ZipWriter};

use cfg::Config;
use data::{
    node_types::{Node, DataNode}, rel_types::Rel, HasDst, HasID, HasSrc, HasUUID, ID,
};
use views::{DBTr, View, ViewInst};

const HYDRATE_SH_PRE: &str = r#"#! /bin/bash
export NEO4J_USER=neo4j
export NEO4J_PASS=opus

if ! which "neo4j-admin" >/dev/null || ! which "cypher-shell" >/dev/null ; then
    echo "Cannot find neo4j binaries"
    echo "Please make sure that the neo4j binaries are in \$PATH"
    exit 1
fi

echo "Preparing to hydrate database"
read -p "Ensure neo4j is stopped and that any database files have been removed. Then press enter."
echo "Importing data"
"#;

const HYDRATE_SH_POST: &str = r#"echo "Data import complete"
read -p "Now start neo4j, wait for it to come up, then press enter."
echo -n "Building indexes..."
cypher-shell -u$NEO4J_USER -p$NEO4J_PASS >/dev/null <<EOF
CREATE INDEX ON :Node(db_id);
CREATE INDEX ON :Process(uuid);
CREATE INDEX ON :File(uuid);
CREATE INDEX ON :EditSession(uuid);
CREATE INDEX ON :Pipe(uuid);
CREATE INDEX ON :Socket(uuid);
CREATE INDEX ON :Ptty(uuid);
CALL db.awaitIndexes();
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
            out.start_file("dbinfo.csv", FileOptions::default())
                .unwrap();
            writeln!(out, ":LABEL,pvm_version:int,source").unwrap();
            writeln!(out, "DBInfo,2,libPVM-{}", ::VERSION).unwrap();

            let mut nodes: HashMap<&'static str, HashMap<ID, Node>> = HashMap::new();
            let mut rels: HashMap<&'static str, HashMap<ID, Rel>> = HashMap::new();

            for evt in stream {
                match *evt {
                    DBTr::CreateNode(ref node) | DBTr::UpdateNode(ref node) => {
                        nodes
                            .entry(node.fname())
                            .or_insert_with(HashMap::new)
                            .insert(node.get_db_id(), node.clone());
                    }
                    DBTr::CreateRel(ref rel) | DBTr::UpdateRel(ref rel) => {
                        rels.entry(rel.fname())
                            .or_insert_with(HashMap::new)
                            .insert(rel.get_db_id(), rel.clone());
                    }
                }
            }

            out.start_file("hydrate.sh", FileOptions::default().unix_permissions(0o755))
                .unwrap();
            {
                write!(out, "{}", HYDRATE_SH_PRE).unwrap();
                let mut options = vec![
                    "--id-type=INTEGER".to_string(),
                    "--multiline-fields=true".to_string(),
                ];
                options.extend(nodes.keys().map(|k| format!("--nodes {}", k)));
                options.extend(rels.keys().map(|k| format!("--relationships {}", k)));
                writeln!(out, "neo4j-admin import {}", options.join(" "),).unwrap();
                write!(out, "{}", HYDRATE_SH_POST).unwrap();
            }

            for (fname, rlist) in rels {
                out.start_file(fname, FileOptions::default()).unwrap();
                for (i, r) in rlist.values().enumerate() {
                    if i == 0 {
                        r.write_header(&mut out);
                    }
                    r.write_self(&mut out);
                }
            }
            for (fname, nlist) in &nodes {
                out.start_file(*fname, FileOptions::default()).unwrap();
                for (i, n) in nlist.values().enumerate() {
                    if i == 0 {
                        n.write_header(&mut out);
                    }
                    n.write_self(&mut out);
                }
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

trait ToCSV {
    fn fname(&self) -> &'static str;
    fn write_header<F: Write>(&self, f: &mut F);
    fn write_self<F: Write>(&self, f: &mut F);
}

impl ToCSV for Node {
    fn fname(&self) -> &'static str {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => "es.csv",
                DataNode::File(_) => "file.csv",
                DataNode::Pipe(_) => "pipe.csv",
                DataNode::Proc(_) => "proc.csv",
                DataNode::Ptty(_) => "ptty.csv",
                DataNode::Socket(_) => "socket.csv",
            }
        }
    }

    fn write_header<F: Write>(&self, f: &mut F) {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => writeln!(f, "db_id:ID,:LABEL,uuid,name").unwrap(),
                DataNode::File(_) => writeln!(f, "db_id:ID,:LABEL,uuid,name").unwrap(),
                DataNode::Pipe(_) => writeln!(f, "db_id:ID,:LABEL,uuid,fd:int").unwrap(),
                DataNode::Proc(_) => {
                    writeln!(f, "db_id:ID,:LABEL,uuid,cmdline,pid:int,thin:boolean").unwrap()
                }
                DataNode::Ptty(_) => writeln!(f, "db_id:ID,:LABEL,uuid,name").unwrap(),
                DataNode::Socket(_) => {
                    writeln!(f, "db_id:ID,:LABEL,uuid,class:int,path,ip,port:int").unwrap()
                }
            }
        }
    }

    fn write_self<F: Write>(&self, f: &mut F) {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(v) => writeln!(
                    f,
                    "{},Node;EditSession,{},\"{}\"",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.name
                ).unwrap(),
                DataNode::File(v) => writeln!(
                    f,
                    "{},Node;File,{},\"{}\"",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.name
                ).unwrap(),
                DataNode::Pipe(v) => {
                    writeln!(f, "{},Node;Pipe,{},{}", v.get_db_id(), v.get_uuid(), v.fd).unwrap()
                }
                DataNode::Proc(v) => writeln!(
                    f,
                    "{},Node;Process,{},\"{}\",{},{}",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.cmdline.replace("\"", "\"\""),
                    v.pid,
                    v.thin
                ).unwrap(),
                DataNode::Ptty(v) => writeln!(
                    f,
                    "{},Node;Ptty,{},\"{}\"",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.name
                ).unwrap(),
                DataNode::Socket(v) => writeln!(
                    f,
                    "{},Node;Socket,{},{},\"{}\",\"{}\",{}",
                    v.get_db_id(),
                    v.get_uuid(),
                    v.class as i64,
                    v.path,
                    v.ip,
                    v.port
                ).unwrap(),
            }
        }
    }
}

impl ToCSV for Rel {
    fn fname(&self) -> &'static str {
        match self {
            Rel::Inf(_) => "inf.csv",
        }
    }

    fn write_header<F: Write>(&self, f: &mut F) {
        write!(f, "db_id,:START_ID,:END_ID,:TYPE").unwrap();
        match self {
            Rel::Inf(_) => writeln!(f, ",pvm_op,generating_call,byte_count:int").unwrap(),
        }
    }

    fn write_self<F: Write>(&self, f: &mut F) {
        write!(
            f,
            "{},{},{}",
            self.get_db_id(),
            self.get_src(),
            self.get_dst(),
        ).unwrap();
        match self {
            Rel::Inf(i) => writeln!(
                f,
                ",INF,{:?},\"{}\",{}",
                i.pvm_op, i.generating_call, i.byte_count
            ).unwrap(),
        }
    }
}
