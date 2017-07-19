#include "opus.h"

int main() {
  Config cfg { Auto, "opus", "neo4j" };
  OpusHdl* hdl = opus_init(cfg);

  print_cfg(hdl);

  opus_cleanup(hdl);

  return 0;
}
