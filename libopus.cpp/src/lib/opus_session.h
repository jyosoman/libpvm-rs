//
// Created by tb403 on 04/09/17.
//
#ifndef LIBOPUS_LIBOPUS_H_H
#define LIBOPUS_LIBOPUS_H_H

#include "opus/opus.h"

class OpusSession{
  Config cfg;
public:
  explicit OpusSession(Config cfg);

  OpusHdl* to_hdl();
  static OpusSession* from_hdl(OpusHdl* hdl);
};

#endif //LIBOPUS_LIBOPUS_H_H
