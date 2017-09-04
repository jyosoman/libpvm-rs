/**** Notice
 * api_test.cpp: libopus source code
 *
 **/

#include <errno.h>
#include <fcntl.h>
#include "gtest/gtest.h"
#include <stdio.h>

#include "opus/opus.h"

class APITest : public testing::Test
{
 protected:
  virtual void SetUp()
  {
    Config cfg = { Auto, "localhost:7687", "neo4j", "opus", 0 };
    hdl = opus_init(cfg);
    ASSERT_NE(nullptr, hdl);
  }

  virtual void TearDown()
  {
    opus_cleanup(hdl);
  }

  OpusHdl* hdl;

};


TEST_F(APITest,
       PrintConfig)
{
  print_cfg(hdl);
  ASSERT_TRUE(true);
}

