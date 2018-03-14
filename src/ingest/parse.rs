use trace::{AuditEvent, TraceEvent};
use super::pvm::{NodeGuard, PVM};
use data::Enumerable;
use data::node_types::{File, Process, ProcessInit, Socket};

fn proc_exec(mut tr: AuditEvent, mut pro: NodeGuard, pvm: &mut PVM) {
    let cmdline = tr.cmdline.take().expect("exec missing cmdline");
    let binuuid = tr.arg_objuuid1.expect("exec missing arg_objuuid1");
    let binname = tr.upath1.take().expect("exec missing upath1");
    let lduuid = tr.arg_objuuid2.expect("exec missing arg_objuuid2");
    let ldname = tr.upath2.take().expect("exec missing upath2");

    let mut bin = pvm.declare::<File>(binuuid, None);
    pvm.name(&mut bin, binname);

    let mut ld = pvm.declare::<File>(lduuid, None);
    pvm.name(&mut ld, ldname);

    if Process::denumerate(&pro).thin {
        {
            let pref = Process::denumerate_mut(&mut pro);
            pref.cmdline = cmdline;
            pref.thin = false;
        }
        pvm.prop(&pro);
        pvm.source(&pro, &bin, "binary");
        pvm.source(&pro, &ld, "linker");
    } else {
        let next = pvm.add::<Process>(
            tr.subjprocuuid,
            Some(ProcessInit {
                pid: tr.pid,
                cmdline,
                thin: false,
            }),
        );
        pvm.source(&next, &pro, "next");
        pvm.source(&next, &bin, "binary");
        pvm.source(&next, &ld, "linker");
    }
}

fn proc_fork(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let ret_objuuid1 = tr.ret_objuuid1.expect("fork missing ret_objuuid1");

    let mut ch = pvm.declare::<Process>(ret_objuuid1, None);
    {
        let pref = Process::denumerate(&pro);
        let chref = Process::denumerate_mut(&mut ch);
        chref.pid = tr.retval;
        chref.cmdline = pref.cmdline.clone();
        chref.thin = true;
    }
    pvm.prop(&ch);
    pvm.source(&ch, &pro, "child");
}

fn proc_exit(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    pvm.release(&tr.subjprocuuid);
}

fn posix_open(mut tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    if let Some(fuuid) = tr.ret_objuuid1 {
        let fname = tr.upath1.take().expect("open missing upath1");

        let mut f = pvm.declare::<File>(fuuid, None);
        pvm.name(&mut f, fname);
    }
}

fn posix_read(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("read missing arg_objuuid1");

    let f = pvm.declare::<File>(fuuid, None);
    pvm.source(&pro, &f, "read");
}

fn posix_write(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("write missing arg_objuuid1");

    let f = pvm.declare::<File>(fuuid, None);
    pvm.sinkstart(&pro, &f, "write");
}

fn posix_close(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    if let Some(fuuid) = tr.arg_objuuid1 {
        let f = pvm.declare::<File>(fuuid, None);
        pvm.sinkend(&pro, &f, "close");
    }
}

fn posix_socket(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.ret_objuuid1.expect("socket missing ret_objuuid1");
    pvm.declare::<Socket>(suuid, None);
}

fn posix_listen(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("listen missing arg_objuuid1");
    pvm.declare::<Socket>(suuid, None);
}

fn posix_bind(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("bind missing arg_objuuid1");
    pvm.declare::<Socket>(suuid, None);
}

fn posix_accept(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let luuid = tr.arg_objuuid1.expect("bind missing arg_objuuid1");
    let ruuid = tr.ret_objuuid1.expect("bind missing ret_objuuid1");
    pvm.declare::<Socket>(luuid, None);
    pvm.declare::<Socket>(ruuid, None);
}

fn posix_connect(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("connect missing arg_objuuid1");
    pvm.declare::<Socket>(suuid, None);
}

fn posix_mmap(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("write missing arg_objuuid1");
    let mut f = pvm.declare::<File>(fuuid, None);
    if let Some(fdpath) = tr.fdpath {
        pvm.name(&mut f, fdpath.clone());
    }
    if let Some(flags) = tr.arg_mem_flags {
        if flags.contains(&String::from("PROT_WRITE"))
            && !flags.contains(&String::from("MAP_PRIVATE"))
        {
            pvm.sinkstart(&pro, &f, "mmap");
        }
        if flags.contains(&String::from("PROT_READ")) {
            pvm.source(&pro, &f, "mmap");
        }
    }
}

pub fn parse_trace(tr: TraceEvent, pvm: &mut PVM) {
    match tr {
        TraceEvent::Audit(box mut tr) => {
            let pro = pvm.declare::<Process>(
                tr.subjprocuuid,
                Some(ProcessInit {
                    pid: tr.pid,
                    cmdline: tr.exec.take().expect("Event missing exec"),
                    thin: true,
                }),
            );
            match &tr.event[..] {
                "audit:event:aue_execve:" => proc_exec(tr, pro, pvm),
                "audit:event:aue_fork:" | "audit:event:aue_vfork:" => proc_fork(tr, pro, pvm),
                "audit:event:aue_exit:" => proc_exit(tr, pro, pvm),
                "audit:event:aue_open_rwtc:" | "audit:event:aue_openat_rwtc:" => {
                    posix_open(tr, pro, pvm)
                }
                "audit:event:aue_read:" | "audit:event:aue_pread:" => posix_read(tr, pro, pvm),
                "audit:event:aue_write:"
                | "audit:event:aue_pwrite:"
                | "audit:event:aue_writev:" => posix_write(tr, pro, pvm),
                "audit:event:aue_close:" => posix_close(tr, pro, pvm),
                "audit:event:aue_socket:" => posix_socket(tr, pro, pvm),
                "audit:event:aue_listen:" => posix_listen(tr, pro, pvm),
                "audit:event:aue_bind:" => posix_bind(tr, pro, pvm),
                "audit:event:aue_accept:" => posix_accept(tr, pro, pvm),
                "audit:event:aue_connect:" => posix_connect(tr, pro, pvm),
                "audit:event:aue_mmap:" => posix_mmap(tr, pro, pvm),
                _ => {
                    pvm.unparsed_events.insert(tr.event.clone());
                }
            }
        }
        TraceEvent::FBT(_) => {}
    }
}
