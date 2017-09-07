// Copyright [2017] <Thomas Bytheway & Lucian Carata>
//
// Created by lc525 on 06/09/17.
//
#ifndef LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_TRACE_H_
#define LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_TRACE_H_

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#include "rapidjson/reader.h"
#include "rapidjson/error/en.h"

namespace opus {
  namespace trace {
    using Uuid5 = std::string;

    enum TraceEventFields {
      EVENT = 1,
      HOST  = 1 << 1,
      TIME  = 1 << 2,
      PID   = 1 << 3,
      PPID  = 1 << 4,
      TID   = 1 << 5,
      UID   = 1 << 6,
      EXEC  = 1 << 7,
      CMDLINE = 1 << 8,
      UPATH1  = 1 << 9,
      UPATH2  = 1 << 10,
      ADDRESS = 1 << 11,
      FD      = 1 << 12,
      FLAGS   = 1 << 13,
      FDPATH  = 1 << 14,
      SUBJPROCUUID = 1 << 15,
      SUBJTHRUUID  = 1 << 16,
      ARGOBJUUID1  = 1 << 17,
      ARGOBJUUID2  = 1 << 18,
      RETOBJUUID1  = 1 << 19,
      RETOBJUUID2  = 1 << 20,
      RETVAL       = 1 << 21,
    };

    // Mask contains an 1 for every field required in TraceEvent. Every field in
    // the structure is represented on one bit starting with bit 0 (event) and
    // ending with bit 20 (retval).
    //
    // Required fields:
    // event, time, pid, ppid, tid, uid, subjprocuuid, subjthruuid, retval
    const uint32_t TraceEvent_required = EVENT | TIME | PID | PPID | TID | UID |
                                         SUBJPROCUUID | SUBJTHRUUID | RETVAL;

    struct TraceEvent {
      std::string event;
      std::string host;
      uint64_t time;
      uint32_t pid;
      uint32_t ppid;
      uint32_t tid;
      uint32_t uid;
      std::string exec;
      std::string cmdline;
      std::string upath1;
      std::string upath2;
      std::string address;
      int32_t fd;
      int32_t flags;
      std::string fdpath;
      Uuid5 subjprocuuid;
      Uuid5 subjthruuid;
      Uuid5 arg_objuuid1;
      Uuid5 arg_objuuid2;
      Uuid5 ret_objuuid1;
      Uuid5 ret_objuuid2;
      uint32_t retval;
    };

    class TraceReaderHandler :
      public rapidjson::BaseReaderHandler<rapidjson::UTF8<>, TraceReaderHandler>
    {
      public:
      TraceReaderHandler() : state_(kExpectObjectStart),
                             current_key(-1),
                             trace_member_offset(-1),
                             current_event_mask(0)
      {}

      bool StartObject();
      bool EndObject(rapidjson::SizeType);

      bool Key(const char* str, rapidjson::SizeType len, bool copy);
      bool String(const char* str, rapidjson::SizeType size, bool copy);
      bool Int(int);
      bool Uint(unsigned);
      bool Int64(int64_t);
      bool Uint64(uint64_t);

      bool Default() { return false; }

      std::vector<std::unique_ptr<TraceEvent>>* get_events(){
        return &events;
      }

      private:
      enum State {
          kExpectObjectStart,
          kExpectKeyOrEndObject,
          kExpectValue,
          kIgnoreValue,
      } state_;
      short current_key;
      short trace_member_offset;
      uint32_t current_event_mask;
      std::vector<std::unique_ptr<TraceEvent>> events;
      std::unique_ptr<TraceEvent> current_event;

    };
  } // namespace trace
} // namespace opus

#endif // LIBOPUS_CPP_SRC_INCLUDE_OPUS_INTERNAL_TRACE_H_

