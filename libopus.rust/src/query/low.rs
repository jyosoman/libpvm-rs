use std::collections::{HashMap, VecDeque};

use packstream::values::{Data, Value, ValueCast};
use neo4j::cypher::CypherStream;
use Process;

pub trait Recoverable<T> {
    fn from_node(rec: Value) -> T;
}

impl Recoverable<Process> for Process{
    fn from_node(rec: Value) -> Process{
        match rec {
            Value::Structure{signature, fields} => {
                assert!(signature == 0x4E);
                let props = match fields[2] {
                    Value::Map(ref m) => m,
                    _ => { panic!() },
                };
                let db_id = match props["db_id"] {
                    Value::Integer(i) => i as u64,
                    _ => { panic!() },
                };
                let cmdline = match props["cmdline"] {
                    Value::String(ref s) => s.clone(),
                    _ => { panic!() },
                };
                let uuid = match props["uuid"] {
                    Value::String(ref s) => s.clone(),
                    _ => { panic!() },
                };
                let pid = match props["pid"] {
                    Value::Integer(i) => i as i32,
                    _ => { panic!() },
                };
                let thin = match props["thin"] {
                    Value::Boolean(b) => b,
                    _ => { panic!() },
                };
                Process {
                    db_id: db_id,
                    cmdline: cmdline,
                    uuid: uuid,
                    pid: pid,
                    thin: thin,
                }
            },
            _ => { panic!() },
        }
    }
}

pub fn nodes_by_uuid(cypher: &mut CypherStream, uuid: &str) -> Vec<Process> {
    let mut props = HashMap::new();
    props.insert("uuid", uuid.from());
    let result = cypher.run(
        "MATCH (n {uuid: {uuid}})
         RETURN n",
        props,
    );
    let mut records: VecDeque<Data> = VecDeque::new();
    while cypher.fetch(&result, &mut records) > 0 {}
    let _ = cypher.fetch_summary(&result);

    let mut ret = Vec::with_capacity(records.len());
    for rec in records.drain(..){
        match rec {
            Data::Record(mut v) => ret.push(Process::from_node(v.remove(0))),
        }
    }
    ret
}
