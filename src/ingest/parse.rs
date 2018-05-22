use super::pvm::{ConnectDir, NodeGuard, PVMError, PVM};
use data::{
    node_types::{File, Name, Pipe, PipeInit, Process, ProcessInit, Ptty, Socket}, Denumerate,
};
use trace::{AuditEvent, TraceEvent};

macro_rules! tr_field {
    ($TR:ident, $F:ident) => {
        $TR.$F.ok_or(PVMError::MissingField {
            evt: $TR.event.clone(),
            field: stringify!($F),
        })?
    };
}

macro_rules! tr_opt_field {
    ($TR:ident, $F:ident) => {
        $TR.$F.clone().ok_or(PVMError::MissingField {
            evt: $TR.event.clone(),
            field: stringify!($F),
        })?
    };
}

fn proc_exec(tr: &AuditEvent, mut pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let cmdline = tr_opt_field!(tr, cmdline);
    let binuuid = tr_field!(tr, arg_objuuid1);
    let binname = tr_opt_field!(tr, upath1);
    let lduuid = tr_field!(tr, arg_objuuid2);
    let ldname = tr_opt_field!(tr, upath2);

    let bin = pvm.declare::<File>(binuuid, None);
    pvm.name(&bin, Name::Path(binname));

    let ld = pvm.declare::<File>(lduuid, None);
    pvm.name(&ld, Name::Path(ldname));

    if Process::denumerate(&pro).thin {
        {
            let pref = Process::denumerate_mut(&mut pro);
            pref.cmdline = cmdline;
            pref.thin = false;
        }
        pvm.prop_node(&pro);
        pvm.source(&pro, &bin);
        pvm.source(&pro, &ld);
    } else {
        let next = pvm.add::<Process>(
            tr.subjprocuuid,
            Some(ProcessInit {
                pid: tr.pid,
                cmdline,
                thin: false,
            }),
        );
        pvm.source(&next, &pro);
        pvm.source(&next, &bin);
        pvm.source(&next, &ld);
    }
    Ok(())
}

fn proc_fork(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let ret_objuuid1 = tr_field!(tr, ret_objuuid1);

    let mut ch = pvm.declare::<Process>(ret_objuuid1, None);
    {
        let pref = Process::denumerate(&pro);
        let chref = Process::denumerate_mut(&mut ch);
        chref.pid = tr.retval;
        chref.cmdline = pref.cmdline.clone();
        chref.thin = true;
    }
    pvm.prop_node(&ch);
    pvm.source(&ch, &pro);
    Ok(())
}

fn proc_exit(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    pvm.release(&tr.subjprocuuid);
    Ok(())
}

fn posix_open(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    if let Some(fuuid) = tr.ret_objuuid1 {
        let fname = tr_opt_field!(tr, upath1);

        let f = pvm.declare::<File>(fuuid, None);
        pvm.name(&f, Name::Path(fname));
    }
    Ok(())
}

fn posix_read(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);

    let f = pvm.declare::<File>(fuuid, None);
    if let Some(pth) = tr.fdpath.clone() {
        if pth != "<unknown>" {
            pvm.name(&f, Name::Path(pth));
        }
    }
    pvm.source_nbytes(&pro, &f, tr.retval);
    Ok(())
}

fn posix_write(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);

    let f = pvm.declare::<File>(fuuid, None);
    if let Some(pth) = tr.fdpath.clone() {
        if pth != "<unknown>" {
            pvm.name(&f, Name::Path(pth));
        }
    }
    pvm.sinkstart_nbytes(&pro, &f, tr.retval);
    Ok(())
}

fn posix_close(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    if let Some(fuuid) = tr.arg_objuuid1 {
        let f = pvm.declare::<File>(fuuid, None);
        pvm.sinkend(&pro, &f);
    }
    Ok(())
}

fn posix_socket(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, ret_objuuid1);
    pvm.declare::<Socket>(suuid, None);
    Ok(())
}

fn posix_listen(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    pvm.declare::<Socket>(suuid, None);
    Ok(())
}

fn posix_bind(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    Ok(())
}

fn posix_accept(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let luuid = tr_field!(tr, arg_objuuid1);
    let ruuid = tr_field!(tr, ret_objuuid1);
    pvm.declare::<Socket>(luuid, None);
    let r = pvm.declare::<Socket>(ruuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&r, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&r, Name::Net(addr, prt));
    }
    Ok(())
}

