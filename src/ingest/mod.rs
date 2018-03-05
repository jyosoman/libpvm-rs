mod parse;
mod persist;
mod pvm;
mod db;

use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use rayon::prelude::*;
use neo4j::cypher::CypherStream;
use serde_json;

use self::pvm::PVM;
use trace::TraceEvent;

fn print_time(tmr: Instant) {
    let dur = tmr.elapsed();
    println!(
        "{:.3} Seconds elapsed",
        dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) * 1e-9
    );
}

pub fn ingest<R>(stream: R, db: CypherStream)
where
    R: BufRead,
{
    let tmr = Instant::now();

    const BATCH_SIZE: usize = 0x80_000;

    let (send, recv) = mpsc::sync_channel(BATCH_SIZE * 2);

    let mut pvm = PVM::new(send);

    let db_worker = thread::spawn(move || {
        persist::execute_loop(db, recv);
    });

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
    drop(pvm);
    println!("Parse Complete");
    print_time(tmr);
    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
    println!("Ingestion Complete");
    print_time(tmr);
}
