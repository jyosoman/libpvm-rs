use std::sync::mpsc::SyncSender;

use ::trace::TraceEvent;
use super::persist::DBTr;
use super::PVMCache;

pub fn parse_trace(
    tr: &TraceEvent,
    send: &mut SyncSender<DBTr>,
    pvm_cache: &mut PVMCache,
) -> Result<(), &'static str> {
    let exec = tr.exec.clone().ok_or("event missing exec")?;
    let mut ret = Ok(());
    let par_chk = pvm_cache.check(tr.subjprocuuid, &exec[..]);
    let par_db = pvm_cache.get(tr.subjprocuuid).db_id;
    if par_chk {
        ret = ret.and(send.send(DBTr::CreateNode {
            id: par_db,
            uuid: tr.subjprocuuid,
            pid: tr.pid,
            cmdline: exec,
        }));
    }

    match &tr.event[..] {
        "audit:event:aue_execve:" => {
            let cmdline = tr.cmdline.clone().ok_or("exec missing cmdline")?;
            if pvm_cache.get(tr.subjprocuuid).thin {
                pvm_cache.set(tr.subjprocuuid, &cmdline[..], false);
                ret = ret.and(send.send(DBTr::UpdateNode {
                    id: par_db,
                    pid: tr.pid,
                    cmdline: cmdline,
                }));
            } else {
                pvm_cache.add(tr.subjprocuuid, &cmdline[..], false);
                let next_db = pvm_cache.get(tr.subjprocuuid).db_id;
                ret = ret.and(send.send(DBTr::CreateNode {
                    id: next_db,
                    uuid: tr.subjprocuuid,
                    pid: tr.pid,
                    cmdline: cmdline,
                }));
                ret = ret.and(send.send(DBTr::CreateRel {
                    src: par_db,
                    dst: next_db,
                    class: String::from("next"),
                }));
            }
        }
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => {
            let ret_objuuid1 = tr.ret_objuuid1.ok_or("fork missing ret_objuuid1")?;
            let par_cmd = pvm_cache.get(tr.subjprocuuid).cmdline.clone();
            let ch_chk = pvm_cache.check(ret_objuuid1, &par_cmd[..]);
            let ch_db = pvm_cache.get(ret_objuuid1).db_id;
            if ch_chk {
                ret = ret.and(send.send(DBTr::CreateNode {
                    id: ch_db,
                    uuid: ret_objuuid1,
                    pid: tr.retval,
                    cmdline: par_cmd,
                }));
            } else {
                pvm_cache.set(ret_objuuid1, &par_cmd[..], true);
                ret = ret.and(send.send(DBTr::UpdateNode {
                    id: ch_db,
                    pid: tr.retval,
                    cmdline: par_cmd,
                }));
            }
            ret = ret.and(send.send(DBTr::CreateRel {
                src: par_db,
                dst: ch_db,
                class: String::from("child"),
            }));
        }
        _ => {}
    }
    ret.map_err(|_| "Database worker closed queue unexpectadly")
}
