#pragma once
#include <cstdint>
#include <limits>
#include <optional>
#include <stdexcept>
#include <utility>
#include <vector>
// Worker-confined. Logical full-state charges intentionally overcount shared buffers.
#ifndef RS_QWEN_STATE_ID_DEFINED
#define RS_QWEN_STATE_ID_DEFINED
struct StateId {uint64_t owner,slot,generation,position,bytes;};
#endif
static_assert(sizeof(StateId)==40);
template<class T,uint64_t GenerationLimit=std::numeric_limits<uint64_t>::max()> class StateTable {
 static_assert(GenerationLimit>0);
 struct Entry {uint64_t generation=0,position=0,bytes=0;bool reserved=false;std::optional<T> value;};
 uint64_t owner_,cap_,used_=0;std::vector<Entry> entries_;
 Entry& checked(StateId id){
  if(id.owner!=owner_||id.slot>=entries_.size())throw std::invalid_argument("foreign state owner/slot");
  auto& e=entries_[id.slot];
  if(!e.reserved||e.generation!=id.generation||e.position!=id.position||e.bytes!=id.bytes)throw std::invalid_argument("stale state generation/geometry");return e;
 }
 public:
 StateTable(uint64_t owner,size_t slots,uint64_t bytes):owner_(owner),cap_(bytes),entries_(slots){if(!owner||!slots||slots>64||!bytes)throw std::invalid_argument("state table bounds");}
 StateId reserve(uint64_t position,uint64_t bytes){
  if(!position||!bytes||bytes>cap_-used_)throw std::runtime_error("resident state byte budget exhausted");
  for(size_t i=0;i<entries_.size();++i){auto& e=entries_[i];if(!e.reserved&&e.generation!=GenerationLimit){
   ++e.generation;e.position=position;e.bytes=bytes;e.reserved=true;used_+=bytes;return {owner_,i,e.generation,position,bytes};}}
  throw std::runtime_error("resident state slots exhausted");
 }
 void publish(StateId id,T value){auto& e=checked(id);if(e.value)throw std::invalid_argument("state already published");e.value.emplace(std::move(value));}
 const T& get(StateId id){auto& e=checked(id);if(!e.value)throw std::invalid_argument("unpublished state");return *e.value;}
 void rollback(StateId id){auto& e=checked(id);e.value.reset();e.reserved=false;used_-=e.bytes;}
 void release(StateId id){(void)get(id);rollback(id);}
 uint64_t bytes() const{return used_;}
 size_t residents() const{size_t n=0;for(const auto& e:entries_)n+=e.reserved;return n;}
 void clear(){for(auto& e:entries_){e.value.reset();e.reserved=false;}used_=0;}
};
