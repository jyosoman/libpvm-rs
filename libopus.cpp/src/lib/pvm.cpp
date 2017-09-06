// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 06/09/17.
//

#include "opus/internal/pvm.h"

#include <vector>

namespace opus {
namespace internal {

void pvm_parse(const TraceEvent *tr,
               PVMCache *cache,
               std::vector<DBTr> *executions){
  auto par_chk = cache->check(tr->subjprocuuid, tr->exec);
  auto par = par_chk.first;
  if (par_chk.second) {
        executions->push_back(DBCreateNode(par->get_db_id(),
                                           tr->subjprocuuid,
                                           tr->pid,
                                           tr->exec));
  }
  if (tr->event == "audit:event:aue_execve:" ) {
      if (par->get_thin()) {
        par->set_cmdline(tr->cmdline);
        executions->push_back(DBUpdateNode(par->get_db_id(),
                                           tr->pid,
                                           tr->cmdline));
      } else {
        auto next = cache->add(tr->subjprocuuid, tr->cmdline, false);
        executions->push_back(DBCreateNode(next->get_db_id(),
                                           tr->subjprocuuid,
                                           tr->pid,
                                           tr->cmdline));
        executions->push_back(DBCreateRel(par->get_db_id(),
                                          next->get_db_id(),
                                          std::string("next")));
      }
  } else if (tr->event == "audit:event:aue_fork:" ||
             tr->event == "audit:event:aue_vfork:") {
      auto ch_chk = cache->check(tr->ret_objuuid1, par->get_cmdline());
      auto ch = ch_chk.first;
      if (ch_chk.second) {
        executions->push_back(DBCreateNode(ch->get_db_id(),
                                           tr->ret_objuuid1,
                                           tr->retval,
                                           par->get_cmdline()));
      } else {
        ch->set_cmdline(par->get_cmdline());
        executions->push_back(DBUpdateNode(ch->get_db_id(),
                                           tr->retval,
                                           par->get_cmdline()));

      }
      executions->push_back(DBCreateRel(par->get_db_id(),
                                        ch->get_db_id(),
                                        std::string("child")));
  }
}

}  // namespace internal
}  // namespace opus