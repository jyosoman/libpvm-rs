// Copyright [2017] <Thomas Bytheway & Lucian Carata>
#include "opus/opus.h"

#include <neo4j-client.h>
#include <queue>
#include <iostream>
#include <string>

#include "opus/internal/opus_session.h"
#include "opus/internal/db_tr.h"
#include "opus/internal/pvm_cache.h"
#include "opus/internal/pvm.h"

using opus::internal::OpusSession;
using opus::internal::DBTr;
using opus::internal::PVMCache;
using opus::internal::pvm_parse;

using opus::trace::TraceReaderHandler;
using opus::trace::TraceEvent;

using namespace rapidjson;

OpusHdl *opus_init(Config cfg) {
  neo4j_client_init();
  auto session = new OpusSession(cfg);
  return session->to_hdl();
}

void print_cfg(OpusHdl const *hdl) {
  auto session = OpusSession::from_hdl(hdl);
  auto cfg = session->get_cfg();
  printf("libOpus configuration");
  printf("db_server: %s", cfg->db_server);
  printf("db_user: %s", cfg->db_user);
  printf("db_password: %s", cfg->db_password);
}

void process_events(OpusHdl *hdl, int fd) {
  auto session = OpusSession::from_hdl(hdl);
  Reader reader;
  TraceReaderHandler handler;
  PVMCache pvm_cache;
  std::queue<std::unique_ptr<DBTr>> trans;

  auto fp = fdopen(fd, "r");

  char line[65536];
  memset(line, '\0', 65536);
  while (fgets(line, 65536, fp) != nullptr) {
    StringStream s(line);
    if (reader.Parse(s, handler)) {
      auto tr = std::move(handler.get_events()->front());
      handler.get_events()->clear();
      pvm_parse(*tr, &pvm_cache, &trans);
    }
    memset(line, '\0', 65536);
  }
  auto db = session->db();
  while (!trans.empty()) {
    trans.front()->execute(db);
    trans.pop();
  }
}

void opus_cleanup(OpusHdl *hdl) {
  auto session = OpusSession::from_hdl(hdl);
  delete session;
  neo4j_client_cleanup();
}
