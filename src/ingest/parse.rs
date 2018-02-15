use trace::TraceEvent;

use super::PVMCache;
use super::DB;

fn proc_check(tr: &TraceEvent,
              db: &mut DB,
              pvm_cache: &mut PVMCache) -> Result<(), &'static str> {
    let exec = tr.exec.clone().ok_or("event missing exec")?;
    let par_chk = pvm_cache.check(tr.subjprocuuid, &exec[..]);
    let par_db = pvm_cache.get(tr.subjprocuuid).db_id;
    if par_chk {
        db.create_node(par_db, tr.subjprocuuid, tr.pid, exec)
    } else {
        Ok(())
    }
}

fn proc_exec(tr: &TraceEvent,
             db: &mut DB,
             pvm_cache: &mut PVMCache) -> Result<(), &'static str> {
    let cmdline = tr.cmdline.clone().ok_or("exec missing cmdline")?;
    let par_db = pvm_cache.get(tr.subjprocuuid).db_id;
    if pvm_cache.get(tr.subjprocuuid).thin {
        pvm_cache.set(tr.subjprocuuid, &cmdline[..], false);
        db.update_node(par_db, tr.pid, cmdline)
    } else {
        pvm_cache.add(tr.subjprocuuid, &cmdline[..], false);
        let next_db = pvm_cache.get(tr.subjprocuuid).db_id;
        db.create_node(next_db, tr.subjprocuuid, tr.pid, cmdline)?;
        db.create_rel(par_db, next_db, String::from("next"))
    }
}

fn proc_fork(tr: &TraceEvent,
             db: &mut DB,
             pvm_cache: &mut PVMCache) -> Result<(), &'static str> {
    let ret_objuuid1 = tr.ret_objuuid1.ok_or("fork missing ret_objuuid1")?;
    let par_db = pvm_cache.get(tr.subjprocuuid).db_id;
    let par_cmd = pvm_cache.get(tr.subjprocuuid).cmdline.clone();
    let ch_chk = pvm_cache.check(ret_objuuid1, &par_cmd[..]);
    let ch_db = pvm_cache.get(ret_objuuid1).db_id;
    if ch_chk {
        db.create_node(ch_db,ret_objuuid1,tr.retval, par_cmd)?;
    } else {
        pvm_cache.set(ret_objuuid1, &par_cmd[..], true);
        db.update_node(ch_db, tr.retval, par_cmd)?;
    }
    db.create_rel(par_db, ch_db, String::from("child"))
}

fn proc_exit(tr: &TraceEvent,
             _: &mut DB,
             pvm_cache: &mut PVMCache) -> Result<(), &'static str> {
    pvm_cache.release(tr.subjprocuuid);
    Ok(())
}

pub fn parse_trace(
    tr: &TraceEvent,
    db: &mut DB,
    pvm_cache: &mut PVMCache,
) -> Result<(), &'static str> {
    proc_check(tr, db, pvm_cache)?;
    match &tr.event[..] {
        "audit:event:aue_execve:" => proc_exec(tr, db, pvm_cache),
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => proc_fork(tr, db, pvm_cache),
        "audit:event:aue_exit:" => proc_exit(tr, db, pvm_cache),
        _ => Ok(())
    }
}
