mod db;
mod parse;
pub mod persist;
mod pvm;

use std::{thread, io::{BufRead, Write}, sync::mpsc};

use neo4j::Neo4jDB;
use rayon::prelude::*;
use serde_json;

use self::{persist::ViewCoordinator, pvm::PVM};

use neo4j_glue::{CypherView, Neo4JView};

use trace::TraceEvent;

const BATCH_SIZE: usize = 0x80_000;

fn parse<R>(stream: R, mut pvm: PVM)
where
    R: BufRead,
{
    let mut pre_vec: Vec<String> = Vec::with_capacity(BATCH_SIZE);
    let mut post_vec: Vec<Option<TraceEvent>> = Vec::with_capacity(BATCH_SIZE);
    let mut lines = stream.lines();

    loop {
        pre_vec.clear();
        while pre_vec.len() < BATCH_SIZE {
            let l = match lines.next() {
                Some(l) => match l {
                    Ok(l) => l,
                    Err(perr) => {
                        println!("Parsing error: {}", perr);
                        continue;
                    }
                },
                None => {
                    break;
                }
            };
            if l.is_empty() {
                continue;
            }
            pre_vec.push(l);
        }

        pre_vec
            .par_iter()
            .map(|s| match serde_json::from_slice(s.as_bytes()) {
                Ok(evt) => Some(evt),
                Err(perr) => {
                    println!("Parsing error: {}", perr);
                    println!("{}", s);
                    None
                }
            })
            .collect_into(&mut post_vec);

        for tr in post_vec.drain(..) {
            if let Some(tr) = tr {
                parse::parse_trace(tr, &mut pvm);
            }
        }
        if pre_vec.len() < BATCH_SIZE {
            break;
        }
    }
    println!("Missing Events:");
    for evt in pvm.unparsed_events.drain() {
        println!("{}", evt);
    }
}

pub fn ingest<R, S>(stream: R, db: Option<Neo4jDB>, cy: Option<S>)
where
    R: BufRead,
    S: Write + Send + 'static,
{
    let (send, recv) = mpsc::sync_channel(BATCH_SIZE * 2);

    let pvm = PVM::new(send);

    let db_worker = thread::spawn(move || {
        let mut view_ctrl = ViewCoordinator::new();
        if let Some(db) = db {
            view_ctrl.register(Neo4JView::new(db));
        }
        if let Some(cy) = cy {
            view_ctrl.register(CypherView::new(cy));
        }
        view_ctrl.run(recv);
    });

    timeit!(parse(stream, pvm));

    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
}
