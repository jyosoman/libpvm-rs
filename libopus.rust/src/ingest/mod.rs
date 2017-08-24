mod parse;
mod persist;
mod pvm_cache;

use std::collections::HashMap;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;

use neo4j::cypher::CypherStream;
use serde_json;

use self::pvm_cache::PVMCache;

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

    for line in stream.lines() {
        let l = match line {
            Ok(l) => l,
            Err(perr) => {
                println!("Parsing error: {}", perr);
                break;
            }
        };
        if l.is_empty() {
            continue;
        }
        match serde_json::from_slice(l.as_bytes()) {
            Ok(evt) => {
                if let Err(perr) = parse::parse_trace(&evt, &mut send, &mut cache) {
                    println!("PVM parsing error {}", perr);
                }
            }
            Err(perr) => {
                println!("Parsing error: {}", perr);
                println!("{}", l);
                break;
            }
        }
    }
    drop(send);
    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
}
