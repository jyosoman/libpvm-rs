use std::{
    collections::HashMap, fs::File, io::Write, sync::{mpsc::Receiver, Arc}, thread,
};

use zip::{write::FileOptions, ZipWriter};

use cfg::Config;
use data::{
    node_types::{DataNode, NameNode, Node}, rel_types::Rel, HasDst, HasID, HasSrc, HasUUID, ID,
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
    fn _lab(&self) -> &'static str;
    fn write_header(&self, f: &mut impl Write);
    fn write_self(&self, f: &mut impl Write);
}

impl ToCSV for Node {
    fn fname(&self) -> &'static str {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => "es.csv",
                DataNode::File(_) => "file.csv",
                DataNode::FileCont(_) => "file_c.csv",
                DataNode::Pipe(_) => "pipe.csv",
                DataNode::Proc(_) => "proc.csv",
                DataNode::Ptty(_) => "ptty.csv",
                DataNode::Socket(_) => "socket.csv",
            },
            Node::Name(n) => match n {
                NameNode::Path(..) => "paths.csv",
                NameNode::Net(..) => "net.csv",
            },
        }
    }

    fn _lab(&self) -> &'static str {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => "Node;EditSession",
                DataNode::File(_) => "Node;File",
                DataNode::FileCont(_) => "Node;FileCont",
                DataNode::Pipe(_) => "Node;Pipe",
                DataNode::Proc(_) => "Node;Process",
                DataNode::Ptty(_) => "Node;Ptty",
                DataNode::Socket(_) => "Node;Socket",
            },
            Node::Name(n) => match n {
                NameNode::Path(..) => "Node;Name;Path",
                NameNode::Net(..) => "Node;Name;Net",
            },
        }
    }

    fn write_header(&self, f: &mut impl Write) {
        write!(f, "db_id:ID,:LABEL").unwrap();
        match self {
            Node::Data(d) => {
                write!(f, ",uuid").unwrap();
                match d {
                    DataNode::Pipe(_) => writeln!(f, ",fd:int").unwrap(),
                    DataNode::Proc(_) => writeln!(f, ",cmdline,pid:int,thin:boolean").unwrap(),
                    DataNode::Socket(_) => writeln!(f, ",class:int").unwrap(),
                    _ => writeln!(f).unwrap(),
                }
            }
            Node::Name(n) => match n {
                NameNode::Path(..) => writeln!(f, ",path").unwrap(),
                NameNode::Net(..) => writeln!(f, ",addr,port:integer").unwrap(),
            },
        }
    }

    fn write_self(&self, f: &mut impl Write) {
        write!(f, "{},{}", self.get_db_id(), self._lab()).unwrap();
        match self {
            Node::Data(d) => {
                write!(f, ",{}", d.get_uuid()).unwrap();
                match d {
                    DataNode::Pipe(v) => writeln!(f, ",{}", v.fd).unwrap(),
                    DataNode::Proc(v) => writeln!(
                        f,
                        ",\"{}\",{},{}",
                        v.cmdline.replace("\"", "\"\""),
                        v.pid,
                        v.thin
                    ).unwrap(),
                    DataNode::Socket(v) => writeln!(f, ",{}", v.class as i64,).unwrap(),
                    _ => writeln!(f).unwrap(),
                }
            }
            Node::Name(n) => match n {
                NameNode::Path(_, path) => writeln!(f, ",\"{}\"", path).unwrap(),
                NameNode::Net(_, addr, port) => writeln!(f, ",\"{}\",{}", addr, port).unwrap(),
            },
        }
    }
}

impl ToCSV for Rel {
    fn fname(&self) -> &'static str {
        match self {
            Rel::Inf(_) => "inf.csv",
            Rel::Named(_) => "named.csv",
        }
    }

    fn _lab(&self) -> &'static str {
        match self {
            Rel::Inf(_) => "INF",
            Rel::Named(_) => "NAMED",
        }
    }

    fn write_header(&self, f: &mut impl Write) {
        write!(f, "db_id,:START_ID,:END_ID,:TYPE").unwrap();
        match self {
            Rel::Inf(_) => writeln!(f, ",pvm_op,generating_call,byte_count:int").unwrap(),
            Rel::Named(_) => writeln!(f, ",start:int,generating_call").unwrap(),
        }
    }

    fn write_self(&self, f: &mut impl Write) {
        write!(
            f,
            "{},{},{},{}",
            self.get_db_id(),
            self.get_src(),
            self.get_dst(),
            self._lab(),
        ).unwrap();
        match self {
            Rel::Inf(i) => writeln!(
                f,
                ",{:?},\"{}\",{}",
                i.pvm_op, i.generating_call, i.byte_count
            ).unwrap(),
            Rel::Named(n) => writeln!(
                f,
                ",{},\"{}\"",
                n.start.timestamp_nanos(),
                n.generating_call
            ).unwrap(),
        }
    }
}
