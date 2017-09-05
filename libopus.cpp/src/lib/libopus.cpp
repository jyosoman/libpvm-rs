#include <neo4j-client.h>

#include "opus/opus.h"
#include "opus/internal/opus_session.h"

using opus::internal::OpusSession;

OpusHdl *opus_init(Config cfg) {
  neo4j_client_init();
  auto session = new OpusSession(cfg);
  return session->to_hdl();
}

void print_cfg(OpusHdl const *hdl) {
  printf("libOpus configuration");
}

void process_events(OpusHdl *hdl, int fd) {
}

void opus_cleanup(OpusHdl *hdl) {
  auto session = OpusSession::from_hdl(hdl);
  delete session;
  neo4j_client_cleanup();
}
