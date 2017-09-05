// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 05/09/17.
//

#include "../include/opus/internal/pvm_cache.h"

#include <string>

namespace opus {
namespace internal {

using std::string;

Node* PVMCache::add(string uuid, string cmdline, bool thin) {
  auto node = new Node(this->id_counter++, std::move(cmdline), thin);
  this->node_cache[std::move(uuid)] = node;
  return node;
}

Node* PVMCache::check(string uuid, string cmdline) {
  auto it = this->node_cache.find(uuid);
  if ( it != this->node_cache.end() ) {
    return it->second;
  } else {
    return this->add(std::move(uuid), std::move(cmdline), true);
  }
}

}  // namespace internal
}  // namespace opus
