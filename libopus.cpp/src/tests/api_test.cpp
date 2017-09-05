/**** Notice
 * api_test.cpp: libopus source code
 *
 **/

#include <fcntl.h>
#include <string>
#include "gtest/gtest.h"

#include "opus/opus.h"
#include "../lib/db_tr.h"
#include "../lib/opus_session.h"

class APITest : public testing::Test {
 protected:
  void SetUp() override {
    Config cfg = {Auto,
                  const_cast<char*>("bolt://localhost"),
                  const_cast<char*>("neo4j"),
                  const_cast<char*>("opus"),
                  nullptr};
    hdl = opus_init(cfg);
    ASSERT_NE(nullptr, hdl);
  }

  void TearDown() override {
    opus_cleanup(hdl);
  }

  OpusHdl *hdl;
};

TEST_F(APITest,
       PrintConfig) {
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
