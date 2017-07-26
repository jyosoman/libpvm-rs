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

```cypher
MERGE (p:Process {uuid: {uuid}})
  ON CREATE SET p.pid = {pid}
  ON CREATE SET p.cmdline = {cmdline}
  ON CREATE SET p.thin = true
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

```
MATCH (p:Process {uuid: {uuid}, thin: true})
WHERE NOT (p)-[:INF {class: 'next'}]->()
SET p.cmdline = {cmdline}
SET p.thin = false
UNION
MATCH (p:Process {uuid: {uuid}, thin: false})
CREATE (q:Process {uuid: {uuid}, pid: {pid}, cmdline: {cmdline}, thin: false})
CREATE (p)-[:INF {class: 'next'}]->(q)
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

```
MATCH (p:Process {uuid: {par_uuid}})
WHERE NOT (p)-[:INF {class: 'next'}]->()
CREATE (c:Process {uuid: {ch_uuid}, pid: {pid}, cmdline: p.cmdline, thin: true})
CREATE (p)-[:INF {class: 'child'}]->(c)
```
