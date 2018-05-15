mod csv_view;
mod neo4j_view;

pub use self::{csv_view::CSVView, neo4j_view::Neo4JView};

use std::collections::HashMap;

use neo4j::{Node as NeoNode, Value};

use data::{
    node_types::{
        DataNode, EditInit, EditSession, NameNode, Node, File, FileInit, Pipe, PipeInit, Process, ProcessInit,
        Ptty, PttyInit, Socket, SocketClass, SocketInit,
    },
    rel_types::{PVMOps, Rel}, Enumerable, Generable, HasDst, HasID, HasSrc, HasUUID, ID,
};

use uuid::Uuid;

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
        Value::Integer(self.inner())
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
            Value::Integer(i) => Some(ID::new(i)),
            _ => None,
        }
    }
}

pub trait ToDBNode: HasID {
    fn get_labels(&self) -> Vec<&'static str>;
    fn get_props(&self) -> HashMap<&'static str, Value>;
    fn to_db(&self) -> (ID, Vec<&'static str>, HashMap<&'static str, Value>) {
        let mut props = self.get_props();
        props.insert("db_id", self.get_db_id().into_val());
        (self.get_db_id(), self.get_labels(), props)
    }
}

impl ToDBNode for Node {
    fn get_labels(&self) -> Vec<&'static str> {
        match self {
            Node::Data(d) => match d {
                DataNode::EditSession(_) => vec!["Node", "EditSession"],
                DataNode::File(_) => vec!["Node", "File"],
                DataNode::Pipe(_) => vec!["Node", "Pipe"],
                DataNode::Proc(_) => vec!["Node", "Process"],
                DataNode::Socket(_) => vec!["Node", "Socket"],
                DataNode::Ptty(_) => vec!["Node", "Ptty"],
            }
            Node::Name(n) => match n {
                NameNode::Path(..) => vec!["Node", "Name", "Path"],
                NameNode::Net(..) => vec!["Node", "Name", "Net"],
            }
        }
    }
    fn get_props(&self) -> HashMap<&'static str, Value> {
        match self {
            Node::Data(d) => {
                let mut props = match d {
                    DataNode::EditSession(e) => hashmap!("name"  => Value::from(e.name.clone())),
                    DataNode::File(f) => hashmap!("name"  => Value::from(f.name.clone())),
                    DataNode::Pipe(p) => hashmap!("fd"    => Value::from(p.fd)),
                    DataNode::Proc(p) => hashmap!("cmdline" => Value::from(p.cmdline.clone()),
                                                  "pid"     => Value::from(p.pid),
                                                  "thin"    => Value::from(p.thin)),
                    DataNode::Socket(s) => hashmap!("class"  => Value::from(s.class as i64),
                                                    "path" => Value::from(s.path.clone()),
                                                    "ip" => Value::from(s.ip.clone()),
                                                    "port" => Value::from(s.port)),
                    DataNode::Ptty(p) => hashmap!("name"  => Value::from(p.name.clone())),
                };
                props.insert("uuid", d.get_uuid().into_val());
                props
            }
            Node::Name(n) => match n {
                NameNode::Path(_, path) => hashmap!("path" => Value::from(path.clone())),
                NameNode::Net(_, addr, port) => hashmap!("addr" => Value::from(addr.clone()),
                                                         "port" => Value::from(*port)),
            }
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

        let id = g.props
            .remove("db_id")
            .and_then(Value::into_id)
            .ok_or("db_id property is missing or not an Integer")?;
        let uuid = g.props
            .remove("uuid")
            .and_then(Value::into_uuid)
            .ok_or("uuid property is missing or not a UUID5")?;

        if g.labs.contains(&String::from("Process")) {
            Ok(Process::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("File")) {
            Ok(File::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("EditSession")) {
            Ok(EditSession::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("Socket")) {
            Ok(Socket::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("Pipe")) {
            Ok(Pipe::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else if g.labs.contains(&String::from("Ptty")) {
            Ok(Ptty::new(id, uuid, Some(g.into_init()?)).enumerate())
        } else {
            Err("Node doesn't match any known type.")
        }
    }
}

trait IntoInit<T> {
    fn into_init(self) -> Result<T, &'static str>;
}

impl IntoInit<FileInit> for NeoNode {
    fn into_init(mut self) -> Result<FileInit, &'static str> {
        Ok(FileInit {
            name: self.props
                .remove("name")
                .and_then(Value::into_string)
                .ok_or("name property is missing or not a string")?,
        })
    }
}

impl IntoInit<EditInit> for NeoNode {
    fn into_init(mut self) -> Result<EditInit, &'static str> {
        Ok(EditInit {
            name: self.props
                .remove("name")
                .and_then(Value::into_string)
                .ok_or("name property is missing or not a string")?,
        })
    }
}

impl IntoInit<PipeInit> for NeoNode {
    fn into_init(mut self) -> Result<PipeInit, &'static str> {
        Ok(PipeInit {
            fd: self.props
                .remove("fd")
                .and_then(Value::into_int)
                .ok_or("fd property is missing or not an Integer")?,
        })
    }
}

impl IntoInit<ProcessInit> for NeoNode {
    fn into_init(mut self) -> Result<ProcessInit, &'static str> {
        Ok(ProcessInit {
            cmdline: self.props
                .remove("cmdline")
                .and_then(Value::into_string)
                .ok_or("cmdline property is missing or not a String")?,
            pid: self.props
                .remove("pid")
                .and_then(Value::into_int)
                .ok_or("pid property is missing or not an Integer")?,
            thin: self.props
                .remove("thin")
                .and_then(Value::into_bool)
                .ok_or("thin property is missing or not a bool")?,
        })
    }
}

impl IntoInit<SocketInit> for NeoNode {
    fn into_init(mut self) -> Result<SocketInit, &'static str> {
        Ok(SocketInit {
            class: self.props
                .remove("class")
                .and_then(Value::into_int)
                .and_then(SocketClass::from_int)
                .ok_or("class property is missing or not an Integer")?,
            path: self.props
                .remove("path")
                .and_then(Value::into_string)
                .ok_or("path property is missing or not a string")?,
            ip: self.props
                .remove("ip")
                .and_then(Value::into_string)
                .ok_or("ip property is missing or not a string")?,
            port: self.props
                .remove("port")
                .and_then(Value::into_int)
                .ok_or("port property is missing or not an Integer")?,
        })
    }
}

impl IntoInit<PttyInit> for NeoNode {
    fn into_init(mut self) -> Result<PttyInit, &'static str> {
        Ok(PttyInit {
            name: self.props
                .remove("name")
                .and_then(Value::into_string)
                .ok_or("name property is missing or not a string")?,
        })
    }
}
