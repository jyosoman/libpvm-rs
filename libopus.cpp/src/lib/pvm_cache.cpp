// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 05/09/17.
//

#include "../include/opus/internal/pvm_cache.h"

#include <string>
#include <utility>

namespace opus {
namespace internal {

using std::string;

PVMCache::~PVMCache() {
  for (auto &it : this->node_cache) {
    delete it.second;
  }
}

Node* PVMCache::add(string uuid, string cmdline, bool thin) {
  auto node = new Node(this->id_counter++, std::move(cmdline), thin);
  this->node_cache[std::move(uuid)] = node;
  return node;
}

std::pair<Node*, bool> PVMCache::check(string uuid, string cmdline) {
  auto it = this->node_cache.find(uuid);
  if ( it != this->node_cache.end() ) {
    return std::make_pair(it->second, false);
  } else {
    return std::make_pair(this->add(std::move(uuid),
                                    std::move(cmdline),
                                    true),
                          true);
  }
}

void PVMCache::release(string &uuid) {
  delete this->node_cache[uuid];
  this->node_cache.erase(uuid);
}

}  // namespace internal
}  // namespace opus
