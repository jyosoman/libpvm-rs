// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 06/09/17.
//

#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_H_

#include <vector>

#include "opus/internal/db_tr.h"
#include "opus/internal/pvm_cache.h"
#include "opus/internal/trace.h"

namespace opus {
namespace internal {

void pvm_parse(const opus::trace::TraceEvent &tr,
               PVMCache *cache,
               std::vector<DBTr> *executions);

}  // namespace internal
}  // namespace opus

#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_PVM_H_
