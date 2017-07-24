#include "opus.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
  char* user = malloc(5*sizeof(char));
  strcpy(user, "neo4j");

  Config cfg = { Auto, user, "opus", 0 };
  OpusHdl* hdl = opus_init(cfg);
  printf("Rust C API handle ptr: hdl(%p) \n", hdl);

  print_cfg(hdl);
  strcpy(user, "dummy_info");

  process_events(hdl, 1);

  opus_cleanup(hdl);

  return 0;
}
