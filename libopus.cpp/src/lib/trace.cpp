#include "opus/internal/trace.h"

#include <cstddef>
#include <iostream>

namespace opus {
  namespace trace {

using namespace rapidjson;

#define PARSE_KEY(field, maskv, key, k_len) {          \
  if(memcmp(str, key, k_len) == 0) {                   \
    trace_member_offset = offsetof(TraceEvent, field); \
    state_ = kExpectValue;                             \
    current_event_mask |= maskv;                       \
  }                                                    \
}


bool TraceReaderHandler::StartObject() {
  switch(state_) {
    case kExpectObjectStart: {
      state_ = kExpectKeyOrEndObject;
      current_event = std::make_unique<TraceEvent>();
      return true;
    }
    default: {
      std::clog << "StartObject wrong state";
      return false;
    }
  }
}

bool TraceReaderHandler::EndObject(SizeType) {
  bool ret;
  if((current_event_mask & TraceEvent_required) == TraceEvent_required){
    ret = (state_ == kExpectKeyOrEndObject || state_ == kIgnoreValue);
    state_ = kExpectObjectStart;
    events.push_back(std::move(current_event));
    return ret;
  }
  else {
    std::clog<<"Event missing required fields: "<<
      (current_event_mask ^ TraceEvent_required)<<std::endl;
    return false;
  }
}
bool TraceReaderHandler::StartArray() {
  state_ = kIgnoreValue;
  return true;
}
bool TraceReaderHandler::EndArray(SizeType) {
  state_ = kExpectKeyOrEndObject;
  return true;
}

// This function uses the properties of CADETS traces keys for identifying
// them using the minimum number of comparisons.
// Criteria used: key length, selective character comparisons for keys of
// the same length.
bool TraceReaderHandler::Key(const char* str, SizeType len, bool) {
  switch(state_) {
    case kExpectKeyOrEndObject:
    case kIgnoreValue: {
      state_ = kIgnoreValue;
      switch(len) {
        case 2: {
          if(str[0]=='f')
            PARSE_KEY(fd, FD, "fd", 2);
          break;
        }
        case 3: {
          switch(str[0]){
            case 'p':
              PARSE_KEY(pid, PID, "pid", 3);
              break;
            case 't':
              PARSE_KEY(tid, TID, "tid", 3);
              break;
            case 'u':
              PARSE_KEY(uid, UID, "uid", 3);
              break;
          }
          break;
        }
        case 4: {
          switch(str[0]){
            case 'h':
              PARSE_KEY(host, HOST, "host", 4);
              break;
            case 't':
              PARSE_KEY(time, TIME, "time", 4);
              break;
            case 'p':
              PARSE_KEY(ppid, PPID, "ppid", 4);
              break;
            case 'e':
              PARSE_KEY(exec, EXEC, "exec", 4);
              break;
          }
          break;
        }
        case 5: {
          switch(str[0]){
            case 'e':
              PARSE_KEY(event, EVENT, "event", 5);
              break;
            case 'f':
              PARSE_KEY(flags, FLAGS, "flags", 5);
              break;
          }
          break;
        }
        case 6: {
          switch(str[5]){
            case 'h':
              PARSE_KEY(fdpath, FDPATH, "fdpath", 6);
              break;
            case '1':
              PARSE_KEY(upath1, UPATH1, "upath1", 6);
              break;
            case '2':
              PARSE_KEY(upath2, UPATH2, "upath2", 6);
              break;
            case 'l':
              PARSE_KEY(retval, RETVAL, "retval", 6);
              break;
          }
          break;
        }
        case 7: {
          switch(str[0]){
            case 'c':
              PARSE_KEY(cmdline, CMDLINE, "cmdline", 7);
              break;
            case 'a':
              PARSE_KEY(address, ADDRESS, "address", 7);
              break;
          }
          if (str[0] == 'a') {
            trace_member_offset = offsetof(TraceEvent, address);
            current_event_mask |= ADDRESS;
          }
          break;
        }
        case 11: {
          if(str[0]=='s')
            PARSE_KEY(subjthruuid, SUBJTHRUUID, "subjthruuid", 11);
          break;
        }
        case 12: {
          switch(str[0]){
            case 's':
              PARSE_KEY(subjprocuuid, SUBJPROCUUID, "subjprocuuid", 12);
              break;
            case 'a': {
              if(str[11] == '1')
                PARSE_KEY(arg_objuuid1, ARGOBJUUID1, "arg_objuuid1", 12)
              else
                PARSE_KEY(arg_objuuid2, ARGOBJUUID2, "arg_objuuid2", 12)
              break;
            }
            case 'r': {
              if(str[11] == '1')
                PARSE_KEY(ret_objuuid1, RETOBJUUID1, "ret_objuuid1", 12)
              else
                PARSE_KEY(ret_objuuid2, RETOBJUUID2, "ret_objuuid2", 12)
              break;
            }
          }
          break;
        }
      }
      return true;
    }
    default: {
      std::cout<< "Unexpected state: "<< state_ <<std::endl;
      return false; // unexpected state
    }
  }
}

bool TraceReaderHandler::String(const char* str, SizeType, bool) {
  std::string* m = nullptr;
  switch(state_){
    case kExpectValue: {
      switch(trace_member_offset) {
        case offsetof(TraceEvent, event):
          m = &current_event->event;
          break;
        case offsetof(TraceEvent, host):
          m = &current_event->host;
          break;
        case offsetof(TraceEvent, exec):
          m = &current_event->exec;
          break;
        case offsetof(TraceEvent, cmdline):
          m = &current_event->cmdline;
          break;
        case offsetof(TraceEvent, upath1):
          m = &current_event->upath1;
          break;
        case offsetof(TraceEvent, upath2):
          m = &current_event->upath2;
          break;
        case offsetof(TraceEvent, address):
          m = &current_event->address;
          break;
        case offsetof(TraceEvent, fdpath):
          m = &current_event->fdpath;
          break;
        case offsetof(TraceEvent, subjprocuuid):
          m = &current_event->subjprocuuid;
          break;
        case offsetof(TraceEvent, subjthruuid):
          m = &current_event->subjthruuid;
          break;
        case offsetof(TraceEvent, arg_objuuid1):
          m = &current_event->arg_objuuid1;
          break;
        case offsetof(TraceEvent, arg_objuuid2):
          m = &current_event->arg_objuuid2;
          break;
        case offsetof(TraceEvent, ret_objuuid1):
          m = &current_event->ret_objuuid1;
          break;
        case offsetof(TraceEvent, ret_objuuid2):
          m = &current_event->ret_objuuid2;
          break;
      }
      // think of string_view and doing differen things whether the bool copy
      // argument is true or false
      if(m != nullptr) *m = str;
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      return true;
    default: return false;
  }
}

bool TraceReaderHandler::Uint64(uint64_t val) {
  switch(state_){
    case kExpectValue: {
      switch(trace_member_offset) {
        case offsetof(TraceEvent, time):
          current_event->time = val;
          break;
        default:
          std::clog<<"Key with offset "<<trace_member_offset<<
            " was wrongly interpreted as Uint64"<<std::endl;
      }
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      return true;
    default: return false;
  }
}

bool TraceReaderHandler::Int64(int64_t val) {
  switch(state_){
    case kExpectValue: {
      switch(trace_member_offset) {
        case offsetof(TraceEvent, time):
          current_event->time = static_cast<uint64_t>(val);
          break;
        default:
          std::clog<<"Key with offset "<<trace_member_offset<<
            " was wrongly interpreted as Int64"<<std::endl;
      }
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      return true;
    default: return false;
  }
}
bool TraceReaderHandler::Uint(unsigned val) {
  switch(state_){
    case kExpectValue: {
      switch(trace_member_offset) {
        case offsetof(TraceEvent, pid):
          current_event->pid = val;
          break;
        case offsetof(TraceEvent, ppid):
          current_event->ppid = val;
          break;
        case offsetof(TraceEvent, tid):
          current_event->tid = val;
          break;
        case offsetof(TraceEvent, uid):
          current_event->uid = val;
          break;
        case offsetof(TraceEvent, fd):
          current_event->fd = val;
          break;
        case offsetof(TraceEvent, flags):
          current_event->flags = val;
          break;
        case offsetof(TraceEvent, retval):
          current_event->retval = val;
          break;
        default:
          std::clog<<"Key with offset "<<trace_member_offset<<
            " was wrongly interpreted as Uint"<<std::endl;
      }
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      return true;
    default: return false;
  }
}
bool TraceReaderHandler::Int(int val) {
  switch(state_){
    case kExpectValue: {
      switch(trace_member_offset) {
        case offsetof(TraceEvent, pid):
          current_event->pid = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, ppid):
          current_event->ppid = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, tid):
          current_event->tid = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, uid):
          current_event->uid = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, fd):
          current_event->fd = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, flags):
          current_event->flags = static_cast<uint32_t>(val);
          break;
        case offsetof(TraceEvent, retval):
          current_event->retval = static_cast<uint32_t>(val);
          break;
        default:
          std::clog<<"Key with offset "<<trace_member_offset<<
            " was wrongly interpreted as Int"<<std::endl;
      }
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      return true;
    default: return false;
  }
}

  } // namespace trace
} // namespace opus
