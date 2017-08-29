mod parse;
mod persist;
mod pvm_cache;

use std::cell::Cell;
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

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

    let db_worker = thread::spawn(move || {
       persist::execute_loop(db, recv); 
    });

    const BATCH_SIZE: usize = 65536;
    const WORKERS: usize = 8;

    let mut pre_vec: Arc<Vec<Mutex<Cell<Option<String>>>>> = Arc::new(Vec::with_capacity(BATCH_SIZE));
    let mut post_vec: Arc<Vec<Mutex<Cell<Option<TraceEvent>>>>> = Arc::new(Vec::with_capacity(BATCH_SIZE));
    let mut wrkers = Vec::with_capacity(WORKERS);
    let start_b = Arc::new(Barrier::new(WORKERS+1));
    let done_b = Arc::new(Barrier::new(WORKERS+1));
    let done = Arc::new(AtomicBool::new(false));

    for _ in 0..BATCH_SIZE {
        let pre_mut = Arc::get_mut(&mut pre_vec).unwrap();
        let post_mut = Arc::get_mut(&mut post_vec).unwrap();
        pre_mut.push(Mutex::new(Cell::new(None)));
        post_mut.push(Mutex::new(Cell::new(None)));
    }

    for i in 0..WORKERS {
        let st_b = start_b.clone();
        let do_b = done_b.clone();
        let pre = pre_vec.clone();
        let post = post_vec.clone();
        let done_h = done.clone();
        wrkers.push(thread::spawn(move || {
            loop {
                st_b.wait();
                if !done_h.load(Ordering::SeqCst) {
                    for j in 0..(BATCH_SIZE/WORKERS) {
                        let idx = i*(BATCH_SIZE/WORKERS) + j;
                        match pre[idx].lock().unwrap().take() {
                            Some(s) => { 
                                match serde_json::from_slice(s.as_bytes()) {
                                    Ok(evt) => post[idx].lock().unwrap().set(Some(evt)),
                                    Err(perr) => {
                                        println!("Parsing error: {}", perr);
                                        println!("{}", s);
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                } else {
                    break
                }
                do_b.wait();
            }
        }));
    }

    let mut lines = stream.lines();
    let mut last = false;

    loop {
        for i in 0..BATCH_SIZE {
            let l = match lines.next() {
                Some(l) => match l {
                    Ok(l) => l,
                    Err(perr) => {
                        println!("Parsing error: {}", perr);
                        continue;
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
            pre_vec[i].lock().unwrap().set(Some(l));
        }
        start_b.wait(); // Start threads
        done_b.wait(); // Threads done
        for i in 0..BATCH_SIZE {
            let tr = match post_vec[i].lock().unwrap().take() {
                Some(tr) => tr,
                None => continue,
            };
            if let Err(perr) = parse::parse_trace(&tr, &mut send, &mut cache) {
                println!("PVM parsing error {}", perr);
            }
        }
        if last {
            done.store(true, Ordering::SeqCst);
            start_b.wait();
            break;
        }
    }
    drop(send);
    for w in wrkers {
        w.join().unwrap();
    }
    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
}
