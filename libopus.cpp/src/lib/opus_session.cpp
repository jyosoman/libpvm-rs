//
// Created by tb403 on 04/09/17.
//
#include "opus_session.h"

OpusSession::OpusSession(Config cfg) {
  this->cfg = cfg;
}

OpusHdl* OpusSession::to_hdl() {
  return reinterpret_cast<OpusHdl*>(this);
}

OpusSession* OpusSession::from_hdl(OpusHdl *hdl) {
  return reinterpret_cast<OpusSession*>(hdl);
}