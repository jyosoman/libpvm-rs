// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 06/09/17.
//

#include "opus/internal/pvm.h"

#include <string>
#include <queue>

namespace opus {
namespace internal {

using opus::trace::TraceEvent;
using std::make_unique;

void pvm_parse(const TraceEvent &tr,
               PVMCache *cache,
               std::queue<std::unique_ptr<DBTr>> *executions) {
  auto parent_chk = cache->check(tr.subjprocuuid, tr.exec);
  auto parent = parent_chk.first;
  if (parent_chk.second) {
        executions->push(make_unique<DBCreateNode>(parent->get_db_id(),
                                                   tr.subjprocuuid,
                                                   tr.pid,
                                                   tr.exec));
  }
  if (tr.event == "audit:event:aue_execve:") {
      if (parent->get_thin()) {
        parent->set_cmdline(tr.cmdline);
        parent->set_thin(false);
        executions->push(make_unique<DBUpdateNode>(parent->get_db_id(),
                                                   tr.pid,
                                                   tr.cmdline));
      } else {
        auto next = cache->add(tr.subjprocuuid, tr.cmdline, false);
        executions->push(make_unique<DBCreateNode>(next->get_db_id(),
                                                   tr.subjprocuuid,
                                                   tr.pid,
                                                   tr.cmdline));
        executions->push(make_unique<DBCreateRel>(parent->get_db_id(),
                                                  next->get_db_id(),
                                                  std::string("next")));
      }
  } else if (tr.event == "audit:event:aue_fork:" ||
             tr.event == "audit:event:aue_vfork:") {
      auto child_chk = cache->check(tr.ret_objuuid1, parent->get_cmdline());
      auto child = child_chk.first;
      if (child_chk.second) {
        executions->push(make_unique<DBCreateNode>(child->get_db_id(),
                                                   tr.ret_objuuid1,
                                                   tr.retval,
                                                   parent->get_cmdline()));
      } else {
        child->set_cmdline(parent->get_cmdline());
        executions->push(make_unique<DBUpdateNode>(child->get_db_id(),
                                                   tr.retval,
                                                   parent->get_cmdline()));
      }
      executions->push(make_unique<DBCreateRel>(parent->get_db_id(),
                                                child->get_db_id(),
                                                std::string("child")));
  } else if (tr.event == "audit:event:aue_exit:") {
    cache->release(tr.subjprocuuid);
  }
}

}  // namespace internal
}  // namespace opus
