extern crate libc;

use std::ffi::CStr;
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
    cfg_detail: *const AdvancedConfig,
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


#[no_mangle]
pub unsafe extern "C" fn opus_init(cfg: Config) -> *mut OpusHdl {
    let user_b = CStr::from_ptr(cfg.db_user).to_bytes();
    let password_b = CStr::from_ptr(cfg.db_password).to_bytes();
    let mut user_ownd: Vec<u8> = vec![0; user_b.len()];
    let mut passwd_ownd: Vec<u8> = vec![0; password_b.len()];
    user_ownd.copy_from_slice(user_b);
    passwd_ownd.copy_from_slice(password_b);

    let hdl = Box::new(OpusHdl(LibOpus {
        cfg: RConfig {
            cfg_mode: cfg.cfg_mode,
            db_user: String::from_utf8(user_ownd).unwrap_or_else(|_| String::from("neo4j")),
            db_password: String::from_utf8(passwd_ownd).unwrap_or_else(|_| String::from("opus")),
            cfg_detail: if cfg.cfg_detail.is_null() {
                Option::None
            } else {
                Option::Some(ptr::read(cfg.cfg_detail))
            },
        },
    }));
    Box::into_raw(hdl)
}

#[no_mangle]
pub unsafe extern "C" fn print_cfg(hdl: *const OpusHdl) {
    let hdl = &(*hdl).0;
    println!("LibOpus {:?}", hdl.cfg);
}

#[no_mangle]
pub unsafe extern "C" fn process_events(hdl: *mut OpusHdl, fd: libc::c_int) {
    let hdl = &mut (*hdl).0;
    let user = &hdl.cfg.db_user;
    println!("DB user: {}", user);
}

#[no_mangle]
pub unsafe extern "C" fn opus_cleanup(hdl: *mut OpusHdl) {
    drop(Box::from_raw(hdl));
    println!("Cleaning up..");
}
