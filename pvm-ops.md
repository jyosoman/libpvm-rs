# PVM Operations for liboups tests

## Message types:
* execve
* fork and vfork
* all others

## ALL
```python
if evt.subjprocuuid not in proc_idx:
    Process.new(uuid    = evt.subjprocuuid,
                pid     = evt.pid,
                cmdline = evt.exec,
                thin    = true)
```

## EXECVE
```python
ALL()
proc = Process.get(evt.subjprocuuid)
if proc.thin:
    proc.cmdline = evt.cmdline
    proc.thin = false
else:
    newv = Process.new(uuid    = evt.subjprocuuid,
                       pid     = evt.pid,
                       cmdline = evt.cmdline,
                       thin    = false)
    proc.next = newv
```

## FORK/VFORK
```python
ALL()
par = Process.get(evt.subjprocuuid)
child = Process.new(uuid    = evt.ret_objuuid1,
                    pid     = evt.retval,
                    cmdline = par.cmdline,
                    thin    = true)
par.children.add(child)
```