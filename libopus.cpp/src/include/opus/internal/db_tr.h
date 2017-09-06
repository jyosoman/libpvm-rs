// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 04/09/17.
//

#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_DB_TR_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_DB_TR_H_

#include <neo4j-client.h>
#include <string>

namespace opus {
namespace internal {

using std::string;

class DBTr {
 public:
  virtual bool execute(neo4j_connection_t *conn) const = 0;
};

class DBCreateNode : public DBTr {
  int64_t db_id;
  string uuid;
  int32_t pid;
  string cmdline;
 public:
  explicit DBCreateNode(int64_t db_id,
                        string uuid,
                        int32_t pid,
                        string cmdline) : db_id(db_id),
                                          uuid(std::move(uuid)),
                                          pid(pid),
                                          cmdline(std::move(cmdline)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

class DBCreateRel : public DBTr {
  int64_t src;
  int64_t dst;
  string rclass;
 public:
  explicit DBCreateRel(int64_t src,
                       int64_t dst,
                       string rclass) : src(src),
                                        dst(dst),
                                        rclass(std::move(rclass)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

class DBUpdateNode : public DBTr {
  int64_t db_id;
  int32_t pid;
  string cmdline;
 public:
  explicit DBUpdateNode(int64_t db_id,
                        int32_t pid,
                        string cmdline) : db_id(db_id),
                                          pid(pid),
                                          cmdline(std::move(cmdline)) {}

  bool execute(neo4j_connection_t *conn) const override;
};

}  // namespace internal
}  // namespace opus

#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_DB_TR_H_
