mod csv_view;
mod neo4j_view;

pub use self::{csv_view::CSVView, neo4j_view::Neo4JView};

use std::{borrow::Cow, collections::HashMap, mem};

use neo4j::{Node as NeoNode, Value};

use serde_json;

use data::{
    node_types::{
        DataNode, EditSession, File, NameNode, Node, Pipe, PipeInit, Process, Ptty, Socket,
        SocketClass, SocketInit,
    },
    rel_types::{PVMOps, Rel},
    Enumerable, Generable, HasDst, HasID, HasSrc, HasUUID, MetaStore, ID,
};

use uuid::Uuid;

use chrono::{DateTime, Utc};

pub trait Val2UUID {
    fn into_uuid(self) -> Option<Uuid>;
}

pub trait IntoVal {
    fn into_val(self) -> Value;
}

impl Val2UUID for Value {
    fn into_uuid(self) -> Option<Uuid> {
        match self {
            Value::String(s) => Uuid::parse_str(&s).ok(),
            _ => None,
        }
    }
}

impl IntoVal for Uuid {
    fn into_val(self) -> Value {
        Value::String(self.hyphenated().to_string())
    }
}

impl IntoVal for ID {
    fn into_val(self) -> Value {
        Value::Integer(unsafe { mem::transmute::<u64, i64>(self.inner()) })
    }
}

impl IntoVal for DateTime<Utc> {
    fn into_val(self) -> Value {
        Value::Integer(self.timestamp_nanos())
    }
}

impl IntoVal for PVMOps {
    fn into_val(self) -> Value {
        match self {
            PVMOps::Sink => "Sink".into(),
            PVMOps::Source => "Source".into(),
            PVMOps::Connect => "Connect".into(),
            PVMOps::Version => "Version".into(),
            PVMOps::Unknown => "Unknown".into(),
        }
    }
}

pub trait IntoID {
    fn into_id(self) -> Option<ID>;
}

impl IntoID for Value {
    fn into_id(self) -> Option<ID> {
        match self {
            Value::Integer(i) => Some(ID::new(unsafe { mem::transmute::<i64, u64>(i) })),
            _ => None,
        }
    }
}

fn into_props(meta: &MetaStore) -> HashMap<Cow<'static, str>, Value> {
    let mut ret = HashMap::new();
    for (k, v, _, _) in meta.iter_latest() {
        ret.insert(k.to_string().into(), Value::from(v));
    }
    ret.insert(
        "meta_hist".into(),
        serde_json::to_string(&meta).unwrap().into(),
    );
    ret
}

pub trait ToDBNode: HasID {
    fn get_labels(&self) -> Vec<&'static str>;
    fn get_props(&self) -> HashMap<Cow<'static, str>, Value>;
    fn to_db(&self) -> (ID, Vec<&'static str>, HashMap<Cow<'static, str>, Value>) {
        let mut props = self.get_props();
        props.insert("db_id".into(), self.get_db_id().into_val());
        (self.get_db_id(), self.get_labels(), props)
    }
}

impl ToDBNode for Node {
    fn get_labels(&self) -> Vec<&'static str> {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => vec!["Node", "EditSession"],
                DataNode::File(_) => vec!["Node", "File"],
                DataNode::FileCont(_) => vec!["Node", "FileCont"],
                DataNode::Pipe(_) => vec!["Node", "Pipe"],
                DataNode::Proc(_) => vec!["Node", "Process"],
                DataNode::Socket(_) => vec!["Node", "Socket"],
                DataNode::Ptty(_) => vec!["Node", "Ptty"],
            },
            Node::Name(n) => match n {
                NameNode::Path(..) => vec!["Node", "Name", "Path"],
                NameNode::Net(..) => vec!["Node", "Name", "Net"],
            },
        }
    }

    fn get_props(&self) -> HashMap<Cow<'static, str>, Value> {
        match self {
            Node::Data(d) => {
                let mut props = match d {
                    DataNode::EditSession(_) => hashmap!(),
                    DataNode::File(_) => hashmap!(),
                    DataNode::FileCont(_) => hashmap!(),
                    DataNode::Pipe(p) => hashmap!("fd".into()    => Value::from(p.fd)),
                    DataNode::Proc(p) => into_props(&p.meta),
                    DataNode::Socket(s) => hashmap!("class".into()  => Value::from(s.class as i64)),
                    DataNode::Ptty(_) => hashmap!(),
                };
                props.insert("uuid".into(), d.get_uuid().into_val());
                props
            }
            Node::Name(n) => match n {
                NameNode::Path(_, path) => hashmap!("path".into() => Value::from(path.clone())),
                NameNode::Net(_, addr, port) => {
                    hashmap!("addr".into() => Value::from(addr.clone()),
                                                         "port".into() => Value::from(*port))
                }
            },
        }
    }
}

