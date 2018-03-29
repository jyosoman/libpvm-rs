
#ifndef cheddar_generated_opus_h
#define cheddar_generated_opus_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



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
	char* cypher_file;
	AdvancedConfig* cfg_detail;
} Config;

typedef struct OpusHdl OpusHdl;

OpusHdl* opus_init(Config cfg);

void print_cfg(OpusHdl const* hdl);

void process_events(OpusHdl* hdl, int fd, bool db, bool cypher);

void opus_cleanup(OpusHdl* hdl);

int64_t count_processes(OpusHdl* hdl);

#ifdef __cplusplus
}
#endif


#endif
