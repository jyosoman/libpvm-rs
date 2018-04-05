use iostream::IOStream;
use libc::{c_char, malloc};

use std::{collections::HashMap,
          ffi::CStr,
          hash::Hash,
          mem::size_of,
          ops::Deref,
          os::unix::io::{FromRawFd, RawFd},
          ptr,
          slice};

use engine;

#[repr(C)]
#[derive(Debug)]
pub struct KeyVal {
    key: *mut c_char,
    val: *mut c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct View {
    id: ViewHdl,
    name: *mut c_char,
    desc: *mut c_char,
    num_parameters: usize,
    parameters: *mut KeyVal,
}

#[repr(C)]
#[derive(Debug)]
pub struct ViewInst {
    id: ViewInstHdl,
    vtype: ViewHdl,
    num_parameters: usize,
    parameters: *mut KeyVal,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum CfgMode {
    Auto,
    Advanced,
}

#[repr(C)]
#[derive(Debug)]
pub struct AdvancedConfig {
    consumer_threads: usize,
    persistence_threads: usize,
}

#[repr(C)]
pub struct Config {
    cfg_mode: CfgMode,
    db_server: *mut c_char,
    db_user: *mut c_char,
    db_password: *mut c_char,
    suppress_default_views: bool,
    cfg_detail: *const AdvancedConfig,
}

pub struct OpusHdl(engine::Engine);

#[repr(C)]
#[derive(Debug)]
pub struct ViewHdl(usize);

#[repr(C)]
#[derive(Debug)]
pub struct ViewInstHdl(usize);

fn keyval_arr_to_hashmap(ptr: *mut KeyVal, n: usize) -> HashMap<String, String> {
    let mut ret = HashMap::with_capacity(n);
    if !ptr.is_null() {
        let s = unsafe { slice::from_raw_parts(ptr, n) };
        for kv in s {
            ret.insert(
                string_from_c_char(kv.key).unwrap(),
                string_from_c_char(kv.val).unwrap(),
            );
        }
    }
    ret
}

fn hashmap_to_keyval_arr<T: Deref<Target = str> + Eq + Hash>(
    h: &HashMap<T, T>,
) -> (*mut KeyVal, usize) {
    let len = h.len();
    let data = unsafe { malloc(len * size_of::<KeyVal>()) as *mut KeyVal };
    let s = unsafe { slice::from_raw_parts_mut(data, len) };
    for ((k, v), kv) in h.into_iter().zip(s) {
        kv.key = string_to_c_char(k);
        kv.val = string_to_c_char(v);
    }
    (data, len)
}

fn string_to_c_char(val: &str) -> *mut c_char {
    if val.contains('\0') {
        panic!("Trying to convert a string containing nulls to a C-string");
    }
    unsafe {
        let data = malloc((val.len() + 1) * size_of::<c_char>()) as *mut c_char;
        ptr::copy(val.as_ptr() as *const c_char, data, val.len());
        *data.offset(val.len() as isize) = 0x00 as c_char;
        data
    }
}

fn string_from_c_char(str_p: *mut c_char) -> Option<String> {
    unsafe { CStr::from_ptr(str_p) }
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn opus_init(cfg: Config) -> *mut OpusHdl {
    let r_cfg = engine::Config {
        cfg_mode: cfg.cfg_mode,
        db_server: string_from_c_char(cfg.db_server).unwrap_or("localhost:7687".to_string()),
        db_user: string_from_c_char(cfg.db_user).unwrap_or("neo4j".to_string()),
        db_password: string_from_c_char(cfg.db_password).unwrap_or("opus".to_string()),
        suppress_default_views: cfg.suppress_default_views,
        cfg_detail: if cfg.cfg_detail.is_null() {
            Option::None
        } else {
            Option::Some(ptr::read(cfg.cfg_detail))
        },
    };
    let e = engine::Engine::new(r_cfg);
    let hdl = Box::new(OpusHdl(e));
    Box::into_raw(hdl)
}

#[no_mangle]
pub unsafe extern "C" fn opus_start_pipeline(hdl: *mut OpusHdl) {
    let engine = &mut (*hdl).0;
    engine.init_pipeline().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn opus_shutdown_pipeline(hdl: *mut OpusHdl) {
    let engine = &mut (*hdl).0;
    engine.shutdown_pipeline().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn opus_print_cfg(hdl: *const OpusHdl) {
    let engine = &(*hdl).0;
    engine.print_cfg();
}

#[no_mangle]
pub unsafe extern "C" fn opus_list_view_types(hdl: *const OpusHdl, out: *mut *mut View) -> usize {
    let engine = &(*hdl).0;
    let views = engine.list_view_types().unwrap();
    let len = views.len();
    *out = malloc(len * size_of::<View>()) as *mut View;
    let s = slice::from_raw_parts_mut(*out, len);
    for (view, c_view) in views.into_iter().zip(s) {
        c_view.id = ViewHdl(view.id());
        c_view.name = string_to_c_char(view.name());
        c_view.desc = string_to_c_char(view.desc());
        let (params, num) = hashmap_to_keyval_arr(&view.params());
        c_view.num_parameters = num;
        c_view.parameters = params;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn opus_create_view(
    hdl: *mut OpusHdl,
    view_id: ViewHdl,
    params: *mut KeyVal,
    n_params: usize,
) -> ViewInstHdl {
    let engine = &mut (*hdl).0;
    let rparams = keyval_arr_to_hashmap(params, n_params);
    let ViewHdl(view_id) = view_id;
    ViewInstHdl(engine.create_view_by_id(view_id, rparams).unwrap())
}

#[no_mangle]
pub unsafe extern "C" fn opus_list_view_inst(
    hdl: *const OpusHdl,
    out: *mut *mut ViewInst,
) -> usize {
    let engine = &(*hdl).0;
    let views = engine.list_running_views().unwrap();
    let len = views.len();
    *out = malloc(len * size_of::<ViewInst>()) as *mut ViewInst;
    let s = slice::from_raw_parts_mut(*out, len);
    for (view, c_view) in views.into_iter().zip(s) {
        c_view.id = ViewInstHdl(view.id());
        c_view.vtype = ViewHdl(view.vtype());
        let (params, num) = hashmap_to_keyval_arr(view.params());
        c_view.num_parameters = num;
        c_view.parameters = params;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn opus_ingest_fd(hdl: *mut OpusHdl, fd: i32) {
    let engine = &mut (*hdl).0;
    let stream = IOStream::from_raw_fd(fd as RawFd);
    timeit!(engine.ingest_stream(stream).unwrap());
}

#[no_mangle]
pub unsafe extern "C" fn opus_cleanup(hdl: *mut OpusHdl) {
    drop(Box::from_raw(hdl));
    println!("Cleaning up..");
}

#[no_mangle]
pub unsafe extern "C" fn opus_count_processes(hdl: *const OpusHdl) -> i64 {
    let engine = &(*hdl).0;
    engine.count_processes()
}
