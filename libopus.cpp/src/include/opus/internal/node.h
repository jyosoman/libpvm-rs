// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 05/09/17.
//

#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_NODE_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_NODE_H_

#include <cstdint>
#include <string>

namespace opus {
namespace internal {

using std::string;

class Node {
  int64_t db_id;
  string cmdline;
  bool thin;
 public:
  explicit Node(int64_t db_id,
                string cmdline,
                bool thin) : db_id(db_id),
                             cmdline(std::move(cmdline)),
                             thin(thin) {}

  const string* get_cmdline() const {return &(this->cmdline); }
  const int64_t get_db_id() const { return this->db_id; }
  const bool get_thin() const { return this->thin; }

  void set_cmdline(string cmdline) { this->cmdline = std::move(cmdline); }
  void set_thin(bool thin) { this->thin = thin; }
};

}  // namespace internal
}  // namespace opus

#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_NODE_H_
