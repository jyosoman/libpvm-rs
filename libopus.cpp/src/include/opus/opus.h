// Copyright [2017] <Thomas Bytheway & Lucian Carata>

#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_OPUS_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_OPUS_H_


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>


typedef enum CfgMode {
  Auto,
  Advanced,
} CfgMode;

typedef enum OpusErr {
  NO_ERR,
  ERR_DATABASE,
  ERR_PARSING,
  ERR_PVM,
} OpusErr;

typedef struct AdvancedConfig {
  int32_t consumer_threads;
  int32_t persistence_threads;
} AdvancedConfig;

typedef struct Config {
  CfgMode cfg_mode;
  char* db_server;
  char* db_user;
  char* db_password;
  AdvancedConfig* cfg_detail;
} Config;

typedef struct OpusHdl {
  OpusErr err;
  const char* message;

  void* _internal;
} OpusHdl;

OpusHdl* opus_init(Config cfg);

void print_cfg(OpusHdl const* hdl);

void process_events(OpusHdl* hdl, int fd);

void opus_cleanup(OpusHdl* hdl);



#ifdef __cplusplus
}
#endif


#endif  // LIBOPUS_CPP_SRC_INCLUDE_OPUS_OPUS_H_
