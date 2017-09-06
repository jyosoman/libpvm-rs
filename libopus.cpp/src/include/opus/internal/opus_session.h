// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 04/09/17.
//
#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_OPUS_SESSION_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_OPUS_SESSION_H_

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

  const Config *get_cfg() const { return &(this->cfg); }

  OpusHdl *to_hdl();

  static OpusSession *from_hdl(OpusHdl *hdl);
  static const OpusSession *from_hdl(const OpusHdl *hdl);

  neo4j_connection_t *db();
};

}  // namespace internal
}  // namespace opus

#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_OPUS_SESSION_H_
