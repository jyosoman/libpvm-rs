#include "opus/opus.h"

#include <iostream>
#include <neo4j-client.h>

using namespace std;

OpusHdl* opus_init(Config cfg){
  neo4j_client_init();
  return NULL;
}

void print_cfg(OpusHdl const* hdl){
  cout<<"libOpus configuration"<<endl;
}

void process_events(OpusHdl* hdl, int fd){
}

void opus_cleanup(OpusHdl* hdl){
  neo4j_client_cleanup();
}
