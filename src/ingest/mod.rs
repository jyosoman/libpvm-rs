mod db;
mod parse;
pub mod persist;
pub mod pvm;

use std::io::{BufRead, BufReader, Read};

use rayon::prelude::*;
use serde_json;

use self::pvm::PVM;

use trace::TraceEvent;

const BATCH_SIZE: usize = 0x80_000;

pub fn ingest_stream<R>(stream: R, pvm: &mut PVM)
where
    R: Read,
{
    let mut pre_vec: Vec<String> = Vec::with_capacity(BATCH_SIZE);
    let mut post_vec: Vec<Option<TraceEvent>> = Vec::with_capacity(BATCH_SIZE);
    let mut lines = BufReader::new(stream).lines();

    loop {
        pre_vec.clear();
        while pre_vec.len() < BATCH_SIZE {
            let mut l = match lines.next() {
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
            if l == "[" || l == "]" {
                continue;
            }
            if l.starts_with(", ") {
                l.drain(0..2);
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
                if let Err(e) = parse::parse_trace(&tr, pvm){
                    eprintln!("PVM Parsing error: {}", e);
                    eprintln!("{:?}", tr);
                }
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
