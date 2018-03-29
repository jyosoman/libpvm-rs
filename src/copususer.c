#include "opus.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>

int main(int argc, char** argv) {
  char* user = malloc(5*sizeof(char));
  strcpy(user, "neo4j");

  int in = 0;
  if(strcmp(argv[1], "-") != 0){
    in = open(argv[1], O_RDONLY);
  }

  Config cfg = { Auto, "localhost:7687", user, "opus", "./cypher.db", 0 };
  OpusHdl* hdl = opus_init(cfg);
  printf("Rust C API handle ptr: hdl(%p) \n", hdl);

  print_cfg(hdl);

  // test to see whether rust has copied the underlying memory or still
  // refers to C memory (the user should remain "neo4j" as far as rust
  // is concerned)
  strcpy(user, "dummy_info");

  printf("File fd: %d\n", in);
  process_events(hdl, in, false, true);

  printf("Number of processes: %lld\n", count_processes(hdl));

  opus_cleanup(hdl);

  return 0;
}
