mod parse;
mod persist;
mod pvm_cache;

use std::collections::HashMap;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;

use rayon::prelude::*;
use neo4j::cypher::CypherStream;
use serde_json;

use self::pvm_cache::PVMCache;
use trace::TraceEvent;

pub fn ingest<R>(stream: R, mut db: CypherStream)
where
    R: BufRead,
{
    db.run_unchecked("CREATE INDEX ON :Process(db_id)", HashMap::new());

    let mut cache = PVMCache::new();

    let (mut send, recv) = mpsc::sync_channel(1024);

    let db_worker = thread::spawn(move || { persist::execute_loop(db, recv); });

    const BATCH_SIZE: usize = 65536;

    let mut pre_vec: Vec<Option<String>> = Vec::with_capacity(BATCH_SIZE);
    let mut post_vec: Vec<Option<TraceEvent>> = Vec::with_capacity(BATCH_SIZE);
    let mut lines = stream.lines();
    let mut last = false;

    loop {
        pre_vec.clear();
        while pre_vec.len() < BATCH_SIZE {
            let l = match lines.next() {
                Some(l) => {
                    match l {
                        Ok(l) => l,
                        Err(perr) => {
                            println!("Parsing error: {}", perr);
                            continue;
                        }
                    }
                }
                None => {
                    last = true;
                    break;
                }
            };
            if l.is_empty() {
                continue;
            }
            pre_vec.push(Some(l));
        }

        pre_vec
            .par_iter()
            .map(|s| match *s {
                Some(ref s) => {
                    match serde_json::from_slice(s.as_bytes()) {
                        Ok(evt) => Some(evt),
                        Err(perr) => {
                            println!("Parsing error: {}", perr);
                            println!("{}", s);
                            None
                        }
                    }
                }
                None => None,
            })
            .collect_into(&mut post_vec);

        for tr in &post_vec {
            match *tr {
                Some(ref tr) => {
                    if let Err(perr) = parse::parse_trace(tr, &mut send, &mut cache) {
                        println!("PVM parsing error {}", perr);
                    }
                }
                None => continue,
            }
        }
        if last {
            break;
        }
    }
    drop(send);
    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
}
