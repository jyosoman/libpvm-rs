# Database Setup
```
CREATE CONSTRAINT ON (p:Process) ASSERT p.db_id IS UNIQUE;
CREATE CONSTRAINT ON (p:Process) ASSERT exists(p.db_id);
CREATE INDEX ON :Process(uuid)
```

# Writing Query
```
MERGE (p:Process {db_id: {db_id}})
SET p.uuid = {uuid}
SET p.cmdline = {cmdline}
SET p.pid = {pid}
SET p.thin = {thin}
WITH p
FOREACH (ch IN {chs} |
    MERGE (p)-[e]->(q:Process {db_id: {ch.id}})
    SET e.class = ch.class
```

# Reading Queries
## Node by UUID
```
MATCH (n)
WHERE n.uuid = {uuid}
RETURN n
```

## Node Successors
```
MATCH (n {db_id: {db_id}})-[]->(m)
RETURN m
```

## Node Predecessors
```
MATCH (n {db_id: {db_id}})<-[]-(m)
RETURN m
```
