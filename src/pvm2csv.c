#include "opus.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>

int main(int argc, char** argv) {
  if(argc != 3) {
    printf("pvm2csv trace-file csv-zip");
    return -1;
  }

  int in = 0;
  if(strcmp(argv[1], "-") != 0){
    in = open(argv[1], O_RDONLY);
  }

  Config cfg = { Auto, "", "", "", true, 0 };
  OpusHdl* hdl = opus_init(cfg);
  opus_start_pipeline(hdl);

  View* views;
  size_t num_views = opus_list_view_types(hdl, &views);

  for (int i=0; i<num_views; i++) {
    if(strcmp(views[i].name, "CSVView") == 0) {
      KeyVal params[1];
      params[0].key = "path";
      params[0].val = argv[2];
      opus_create_view(hdl, views[i].id, params, 1);
    }
  }

  for (int i=0; i<num_views; i++) {
    free((void*)views[i].name);
    free((void*)views[i].desc);
    free((void*)views[i].parameters);
  }
  free(views);

  opus_ingest_fd(hdl, in);
  opus_shutdown_pipeline(hdl);
  opus_cleanup(hdl);
  return 0;
}
