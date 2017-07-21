extern crate neo4j;
extern crate packstream;
extern crate libc;

use std::mem::transmute;
use std::ffi::CStr;
use std::str::Utf8Error;


#[repr(C)]
#[derive(Debug)]
pub enum CfgMode {
    Auto,
    Advanced,
}

#[repr(C)]
#[derive(Debug)]
pub struct AdvancedConfig {
    consumer_threads: i32,
    persistence_threads: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Config {
    cfg_mode: CfgMode,
    db_user: *mut libc::c_char,
    db_password: *mut libc::c_char,
    cfg_detail: Option<AdvancedConfig>,
}


pub struct LibOpus {
    cfg: Config,
}

#[repr(C)]
pub struct OpusHdl(LibOpus);

fn str_from_c_char<'a>(val: *const libc::c_char) -> Result<&'a str, Utf8Error> {
    unsafe { CStr::from_ptr(val).to_str() }
}

#[no_mangle]
pub extern "C" fn opus_init(cfg: Config) -> *mut OpusHdl {
    unsafe { transmute(Box::new(OpusHdl(LibOpus { cfg: cfg }))) }
}

#[no_mangle]
pub extern "C" fn print_cfg(hdl: *const OpusHdl) {
    let ref hdl = unsafe { &*hdl }.0;
    println!("LibOpus Config {:?}", hdl.cfg);
}

#[no_mangle]
pub extern "C" fn process_event(hdl: *mut OpusHdl, event: *const libc::c_char) {
    let ref mut hdl = unsafe { &mut *hdl }.0;
    let user = str_from_c_char(hdl.cfg.db_user);
    let event = str_from_c_char(event);
    match event {
        Ok(slice) => println!("User {} Processing event: {}", user.unwrap(), slice),
        Err(e) => println!("Error parsing string {}", e),
    }
}

#[no_mangle]
pub extern "C" fn opus_cleanup(hdl: *mut OpusHdl) {
    let _drop_me: Box<OpusHdl> = unsafe { transmute(hdl) };
}


mod ingest {}

#[derive(Debug)]
pub struct Process {
    db_id: u64,
    uuid: String,
    cmdline: String,
    pid: i32,
    thin: bool,
}

pub mod persist;
pub mod query;

#[cfg(test)]
mod tests {
    use super::*;
    use super::query::low;
    use neo4j::cypher::CypherStream;

    #[test]
    fn it_works() {}

    #[test]
    fn test_cypher() {
        let p = Process {
            db_id: 0,
            uuid: String::from("0000-0000-0000"),
            cmdline: String::from("./foo"),
            pid: 2,
            thin: false,
        };
        let mut cypher = CypherStream::connect("localhost:7687", "neo4j", "opus").unwrap();
        persist::persist_node(&mut cypher, &p);
        let foo = low::nodes_by_uuid(&mut cypher, "0000-0000-0000");
        println!("{:?}", foo);
    }
}
