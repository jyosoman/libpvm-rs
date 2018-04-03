use super::pvm::{ConnectDir, NodeGuard, PVM};
use data::{Denumerate,
           node_types::{EnumNode, File, Pipe, PipeInit, Process, ProcessInit, Socket,
                        SocketClass, SocketInit}};
use trace::{AuditEvent, TraceEvent};

fn socket_addr(tr: AuditEvent, s: &mut EnumNode) -> bool {
    if let EnumNode::Socket(ref mut s) = *s {
        if let SocketClass::Unknown = s.class {
            if let Some(pth) = tr.upath1 {
                s.class = SocketClass::AfUnix;
                s.path = pth;
                return true;
            } else if let Some(prt) = tr.port {
                let addr = tr.address.expect("record with port missing address");
                s.class = SocketClass::AfInet;
                s.port = prt;
                s.ip = addr;
                return true;
            }
        }
    }
    false
}

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

    let mut f = pvm.declare::<File>(fuuid, None);
    if let Some(pth) = tr.fdpath {
        if pth != "<unknown>" {
            pvm.name(&mut f, pth);
        }
    }
    pvm.source(&pro, &f, "read");
}

fn posix_write(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("write missing arg_objuuid1");

    let mut f = pvm.declare::<File>(fuuid, None);
    if let Some(pth) = tr.fdpath {
        if pth != "<unknown>" {
            pvm.name(&mut f, pth);
        }
    }
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
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
}

fn posix_accept(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let luuid = tr.arg_objuuid1.expect("accept missing arg_objuuid1");
    let ruuid = tr.ret_objuuid1.expect("accept missing ret_objuuid1");
    pvm.declare::<Socket>(luuid, None);
    if let Some(pth) = tr.upath1 {
        pvm.declare::<Socket>(
            ruuid,
            Some(SocketInit {
                class: SocketClass::AfUnix,
                path: pth,
                port: 0,
                ip: String::new(),
            }),
        );
    } else if let Some(prt) = tr.port {
        let addr = tr.address.expect("accept with port missing address");
        pvm.declare::<Socket>(
            ruuid,
            Some(SocketInit {
                class: SocketClass::AfInet,
                path: String::new(),
                port: prt,
                ip: addr,
            }),
        );
    } else {
        pvm.declare::<Socket>(ruuid, None);
    }
}

fn posix_connect(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("connect missing arg_objuuid1");
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
}

fn posix_mmap(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let fuuid = tr.arg_objuuid1.expect("write missing arg_objuuid1");
    let mut f = pvm.declare::<File>(fuuid, None);
    if let Some(fdpath) = tr.fdpath {
        pvm.name(&mut f, fdpath);
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

fn posix_socketpair(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let ruuid1 = tr.ret_objuuid1.expect("socketpair missing ret_objuuid1");
    let ruuid2 = tr.ret_objuuid2.expect("socketpair missing ret_objuuid2");
    let s1 = pvm.declare::<Socket>(ruuid1, None);
    let s2 = pvm.declare::<Socket>(ruuid2, None);
    pvm.connect(&s1, &s2, ConnectDir::BiDirectional, "socketpair");
}

fn posix_pipe(tr: AuditEvent, _pro: NodeGuard, pvm: &mut PVM) {
    let ruuid1 = tr.ret_objuuid1.expect("pipe missing ret_objuuid1");
    let rfd1 = tr.ret_fd1.expect("pipe missing ret_fd1");
    let ruuid2 = tr.ret_objuuid2.expect("pipe missing ret_objuuid2");
    let rfd2 = tr.ret_fd2.expect("pipe missing ret_fd2");
    let p1 = pvm.declare::<Pipe>(ruuid1, Some(PipeInit { fd: rfd1 }));
    let p2 = pvm.declare::<Pipe>(ruuid2, Some(PipeInit { fd: rfd2 }));
    pvm.connect(&p1, &p2, ConnectDir::BiDirectional, "pipe");
}

fn posix_sendmsg(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("sendmsg missing arg_objuuid1");
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
    pvm.sinkstart(&pro, &s, "sendmsg");
}

fn posix_sendto(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("sendto missing arg_objuuid1");
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
    pvm.sinkstart(&pro, &s, "sendto");
}

fn posix_recvmsg(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("recvmsg missing arg_objuuid1");
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
    pvm.source(&pro, &s, "recvmsg");
}

fn posix_recvfrom(tr: AuditEvent, pro: NodeGuard, pvm: &mut PVM) {
    let suuid = tr.arg_objuuid1.expect("recvfrom missing arg_objuuid1");
    let mut s = pvm.declare::<Socket>(suuid, None);
    if socket_addr(tr, &mut s) {
        pvm.prop(&s);
    }
    pvm.source(&pro, &s, "recvfrom");
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
                "audit:event:aue_sendmsg:" => posix_sendmsg(tr, pro, pvm),
                "audit:event:aue_sendto:" => posix_sendto(tr, pro, pvm),
                "audit:event:aue_recvmsg:" => posix_recvmsg(tr, pro, pvm),
                "audit:event:aue_recvfrom:" => posix_recvfrom(tr, pro, pvm),
                "audit:event:aue_close:" => posix_close(tr, pro, pvm),
                "audit:event:aue_socket:" => posix_socket(tr, pro, pvm),
                "audit:event:aue_listen:" => posix_listen(tr, pro, pvm),
                "audit:event:aue_bind:" => posix_bind(tr, pro, pvm),
                "audit:event:aue_accept:" => posix_accept(tr, pro, pvm),
                "audit:event:aue_connect:" => posix_connect(tr, pro, pvm),
                "audit:event:aue_mmap:" => posix_mmap(tr, pro, pvm),
                "audit:event:aue_socketpair:" => posix_socketpair(tr, pro, pvm),
                "audit:event:aue_pipe:" => posix_pipe(tr, pro, pvm),
                _ => {
                    pvm.unparsed_events.insert(tr.event.clone());
                }
            }
        }
        TraceEvent::FBT(_) => {}
    }
}