pub trait ToDBRel {
    fn to_db(&self) -> (ID, Value);
}

impl ToDBRel for Rel {
    fn to_db(&self) -> (ID, Value) {
        match self {
            Rel::Inf(i) => {
                let props: HashMap<&str, Value> = hashmap!("db_id" => i.get_db_id().into_val(),
                                                           "pvm_op" => i.pvm_op.into_val(),
                                                           "generating_call" => Value::from(i.generating_call.clone()),
                                                           "byte_count" => Value::from(i.byte_count));
                (
                    i.get_db_id(),
                    hashmap!("src" => i.get_src().into_val(),
                             "dst" => i.get_dst().into_val(),
                             "type" => Value::from("INF"),
                             "props" => Value::from(props))
                        .into(),
                )
            }
            Rel::Named(n) => {
                let props: HashMap<&str, Value> = hashmap!("db_id" => n.get_db_id().into_val(),
                                                           "generating_call" => Value::from(n.generating_call.clone()),
                                                           "start" => n.start.into_val());
                (
                    n.get_db_id(),
                    hashmap!("src" => n.get_src().into_val(),
                             "dst" => n.get_dst().into_val(),
                             "type" => Value::from("NAMED"),
                             "props" => Value::from(props))
                        .into(),
                )
            }
        }
    }
}

pub trait FromDB {
    fn from_value(val: Value) -> Result<Self, &'static str>
    where
        Self: Sized;
}

impl FromDB for DataNode {
    fn from_value(val: Value) -> Result<Self, &'static str> {
        let mut g = NeoNode::from_value(val)?;

        let id = g
            .props
            .remove("db_id")
            .and_then(Value::into_id)
            .ok_or("db_id property is missing or not an Integer")?;
        let uuid = g
            .props
            .remove("uuid")
            .and_then(Value::into_uuid)
            .ok_or("uuid property is missing or not a UUID5")?;

        if g.labs.contains(&String::from("Process")) {
            Ok(Process::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("File")) {
            Ok(File::new(id, uuid, None).enumerate())
        } else if g.labs.contains(&String::from("EditSession")) {
            Ok(EditSession::new(id, uuid, None).enumerate())
        } else if g.labs.contains(&String::from("Socket")) {
            Ok(Socket::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("Pipe")) {
            Ok(Pipe::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("Ptty")) {
            Ok(Ptty::new(id, uuid, None).enumerate())
        } else {
            Err("Node doesn't match any known type.")
        }
    }
}

trait IntoInit<T> {
    fn into_init(self) -> Result<T, &'static str>;
}

impl IntoInit<PipeInit> for NeoNode {
    fn into_init(mut self) -> Result<PipeInit, &'static str> {
        Ok(PipeInit {
            fd: self
                .props
                .remove("fd")
                .and_then(Value::into_int)
                .ok_or("fd property is missing or not an Integer")?,
        })
    }
}

impl IntoInit<MetaStore> for NeoNode {
    fn into_init(mut self) -> Result<MetaStore, &'static str> {
        Ok(serde_json::from_str(
            &self
                .props
                .remove("meta_hist")
                .and_then(Value::into_string)
                .ok_or("meta_hist parameter missing")?,
        ).unwrap())
    }
}

impl IntoInit<SocketInit> for NeoNode {
    fn into_init(mut self) -> Result<SocketInit, &'static str> {
        Ok(SocketInit {
            class: self
                .props
                .remove("class")
                .and_then(Value::into_int)
                .and_then(SocketClass::from_int)
                .ok_or("class property is missing or not an Integer")?,
        })
    }
}
