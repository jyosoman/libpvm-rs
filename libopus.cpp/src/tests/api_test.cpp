/**** Notice
 * api_test.cpp: libopus source code
 *
 **/

#include <errno.h>
#include <fcntl.h>
#include <string>
#include "gtest/gtest.h"
#include <stdio.h>

#include "opus/opus.h"
#include "../lib/db_tr.h"
#include "../lib/opus_session.h"

#include <neo4j-client.h>

class APITest : public testing::Test
{
 protected:
  virtual void SetUp()
  {
    Config cfg = { Auto,
                  (char*)"localhost:7687",
                  (char*)"neo4j",
                  (char*)"opus",
                  0 };
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

TEST_F(APITest,
       ProcessEvents) {
  auto session = OpusSession::from_hdl(hdl);

  auto conn = session->db();

  if (conn == nullptr) {
    FAIL();
  }

  auto tr = new DBCreateNode(1,
                             std::string("00000000-0000-0000-0000-000000000000"),
                             42,
                             std::string("foo"));

  tr->execute(conn);

  delete tr;

  ASSERT_TRUE(true);
}