fn posix_connect(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    Ok(())
}

fn posix_mmap(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let f = pvm.declare::<File>(fuuid, None);
    if let Some(fdpath) = tr.fdpath.clone() {
        pvm.name(&f, Name::Path(fdpath));
    }
    if let Some(flags) = tr.arg_mem_flags.clone() {
        if flags.contains(&String::from("PROT_WRITE")) {
            if let Some(share_flags) = tr.arg_sharing_flags.clone() {
                if !share_flags.contains(&String::from("MAP_PRIVATE")) {
                    pvm.sinkstart(&pro, &f);
                }
            } else {
                pvm.sinkstart(&pro, &f);
            }
        }

        if flags.contains(&String::from("PROT_READ")) {
            pvm.source(&pro, &f);
        }
    }
    Ok(())
}

fn posix_socketpair(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let ruuid1 = tr_field!(tr, ret_objuuid1);
    let ruuid2 = tr_field!(tr, ret_objuuid2);
    let s1 = pvm.declare::<Socket>(ruuid1, None);
    let s2 = pvm.declare::<Socket>(ruuid2, None);
    pvm.connect(&s1, &s2, ConnectDir::BiDirectional);
    Ok(())
}

fn posix_pipe(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let ruuid1 = tr_field!(tr, ret_objuuid1);
    let rfd1 = tr_field!(tr, ret_fd1);
    let ruuid2 = tr_field!(tr, ret_objuuid2);
    let rfd2 = tr_field!(tr, ret_fd2);
    let p1 = pvm.declare::<Pipe>(ruuid1, Some(PipeInit { fd: rfd1 }));
    let p2 = pvm.declare::<Pipe>(ruuid2, Some(PipeInit { fd: rfd2 }));
    pvm.connect(&p1, &p2, ConnectDir::BiDirectional);
    Ok(())
}

fn posix_sendmsg(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    pvm.sinkstart_nbytes(&pro, &s, tr.retval);
    Ok(())
}

fn posix_sendto(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    pvm.sinkstart_nbytes(&pro, &s, tr.retval);
    Ok(())
}

fn posix_recvmsg(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    pvm.source_nbytes(&pro, &s, tr.retval);
    Ok(())
}

fn posix_recvfrom(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let suuid = tr_field!(tr, arg_objuuid1);
    let s = pvm.declare::<Socket>(suuid, None);
    if let Some(pth) = tr.upath1.clone() {
        pvm.name(&s, Name::Path(pth));
    } else if let Some(prt) = tr.port {
        let addr = tr_opt_field!(tr, address);
        pvm.name(&s, Name::Net(addr, prt));
    }
    pvm.source_nbytes(&pro, &s, tr.retval);
    Ok(())
}

fn posix_chdir(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let duuid = tr_field!(tr, arg_objuuid1);
    let d = pvm.declare::<File>(duuid, None);
    if let Some(dpath) = tr.upath1.clone() {
        pvm.name(&d, Name::Path(dpath));
    }
    Ok(())
}

fn posix_chmod(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let fpath = tr_opt_field!(tr, upath1);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.name(&f, Name::Path(fpath));
    pvm.sink(&pro, &f);
    Ok(())
}

fn posix_chown(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let fpath = tr_opt_field!(tr, upath1);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.name(&f, Name::Path(fpath));
    pvm.sink(&pro, &f);
    Ok(())
}

fn posix_fchmod(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.sinkstart(&pro, &f);
    Ok(())
}

fn posix_fchown(tr: &AuditEvent, pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.sinkstart(&pro, &f);
    Ok(())
}

fn posix_posix_openpt(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let ttyuuid = tr_field!(tr, ret_objuuid1);
    pvm.declare::<Ptty>(ttyuuid, None);
    Ok(())
}

fn posix_link(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let upath1 = tr_opt_field!(tr, upath1);
    let upath2 = tr_opt_field!(tr, upath2);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.name(&f, Name::Path(upath1));
    pvm.name(&f, Name::Path(upath2));
    Ok(())
}

