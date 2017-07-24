extern crate libc;

use std::ffi::{CStr, CString};
use std::str::Utf8Error;
use std::ptr;
use std::os::unix::io::RawFd;

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
pub struct Config {
    cfg_mode: CfgMode,
    db_user: *mut libc::c_char,
    db_password: *mut libc::c_char,
    cfg_detail: *mut AdvancedConfig,
}

#[derive(Debug)]
pub struct RConfig {
    cfg_mode: CfgMode,
    db_user: String,
    db_password: String,
    cfg_detail: Option<AdvancedConfig>,
}

pub struct LibOpus {
    cfg: RConfig,
}


#[repr(C)]
pub struct OpusHdl(LibOpus);

fn str_from_c_char<'a>(val: *const libc::c_char) -> Result<&'a str, Utf8Error> {
    unsafe { CStr::from_ptr(val).to_str() }
}

#[no_mangle]
pub extern "C" fn opus_init(cfg: Config) -> *mut OpusHdl {
    let hdl = Box::new(OpusHdl(LibOpus {
        cfg: RConfig {
            cfg_mode: cfg.cfg_mode,
            db_user: unsafe {
                CString::from_raw(cfg.db_user)
                    .into_string()
                    .unwrap_or(String::from("neo4j"))
            },
            db_password: unsafe {
                CString::from_raw(cfg.db_password)
                    .into_string()
                    .unwrap_or(String::from("opus"))
            },
            cfg_detail: if cfg.cfg_detail.is_null() {
                Option::None
            } else {
                unsafe { Option::Some(ptr::read(cfg.cfg_detail)) }
            },
        },
    }));
    Box::into_raw(hdl)
}

#[no_mangle]
pub extern "C" fn print_cfg(hdl: *const OpusHdl) {
    let ref hdl = unsafe { &*hdl }.0;
    println!("LibOpus Config {:?}", hdl.cfg);
}

#[no_mangle]
pub extern "C" fn process_events(hdl: *mut OpusHdl, fd: libc::c_int ) {
    let ref mut hdl = unsafe { &mut *hdl }.0;
    let ref user = hdl.cfg.db_user;
    println!("{}", user);
}

#[no_mangle]
pub extern "C" fn opus_cleanup(hdl: *mut OpusHdl) {
    unsafe {
        drop(Box::from_raw(hdl));
    }
}
