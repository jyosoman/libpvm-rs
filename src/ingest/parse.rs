use trace::TraceEvent;
use super::pvm::{NodeGuard, PVM};
use data::node_types::{EnumNode, File, Process};
use packstream::values::Value;

fn proc_exec(tr: &TraceEvent, mut pro: NodeGuard, pvm: &mut PVM) {
    let cmdline = tr.cmdline.clone().expect("exec missing cmdline");
    let thin = if let EnumNode::Proc(ref pref) = **pro {
        pref.thin
    } else {
        panic!()
    };
    if thin {
        if let EnumNode::Proc(ref mut pref) = **pro {
            pref.cmdline = cmdline;
            pref.thin = false;
        } else {
            panic!()
        }
        pvm.prop(&**pro);
        pvm.checkin(pro);
    } else {
        let next = pvm.add::<Process>(
            tr.subjprocuuid,
            Some(hashmap!{
            "pid"     => Value::from(tr.pid),
            "cmdline" => Value::from(cmdline.clone()),
            "thin"    => Value::from(false),
            }),
        );
        pvm.source(&**next, &**pro, "next");
        pvm.checkin(next);
        pvm.remove(pro);
    }
}

fn proc_fork(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    let ret_objuuid1 = tr.ret_objuuid1.expect("fork missing ret_objuuid1");

    let mut ch = pvm.declare::<Process>(ret_objuuid1, None);
    if let EnumNode::Proc(ref pref) = **pro {
        if let EnumNode::Proc(ref mut chref) = **ch {
            chref.pid = tr.retval;
            chref.cmdline = pref.cmdline.clone();
            chref.thin = true;
        } else {
            panic!()
        }
    } else {
        panic!()
    }
    pvm.prop(&**ch);
    pvm.source(&**ch, &**pro, "child");
    pvm.checkin(ch);
    pvm.checkin(pro);
}

fn proc_exit(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    pvm.release(&tr.subjprocuuid);
    pvm.remove(pro);
}

fn posix_open(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    if let Some(fuuid) = tr.ret_objuuid1 {
        let fname = tr.upath1.clone().expect("open missing upath1");

        let mut f = pvm.declare::<File>(fuuid, None);
        pvm.name(&mut **f, fname);
        pvm.checkin(f);
    }
    pvm.checkin(pro);
}

fn posix_read(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("read missing arg_objuuid1");

    let f = pvm.declare::<File>(fuuid, None);
    pvm.source(&**pro, &**f, "read");
    pvm.checkin(f);
    pvm.checkin(pro);
}

fn posix_write(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("write missing arg_objuuid1");

    let f = pvm.declare::<File>(fuuid, None);
    pvm.sinkstart(&**pro, &**f, "write");
    pvm.checkin(f);
    pvm.checkin(pro);
}

fn posix_close(tr: &TraceEvent, pro: NodeGuard, pvm: &mut PVM) {
    if let Some(fuuid) = tr.arg_objuuid1 {
        let f = pvm.declare::<File>(fuuid, None);
        pvm.sinkend(&**pro, &**f, "close");
        pvm.checkin(f);
    }
    pvm.checkin(pro);
}

pub fn parse_trace(tr: &TraceEvent, pvm: &mut PVM) {
    let pro = pvm.declare::<Process>(
        tr.subjprocuuid,
        Some(hashmap!{
        "pid"     => Value::from(tr.pid),
        "cmdline" => Value::from(tr.exec.clone().expect("Event missing exec")),
        "thin"    => Value::from(true),
        }),
    );
    match &tr.event[..] {
        "audit:event:aue_execve:" => proc_exec(tr, pro, pvm),
        "audit:event:aue_fork:" | "audit:event:aue_vfork:" => proc_fork(tr, pro, pvm),
        "audit:event:aue_exit:" => proc_exit(tr, pro, pvm),
        "audit:event:aue_open_rwtc:" | "audit:event:aue_openat_rwtc:" => posix_open(tr, pro, pvm),
        "audit:event:aue_read:" => posix_read(tr, pro, pvm),
        "audit:event:aue_write:" => posix_write(tr, pro, pvm),
        "audit:event:aue_close:" => posix_close(tr, pro, pvm),
        _ => {
            pvm.unparsed_events.insert(tr.event.clone());
            pvm.checkin(pro)
        }
    };
}
