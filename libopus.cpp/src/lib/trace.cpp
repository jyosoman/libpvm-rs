#include "opus/internal/trace.h"

#include <cstddef>
#include <iostream>

namespace opus {
  namespace trace {

using namespace rapidjson;

#define PARSE_KEY(field, maskv) {  \
  trace_memeber_offset = offsetof(TraceEvent, field); \
  current_event_mask |= maskv; \
} \


bool TraceReaderHandler::StartObject() {
  switch(state_) {
    case kExpectObjectStart: {
      state_ = kExpectKeyOrEndObject;
      current_key = 0;
      current_event = std::make_unique<TraceEvent>();
      return true;
    }
    default: return false;
  }
}

bool TraceReaderHandler::EndObject(SizeType) {
  bool ret;
  if((current_event_mask & TraceEvent_required) == TraceEvent_required){
    ret = (state_ == kExpectKeyOrEndObject);
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

bool TraceReaderHandler::Key(const char* str, SizeType len, bool) {
  switch(state_) {
    case kExpectKeyOrEndObject: {
      state_ = kExpectValue;
      current_key++;
      switch(len) {
        case 2: {
          if(str[0]=='f'){
            trace_member_offset = offsetof(TraceEvent, fd);
            current_event_mask |= FD;
          }
          break;
        }
        case 3: {
          switch(str[0]){
            case 'p': {
                trace_member_offset = offsetof(TraceEvent, pid);
                current_event_mask |= PID;
                break;
            }
            case 't': {
                trace_member_offset = offsetof(TraceEvent, tid);
                current_event_mask |= TID;
                break;
            }
            case 'u': {
                trace_member_offset = offsetof(TraceEvent, uid);
                current_event_mask |= UID;
                break;
            }
          }
          break;
        }
        case 4: {
          switch(str[0]){
            case 'h': {
                trace_member_offset = offsetof(TraceEvent, host);
                current_event_mask |= HOST;
                break;
            }
            case 't': {
                trace_member_offset = offsetof(TraceEvent, time);
                current_event_mask |= TIME;
                break;
            }
            case 'p': {
                trace_member_offset = offsetof(TraceEvent, ppid);
                current_event_mask |= PPID;
                break;
            }
            case 'e': {
                trace_member_offset = offsetof(TraceEvent, exec);
                current_event_mask |= EXEC;
                break;
            }
          }
          break;
        }
        case 5: {
          switch(str[0]){
            case 'e': {
                trace_member_offset = offsetof(TraceEvent, event);
                current_event_mask |= EVENT;
                break;
            }
            case 'f': {
                trace_member_offset = offsetof(TraceEvent, flags);
                current_event_mask |= FLAGS;
                break;
            }
          }
          break;
        }
        case 6: {
          switch(str[5]){
            case 'h': {
                trace_member_offset = offsetof(TraceEvent, fdpath);
                current_event_mask |= FDPATH;
                break;
            }
            case '1': {
                trace_member_offset = offsetof(TraceEvent, upath1);
                current_event_mask |= UPATH1;
                break;
            }
            case '2': {
                trace_member_offset = offsetof(TraceEvent, upath2);
                current_event_mask |= UPATH2;
                break;
            }
            case 'l': {
                trace_member_offset = offsetof(TraceEvent, retval);
                current_event_mask |= RETVAL;
                break;
            }
          }
          break;
        }
        case 7: {
          if(str[0]=='c'){
            trace_member_offset = offsetof(TraceEvent, cmdline);
            current_event_mask |= CMDLINE;
          }
          break;
        }
        case 11: {
          if(str[0]=='s'){
            trace_member_offset = offsetof(TraceEvent, subjthruuid);
            current_event_mask |= SUBJTHRUUID;
          }
          break;
        }
        case 12: {
          switch(str[0]){
            case 's': {
              trace_member_offset = offsetof(TraceEvent, subjprocuuid);
              current_event_mask |= SUBJPROCUUID;
              break;
            }
            case 'a': {
              if(str[11] == '1'){
                trace_member_offset = offsetof(TraceEvent, arg_objuuid1);
                current_event_mask |= ARGOBJUUID1;
              } else {
                trace_member_offset = offsetof(TraceEvent, arg_objuuid2);
                current_event_mask |= ARGOBJUUID2;
              }
              break;
            }
            case 'r': {
              if(str[11] == '1'){
                trace_member_offset = offsetof(TraceEvent, ret_objuuid1);
                current_event_mask |= RETOBJUUID1;
              } else {
                trace_member_offset = offsetof(TraceEvent, ret_objuuid2);
                current_event_mask |= RETOBJUUID2;
              }
              break;
            }
          }
          break;
        }
        default:
            state_=kIgnoreValue;
      }
      return true;
    }
    default: return false;
  }
}

bool TraceReaderHandler::String(const char* str, SizeType, bool) {
  std::string* m;
  switch(state_){
    case kExpectValue:{
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
      *m = str;
      state_ = kExpectKeyOrEndObject;
      return true;
    }
    case kIgnoreValue:
      state_ = kExpectKeyOrEndObject;
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
      state_ = kExpectKeyOrEndObject;
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
      state_ = kExpectKeyOrEndObject;
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
      state_ = kExpectKeyOrEndObject;
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
      state_ = kExpectKeyOrEndObject;
      return true;
    default: return false;
  }
}

  } // namespace trace
} // namespace opus
