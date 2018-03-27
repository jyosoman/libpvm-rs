use std::sync::mpsc::Receiver;
use std::sync::Arc;

use super::{View, DBTr};

pub struct CypherFileView {
}

impl CypherFileView {
    pub fn new() -> Box<Self> {
        Box::new(CypherFileView{
        })
    }
}

impl View for CypherFileView {
    fn run(&mut self, stream: Receiver<Arc<DBTr>>) {
        for evt in stream {
            match *evt {
                DBTr::CreateNode { ref id, ref labs, ref props } => {
                }
                DBTr::CreateRel { ref src, ref dst, ref ty, ref props } => {
                }
                DBTr::UpdateNode { ref id, ref props } => {
                }
            }
        }
    }
}