//
// Created by tb403 on 04/09/17.
//

#ifndef LIBOPUS_CPP_SRC_LIB_DB_TR_H_
#define LIBOPUS_CPP_SRC_LIB_DB_TR_H_

#include <neo4j-client.h>
#include <string>

namespace opus::internal {

class DBTr {
 public:
  virtual bool execute(neo4j_connection_t *conn) const = 0;
};

class DBCreateNode : DBTr {
  int64_t db_id;
  std::string uuid;
  int32_t pid;
  std::string cmdline;
 public:
  explicit DBCreateNode(int64_t db_id,
                        std::string uuid,
                        int32_t pid,
                        std::string cmdline) : db_id(db_id),
                                               uuid(std::move(uuid)),
                                               pid(pid),
                                               cmdline(std::move(cmdline)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

class DBCreateRel : DBTr {
  int64_t src;
  int64_t dst;
  std::string rclass;
 public:
  explicit DBCreateRel(int64_t src,
                       int64_t dst,
                       std::string rclass) : src(src),
                                             dst(dst),
                                             rclass(std::move(rclass)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

class DBUpdateNode : DBTr {
  int64_t db_id;
  int32_t pid;
  std::string cmdline;
 public:
  explicit DBUpdateNode(int64_t db_id,
                        int32_t pid,
                        std::string cmdline) : db_id(db_id),
                                               pid(pid),
                                               cmdline(std::move(cmdline)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

}

#endif  // LIBOPUS_CPP_SRC_LIB_DB_TR_H_
