// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 05/09/17.
//

#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_CACHE_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_CACHE_H_

#include <atomic>
#include <unordered_map>
#include <string>

#include "opus/internal/node.h"

namespace opus {
namespace internal {

using std::string;

class PVMCache {
  std::unordered_map<string, Node*> node_cache;
  std::atomic_int id_counter;
 public:
  Node* add(string uuid, string cmdline, bool thin);
  Node* check(string uuid, string cmdline);
};

}  // namespace internal
}  // namespace opus

#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_CACHE_H_
