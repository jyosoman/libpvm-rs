#include "opus.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>

int main(int argc, char** argv) {
  char* user = malloc(5*sizeof(char));
  strcpy(user, "neo4j");

  int in = open("data.json", O_RDONLY);

  Config cfg = { Auto, "localhost:7687", user, "opus", 0 };
  OpusHdl* hdl = opus_init(cfg);
  printf("Rust C API handle ptr: hdl(%p) \n", hdl);

  print_cfg(hdl);
  strcpy(user, "dummy_info");

  printf("File fd: %d\n", in);
  process_events(hdl, in);

  opus_cleanup(hdl);

  return 0;
}
