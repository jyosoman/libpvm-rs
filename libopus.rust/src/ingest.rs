use std::collections::HashMap;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;

use neo4j::cypher::CypherStream;
use serde_json;

use persist;
use invbloom::InvBloom;


pub fn ingest<R>(stream: R, mut db: CypherStream)
where
    R: BufRead,
{
    db.run_unchecked("CREATE INDEX ON :Process(uuid)", HashMap::new());

    let cache = InvBloom::new();

    let (mut send, recv) = mpsc::sync_channel(1024);

    let db_worker = thread::spawn(move || {
        let mut trs = 0;
        db.begin_transaction(None);
        for tr in recv.iter() {
            if let Err(e) = persist::execute(&mut db, &tr) {
                println!("{}", e);
            }
            trs += 1;
        }
        db.commit_transaction();
        println!("Neo4J Queries Issued: {}", trs);
    });

    for line in stream.lines() {
        let res = match line {
            Ok(l) => serde_json::from_slice(l.as_bytes()),
            Err(perr) => {
                println!("Parsing error: {}", perr);
                break;
            }
        };
        match res {
            Ok(evt) => {
                if let Err(perr) = persist::parse_trace(&evt, &mut send, &cache) {
                    println!("PVM parsing error {}", perr);
                }
            }
            Err(perr) => {
                println!("Parsing error: {}", perr);
                break;
            }
        }
    }
    drop(send);
    if let Err(e) = db_worker.join() {
        println!("Database thread panicked: {:?}", e);
    }
}
