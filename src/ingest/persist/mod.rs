mod cypher_view;
mod neo4j_view;

pub use self::{cypher_view::CypherView, neo4j_view::Neo4JView};

use neo4j::Value;

use std::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

use data::NodeID;

#[derive(Clone, Debug)]
pub enum DBTr {
    CreateNode{
        id: NodeID,
        labs: Vec<&'static str>,
        props: HashMap<&'static str, Value>,
    },
    CreateRel{
        src: NodeID,
        dst: NodeID,
        ty: &'static str,
        props: HashMap<&'static str, Value>,
    },
    UpdateNode{
        id: NodeID,
        props: HashMap<&'static str, Value>,
    },
}

pub trait View {
    fn run(&mut self, stream: mpsc::Receiver<Arc<DBTr>>);
}

pub struct ViewCoordinator {
    threads: Vec<JoinHandle<()>>,
    streams: Vec<mpsc::SyncSender<Arc<DBTr>>>,
}

impl ViewCoordinator {
    pub fn new() -> Self {
        ViewCoordinator {
            threads: Vec::new(),
            streams: Vec::new(),
        }
    }

    pub fn register(&mut self, mut view: Box<View + Send>) {
        let (w, r) = mpsc::sync_channel(1000);
        self.threads.push(spawn(move || {
            view.run(r);
        }));
        self.streams.push(w);
    }

    pub fn run(mut self, recv: mpsc::Receiver<DBTr>) {
        for evt in recv {
            {
                let v = Arc::new(evt);
                for stream in self.streams.iter_mut() {
                    stream.send(v.clone()).unwrap();
                }
            }
        }
        self.streams.clear();
        for thr in self.threads.into_iter() {
            thr.join().unwrap();
        }
    }
}

