// Copyright [2017] <Thomas Bytheway & Lucian Carata>
/**** Notice
 * api_test.cpp: libopus source code
 *
 **/

#include <fcntl.h>
#include <string>
#include "gtest/gtest.h"

#include "opus/internal/trace.h"

using std::string;
using namespace rapidjson;
using namespace opus::trace;

class ParseJsonTest : public testing::Test {
 protected:
  void SetUp() override {
  }

  void TearDown() override {
  }

};

TEST_F(ParseJsonTest,
       ParseOne) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"time\": 1475754879731575644, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"exec\": \"devd\", \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"arg_objuuid1\": \"e393303b-721f-8457-9f72-2da477847b65\", \"fd\": 3, \"retval\": 156,\"fdpath\": \"/dev/devctl\"}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r = reader.Parse(ss, handler);
  ASSERT_TRUE(r);

  auto evts = handler.get_events();
  TraceEvent te = *(evts->at(0));
  EXPECT_EQ(te.event, "audit:event:aue_read:");
  EXPECT_EQ(te.time, 1475754879731575644);
  EXPECT_EQ(te.pid, 407);
  EXPECT_EQ(te.ppid, 1);
  EXPECT_EQ(te.tid, 100062);
  EXPECT_EQ(te.uid, 0);
  EXPECT_EQ(te.exec, "devd");
  EXPECT_EQ(te.subjprocuuid, "93d41a15-8bbb-11e6-a64a-0800270779c7");
  EXPECT_EQ(te.subjthruuid, "89a75773-8bbb-11e6-a5db-0800270779c7");
  EXPECT_EQ(te.arg_objuuid1, "e393303b-721f-8457-9f72-2da477847b65");
  EXPECT_EQ(te.fd, 3);
  EXPECT_EQ(te.retval, 156);
  EXPECT_EQ(te.fdpath, "/dev/devctl");

}

TEST_F(ParseJsonTest,
       ParseMultiple) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"time\": 1475754879731575644, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"exec\": \"devd\", \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"arg_objuuid1\": \"e393303b-721f-8457-9f72-2da477847b65\", \"fd\": 3, \"retval\": 156,\"fdpath\": \"/dev/devctl\"}\n{\"event\": \"audit:event:aue_read:\", \"time\": 1111111111111222222, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"exec\": \"devd\", \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"arg_objuuid1\": \"e393303b-721f-8457-9f72-2da477847b65\", \"fd\": 3, \"retval\": 156,\"fdpath\": \"/dev/devctl\"}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r;
  while(true){
    r = reader.Parse<kParseStopWhenDoneFlag>(ss, handler);
    if(!r) {
      ParseErrorCode e = reader.GetParseErrorCode();
      if(e == kParseErrorDocumentEmpty) break;
      size_t o = reader.GetErrorOffset();
      std::clog << "Error: " << GetParseError_En(e) << std::endl;
      std::clog << " at offset " << o << "  near '" << string(json).substr(o, 10) << "...'" << std::endl;
      break;
    }
  }

  auto evts = handler.get_events();
  ASSERT_EQ(evts->size(), 2);

  TraceEvent te = *(evts->at(0));
  EXPECT_EQ(te.event, "audit:event:aue_read:");
  EXPECT_EQ(te.time, 1475754879731575644);
  EXPECT_EQ(te.pid, 407);
  EXPECT_EQ(te.ppid, 1);
  EXPECT_EQ(te.tid, 100062);
  EXPECT_EQ(te.uid, 0);
  EXPECT_EQ(te.exec, "devd");
  EXPECT_EQ(te.subjprocuuid, "93d41a15-8bbb-11e6-a64a-0800270779c7");
  EXPECT_EQ(te.subjthruuid, "89a75773-8bbb-11e6-a5db-0800270779c7");
  EXPECT_EQ(te.arg_objuuid1, "e393303b-721f-8457-9f72-2da477847b65");
  EXPECT_EQ(te.fd, 3);
  EXPECT_EQ(te.retval, 156);
  EXPECT_EQ(te.fdpath, "/dev/devctl");

  te = *(evts->at(1));
  EXPECT_EQ(te.event, "audit:event:aue_read:");
  EXPECT_EQ(te.time, 1111111111111222222);
  EXPECT_EQ(te.pid, 407);
  EXPECT_EQ(te.ppid, 1);
  EXPECT_EQ(te.tid, 100062);
  EXPECT_EQ(te.uid, 0);
  EXPECT_EQ(te.exec, "devd");
  EXPECT_EQ(te.subjprocuuid, "93d41a15-8bbb-11e6-a64a-0800270779c7");
  EXPECT_EQ(te.subjthruuid, "89a75773-8bbb-11e6-a5db-0800270779c7");
  EXPECT_EQ(te.arg_objuuid1, "e393303b-721f-8457-9f72-2da477847b65");
  EXPECT_EQ(te.fd, 3);
  EXPECT_EQ(te.retval, 156);
  EXPECT_EQ(te.fdpath, "/dev/devctl");
}

TEST_F(ParseJsonTest,
       ParseTrace_MissingRequiredFields) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"exec\": \"devd\", \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"arg_objuuid1\": \"e393303b-721f-8457-9f72-2da477847b65\", \"fd\": 3, \"retval\": 156,\"fdpath\": \"/dev/devctl\"}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r;
  r = reader.Parse(ss, handler);

  ASSERT_EQ(reader.GetParseErrorCode(), kParseErrorTermination);
}

TEST_F(ParseJsonTest,
       ParseTrace_MissingOptionalFields) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"time\": 123333333333, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"retval\": 156}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r;
  r = reader.Parse(ss, handler);
  EXPECT_TRUE(r);

  if(!r) {
    ASSERT_NE(reader.GetParseErrorCode(), kParseErrorTermination);
  }
}

TEST_F(ParseJsonTest,
       ParseTrace_UnknownFields) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"time\": 123333333333, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"address\": \"public/pickup\", \"retval\": 156, \"fictitious\":42}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r;
  r = reader.Parse(ss, handler);
  EXPECT_TRUE(r);

  if(!r) {
    EXPECT_NE(reader.GetParseErrorCode(), kParseErrorTermination);

    ParseErrorCode e = reader.GetParseErrorCode();
    size_t o = reader.GetErrorOffset();
    std::clog << "Error: " << GetParseError_En(e) << std::endl;
    std::clog << " at offset " << o << "  near '" << string(json).substr(o, 10) << "...'" << std::endl;
  }
}

TEST_F(ParseJsonTest,
       ParseTrace_ArrayValues) {

  const char* json = "{\"event\": \"audit:event:aue_read:\", \"time\": 123333333333, \"pid\": 407, \"ppid\": 1, \"tid\": 100062, \"uid\": 0, \"subjprocuuid\": \"93d41a15-8bbb-11e6-a64a-0800270779c7\", \"subjthruuid\": \"89a75773-8bbb-11e6-a5db-0800270779c7\", \"address\": \"public/pickup\", \"retval\": 156, \"fictitious\":[42, \"hello\"]}";
  Reader reader;
  TraceReaderHandler handler;
  StringStream ss(json);
  ParseResult r;
  r = reader.Parse(ss, handler);
  EXPECT_TRUE(r);

  if(!r) {
    ASSERT_NE(reader.GetParseErrorCode(), kParseErrorTermination);
    ParseErrorCode e = reader.GetParseErrorCode();
    size_t o = reader.GetErrorOffset();
    std::clog << "Error: " << GetParseError_En(e) << std::endl;
    std::clog << " at offset " << o << "  near '" << string(json).substr(o, 10) << "...'" << std::endl;
  }
}
