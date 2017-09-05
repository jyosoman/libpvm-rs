// Copyright [2017] <Thomas Bytheway & Lucian Carata>
/**** Notice
 * api_test.cpp: libopus source code
 *
 **/

#include <fcntl.h>
#include <string>
#include "gtest/gtest.h"

#include "opus/opus.h"
#include "opus/internal/db_tr.h"
#include "opus/internal/opus_session.h"

using std::string;
using opus::internal::OpusSession;
using opus::internal::DBCreateNode;

class APITest : public testing::Test {
 protected:
  void SetUp() override {
    Config cfg = {Auto,
                  const_cast<char *>("bolt://localhost"),
                  const_cast<char *>("neo4j"),
                  const_cast<char *>("opus"),
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

  ASSERT_NE(conn, nullptr) << hdl->message;

  if (neo4j_check_failure(
      neo4j_send(conn, "BEGIN", neo4j_null)) != 0) {
    FAIL();
  }

  for (int i = 1; i <= 30000; i++) {
    auto tr = new DBCreateNode(i,
                               string("00000000-0000-0000-0000-000000000000"),
                               42,
                               string("foo"));

    tr->execute(conn);
    delete tr;
  }

  if (neo4j_check_failure(
      neo4j_send(conn, "COMMIT", neo4j_null)) != 0) {
    FAIL();
  }
}
