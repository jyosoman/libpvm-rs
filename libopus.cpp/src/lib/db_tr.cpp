// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 04/09/17.
//

#include "opus/internal/db_tr.h"

namespace opus {
namespace internal {

bool DBCreateNode::execute(neo4j_connection_t *conn) const {
  auto static const N_PROPS = 4;
  neo4j_map_entry_t props[N_PROPS];
  props[0] = neo4j_map_entry("db_id", neo4j_int(this->db_id));
  props[1] = neo4j_map_entry("uuid", neo4j_string(this->uuid.c_str()));
  props[2] = neo4j_map_entry("pid", neo4j_int(this->pid));
  props[3] = neo4j_map_entry("cmdline", neo4j_string(this->cmdline.c_str()));
  auto res = neo4j_send(conn,
                        "CREATE (n:Process {db_id: $db_id, "
                        "                   uuid: $uuid, "
                        "                   pid: $pid, "
                        "                   cmdline: $cmdline})",
                        neo4j_map(props, N_PROPS));
  if (neo4j_check_failure(res) != 0) {
    printf("CreateNode Error: %s\n", neo4j_error_message(res));
  }
  return (neo4j_close_results(res) == 0);
}

bool DBCreateRel::execute(neo4j_connection_t *conn) const {
  auto static const N_PROPS = 3;
  neo4j_map_entry_t props[N_PROPS];
  props[0] = neo4j_map_entry("src", neo4j_int(this->src));
  props[1] = neo4j_map_entry("dst", neo4j_int(this->dst));
  props[2] = neo4j_map_entry("class", neo4j_string(this->rclass.c_str()));
  auto res = neo4j_send(conn,
                        "MATCH (s:Process {db_id: $src}),"
                        "      (d:Process {db_id: $dst}) "
                        "CREATE (s)-[:INF {class: $class}]->(d)",
                        neo4j_map(props, N_PROPS));
  if (neo4j_check_failure(res) != 0) {
    printf("CreateRel Error: %s\n", neo4j_error_message(res));
  }
  return (neo4j_close_results(res) == 0);
}

bool DBUpdateNode::execute(neo4j_connection_t *conn) const {
  auto static const N_PROPS = 3;
  neo4j_map_entry_t props[N_PROPS];
  props[0] = neo4j_map_entry("db_id", neo4j_int(this->db_id));
  props[1] = neo4j_map_entry("pid", neo4j_int(this->pid));
  props[2] = neo4j_map_entry("cmdline", neo4j_string(this->cmdline.c_str()));
  auto res = neo4j_send(conn,
                        "MATCH (p:Process {db_id: $db_id}) "
                        "SET p.pid = $pid "
                        "SET p.cmdline = $cmdline",
                        neo4j_map(props, N_PROPS));
  if (neo4j_check_failure(res) != 0) {
    printf("UpdateNode Error: %s\n", neo4j_error_message(res));
  }
  return (neo4j_close_results(res) == 0);
}

}  // namespace internal
}  // namespace opus
