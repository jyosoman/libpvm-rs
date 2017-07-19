extern crate neo4j;
extern crate packstream;

use std::mem::transmute;

#[repr(C)]
#[derive(Debug)]
pub enum CfgMode {
    Auto,
    Advanced
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
    db_user: String,
    db_password: String,
    cfg_detail: AdvancedConfig,
}

pub struct LibOpus {
    cfg: Config,
}

#[repr(C)]
pub struct OpusHdl(LibOpus);

#[no_mangle]
pub extern fn opus_init(cfg: Config) -> *mut OpusHdl {
    unsafe {
        transmute(Box::new( OpusHdl(LibOpus{ cfg: cfg }) ))
    }
}

#[no_mangle]
pub extern fn print_cfg(hdl: *const OpusHdl) {
    let ref hdl = unsafe { & *hdl }.0;
    println!("I'm now processing an event! Config {:?}", hdl.cfg);
}

#[no_mangle]
pub extern fn process_event(hdl: *mut OpusHdl, event: String) {
    let ref hdl = unsafe { & *hdl }.0;
    println!("Processing event {} in db with user {}", event, hdl.cfg.db_user);
}

#[no_mangle]
pub extern fn opus_cleanup(hdl: *mut OpusHdl) {
    let _drop_me: Box<OpusHdl> = unsafe { transmute(hdl) };
}


mod ingest {}

pub struct Process {
    db_id: u64,
    uuid: String,
    cmdline: String,
    pid: i32,
    thin: bool,
}

pub mod persist;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
