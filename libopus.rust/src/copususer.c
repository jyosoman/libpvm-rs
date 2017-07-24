#include "opus.h"

int main() {
  Config cfg = { Auto, "neo4j", "opus", 0 };
  OpusHdl* hdl = opus_init(cfg);

  print_cfg(hdl);

  opus_cleanup(hdl);

  return 0;
}
