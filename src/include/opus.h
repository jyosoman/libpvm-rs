
#ifndef cheddar_generated_opus_h
#define cheddar_generated_opus_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef size_t ViewHdl;

typedef size_t ViewInstHdl;

typedef struct {
  char* key;
  char* val;
} KeyVal;

typedef struct {
  ViewHdl id;
  const char* name;
  const char* desc;
  size_t num_parameters;
  KeyVal* parameters;
} View;

typedef struct {
  ViewInstHdl id;
  ViewHdl type;
  size_t num_parameters;
  KeyVal* parameters;
} ViewInst;

typedef enum CfgMode {
  Auto,
  Advanced,
} CfgMode;

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

typedef struct OpusHdl OpusHdl;

OpusHdl* opus_init(Config cfg);

void opus_start_pipeline(OpusHdl* hdl);

void opus_shutdown_pipeline(OpusHdl* hdl);

size_t opus_list_view_types(OpusHdl const* hdl, View** out);

ViewInstHdl opus_create_view(OpusHdl* hdl, ViewHdl view_id, KeyVal* params, size_t n_params);

size_t opus_list_view_inst(OpusHdl const* hdl, ViewInst** out);

void opus_print_cfg(OpusHdl const* hdl);

void opus_ingest_fd(OpusHdl* hdl, int fd);

void opus_cleanup(OpusHdl* hdl);

int64_t opus_count_processes(OpusHdl* hdl);

#ifdef __cplusplus
}
#endif


#endif
