#! /usr/bin/env python2.7

import sys

import ijson.backends.yajl2_cffi as ijson

p_map = {}
p_count = 0
e_count = 0

with open(sys.argv[1]) as jfile:
    parser = ijson.common.items(ijson.parse(jfile, multiple_values=True), '')
    for evt in parser:
        if evt['subjprocuuid'] not in p_map:
            p_map[evt['subjprocuuid']] = True
            p_count += 1
        if evt['event'] == "audit:event:aue_execve:":
            if p_map[evt['subjprocuuid']]:
                p_map[evt['subjprocuuid']] = False
            else:
                p_count += 1
        elif evt['event'] in ["audit:event:aue_fork:", "audit:event:aue_vfork:"]:
            if evt['ret_objuuid1'] not in p_map:
                p_map[evt['ret_objuuid1']] = True
                p_count += 1
        e_count += 1

print("{} Events Processed".format(e_count))
print("{} Process Nodes Observed".format(p_count))
print("{} Unique UUIDs Observed".format(len(p_map)))
