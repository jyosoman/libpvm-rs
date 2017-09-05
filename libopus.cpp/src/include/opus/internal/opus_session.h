//
// Created by tb403 on 04/09/17.
//
#ifndef LIBOPUS_CPP_SRC_LIB_OPUS_SESSION_H_
#define LIBOPUS_CPP_SRC_LIB_OPUS_SESSION_H_

#include "opus/opus.h"

#include <neo4j-client.h>

namespace opus {
namespace internal {

class OpusSession {
  Config cfg;
  neo4j_connection_t *conn;
 public:
  explicit OpusSession(Config cfg);

  ~OpusSession();

  OpusHdl *to_hdl();

  static OpusSession *from_hdl(OpusHdl *hdl);

  neo4j_connection_t *db();
};

}
}

#endif  // LIBOPUS_CPP_SRC_LIB_OPUS_SESSION_H_