fn posix_rename(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let src_uuid = tr_field!(tr, arg_objuuid1);
    let src = tr_opt_field!(tr, upath1);
    let dst = tr_opt_field!(tr, upath2);
    let fsrc = pvm.declare::<File>(src_uuid, None);
    pvm.unname(&fsrc, Name::Path(src));
    if let Some(ovr_uuid) = tr.arg_objuuid2 {
        let fovr = pvm.declare::<File>(ovr_uuid, None);
        pvm.unname(&fovr, Name::Path(dst.clone()));
    }
    pvm.name(&fsrc, Name::Path(dst));
    Ok(())
}

fn posix_unlink(tr: &AuditEvent, _pro: NodeGuard, pvm: &mut PVM) -> Result<(), PVMError> {
    let fuuid = tr_field!(tr, arg_objuuid1);
    let upath1 = tr_opt_field!(tr, upath1);
    let f = pvm.declare::<File>(fuuid, None);
    pvm.unname(&f, Name::Path(upath1));
    Ok(())
}

pub fn parse_trace(tr: &TraceEvent, pvm: &mut PVM) -> Result<(), PVMError> {
    match tr {
        TraceEvent::Audit(box tr) => {
            pvm.set_evt(tr.event.clone());
            pvm.set_time(tr.time);
            let pro = pvm.declare::<Process>(
                tr.subjprocuuid,
                Some(ProcessInit {
                    pid: tr.pid,
                    cmdline: tr.exec.clone(),
                    thin: true,
                }),
            );
            match &tr.event[..] {
                "audit:event:aue_accept:" => posix_accept(tr, pro, pvm),
                "audit:event:aue_bind:" => posix_bind(tr, pro, pvm),
                "audit:event:aue_chdir:" | "audit:event:aue_fchdir:" => posix_chdir(tr, pro, pvm),
                "audit:event:aue_chmod:" | "audit:event:aue_fchmodat:" => posix_chmod(tr, pro, pvm),
                "audit:event:aue_chown:" => posix_chown(tr, pro, pvm),
                "audit:event:aue_close:" => posix_close(tr, pro, pvm),
                "audit:event:aue_connect:" => posix_connect(tr, pro, pvm),
                "audit:event:aue_execve:" => proc_exec(tr, pro, pvm),
                "audit:event:aue_exit:" => proc_exit(tr, pro, pvm),
                "audit:event:aue_fork:" | "audit:event:aue_pdfork:" | "audit:event:aue_vfork:" => {
                    proc_fork(tr, pro, pvm)
                }
                "audit:event:aue_fchmod:" => posix_fchmod(tr, pro, pvm),
                "audit:event:aue_fchown:" => posix_fchown(tr, pro, pvm),
                "audit:event:aue_link:" => posix_link(tr, pro, pvm),
                "audit:event:aue_listen:" => posix_listen(tr, pro, pvm),
                "audit:event:aue_mmap:" => posix_mmap(tr, pro, pvm),
                "audit:event:aue_open_rwtc:" | "audit:event:aue_openat_rwtc:" => {
                    posix_open(tr, pro, pvm)
                }
                "audit:event:aue_pipe:" => posix_pipe(tr, pro, pvm),
                "audit:event:aue_posix_openpt:" => posix_posix_openpt(tr, pro, pvm),
                "audit:event:aue_read:" | "audit:event:aue_pread:" => posix_read(tr, pro, pvm),
                "audit:event:aue_recvmsg:" => posix_recvmsg(tr, pro, pvm),
                "audit:event:aue_recvfrom:" => posix_recvfrom(tr, pro, pvm),
                "audit:event:aue_rename:" => posix_rename(tr, pro, pvm),
                "audit:event:aue_sendmsg:" => posix_sendmsg(tr, pro, pvm),
                "audit:event:aue_sendto:" => posix_sendto(tr, pro, pvm),
                "audit:event:aue_socket:" => posix_socket(tr, pro, pvm),
                "audit:event:aue_socketpair:" => posix_socketpair(tr, pro, pvm),
                "audit:event:aue_unlink:" => posix_unlink(tr, pro, pvm),
                "audit:event:aue_write:"
                | "audit:event:aue_pwrite:"
                | "audit:event:aue_writev:" => posix_write(tr, pro, pvm),
                "audit:event:aue_dup2:" => Ok(()), /* IGNORE */
                _ => {
                    pvm.unparsed_events.insert(tr.event.clone());
                    Ok(())
                }
            }
        }
        TraceEvent::FBT(_) => Ok(()),
    }
}
