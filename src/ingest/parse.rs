use trace::TraceEvent;

use super::{NodeGuard, PVMCache, DB};

fn proc_check(tr: &TraceEvent, db: &mut DB, pvm_cache: &mut PVMCache) -> NodeGuard {
    let exec = tr.exec.clone().expect("event missing exec");
    let (chk, pro) = pvm_cache.check(tr.subjprocuuid, tr.pid, &exec[..]);
    if chk {
        db.create_node(&(*pro));
    }
    pro
}

fn proc_exec(tr: &TraceEvent, mut pro: NodeGuard, db: &mut DB, pvm_cache: &mut PVMCache) {
    let cmdline = tr.cmdline.clone().expect("exec missing cmdline");
    if pro.thin {
        pro.cmdline = cmdline;
        pro.thin = false;
        db.update_node(&(*pro));
    } else {
        let next = pvm_cache.add(tr.subjprocuuid, tr.pid, &cmdline[..], false);
        db.create_node(&(*next));
        db.create_rel(&(*pro), &(*next), String::from("next"));
        pvm_cache.checkin(next);
    }
    pvm_cache.checkin(pro);
}

fn proc_fork(tr: &TraceEvent, pro: NodeGuard, db: &mut DB, pvm_cache: &mut PVMCache) {
    let ret_objuuid1 = tr.ret_objuuid1.expect("fork missing ret_objuuid1");
    let (chk, mut ch) = pvm_cache.check(ret_objuuid1, tr.retval, &pro.cmdline[..]);
    if chk {
        db.create_node(&(*ch));
    } else {
        ch.cmdline = pro.cmdline.clone();
        ch.thin = false;
        db.update_node(&(*ch));
    }
    db.create_rel(&(*pro), &(*ch), String::from("child"));
    pvm_cache.checkin(ch);
    pvm_cache.checkin(pro);
}

fn proc_exit(tr: &TraceEvent, pro: NodeGuard, _: &mut DB, pvm_cache: &mut PVMCache) {
    pvm_cache.release(&tr.subjprocuuid);
    pvm_cache.checkin(pro);
}

pub fn parse_trace(tr: &TraceEvent, db: &mut DB, pvm_cache: &mut PVMCache) {
    let pro = proc_check(tr, db, pvm_cache);
    match &tr.event[..] {
        "audit:event:aue_execve:" => proc_exec(tr, pro, db, pvm_cache),
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => proc_fork(tr, pro, db, pvm_cache),
        "audit:event:aue_exit:" => proc_exit(tr, pro, db, pvm_cache),
        _ => pvm_cache.checkin(pro),
    };
}
