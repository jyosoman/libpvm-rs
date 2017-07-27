extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
pub struct TraceEvent {
    event: String,
    time: u64,
    pid: i32,
    ppid: i32,
    tid: i32,
    uid: i32,
    exec: Option<String>,
    cmdline: Option<String>,
    upath1: Option<String>,
    upath2: Option<String>,
    fd: Option<i32>,
    flags: Option<i32>,
    fdpath: Option<i32>,
    subjprocuuid: [i64; 2],
    subjthruuid: [i64; 2],
    arg_objuuid1: Option<[i64; 2]>,
    arg_objuuid2: Option<[i64; 2]>,
    ret_objuuid1: Option<[i64; 2]>,
    ret_objuuid2: Option<[i64; 2]>,
    retval: i32
}
