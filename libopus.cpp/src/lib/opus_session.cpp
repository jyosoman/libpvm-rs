// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by tb403 on 04/09/17.
//
#include "opus/internal/opus_session.h"

#include <neo4j-client.h>
#include <cerrno>

namespace opus {
namespace internal {

#define ERR_MSG_LEN 1024

OpusSession::OpusSession(Config cfg) {
  this->cfg = cfg;
  this->conn = nullptr;
  this->_pinned_err_buf = new char[ERR_MSG_LEN];
  this->api_hdl.err = NO_ERR;
  this->api_hdl.message = "";
  this->api_hdl._internal = this;
}

OpusSession::~OpusSession() {
  if (this->conn != nullptr) {
    neo4j_close(this->conn);
  }
}

OpusHdl *OpusSession::to_hdl() {
  return &api_hdl;
}

OpusSession *OpusSession::from_hdl(OpusHdl *hdl) {
  return reinterpret_cast<OpusSession *>(hdl->_internal);
}

const OpusSession *OpusSession::from_hdl(const OpusHdl *hdl) {
  return reinterpret_cast<const OpusSession *>(hdl->_internal);
}

neo4j_connection_t *OpusSession::db() {
  if (this->conn == nullptr) {
    auto neo_cfg = neo4j_new_config();
    if (neo4j_config_set_username(neo_cfg, this->cfg.db_user) != 0) {
      return nullptr;
    }
    if (neo4j_config_set_password(neo_cfg, this->cfg.db_password) != 0) {
      return nullptr;
    }
    neo4j_config_set_max_pipelined_requests(neo_cfg, 5000);
    this->conn = neo4j_connect(this->cfg.db_server, neo_cfg, NEO4J_INSECURE);
    if (this->conn == nullptr) {
      api_hdl.err = ERR_DATABASE;
      api_hdl.message = neo4j_strerror(errno, _pinned_err_buf, ERR_MSG_LEN);
    }
    neo4j_config_free(neo_cfg);
  }
  return this->conn;
}

}  // namespace internal
}  // namespace opus
