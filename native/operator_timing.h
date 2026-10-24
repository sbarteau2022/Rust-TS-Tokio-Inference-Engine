#pragma once
#include "execution_timing.h"
#include <array>
#include <cstddef>
enum class OperatorStage : size_t {
 Embedding,InputNorm,RecurrentProjection,RecurrentPrepare,RecurrentUpdate,RecurrentOutput,
 AttentionProjection,AttentionReduction,AttentionOutput,ResidualNorm,MoeRouting,MoeUpGate,
 MoeDown,MoeMixture,MoeShared,MoeMerge,LayerOutput,FinalProjection,StateValidation,Count
};
constexpr size_t operator_stage_count=static_cast<size_t>(OperatorStage::Count);
struct OperatorMetrics {uint64_t calls=0,host_ns=0,wait_ns=0;};
using OperatorCounters=std::array<OperatorMetrics,operator_stage_count>;
// Successful diagnostic cuts only; never nested. Disabled mode invokes no clock or eval.
struct OperatorInterval {
 OperatorCounters& counters;bool enabled,finished=false;ForwardTiming::Clock clock;uint64_t start=0;
 OperatorInterval(OperatorCounters& c,bool on,ForwardTiming::Clock read=ForwardTiming::now):counters(c),enabled(on),clock(read){if(enabled)start=clock();}
 OperatorInterval(const OperatorInterval&)=delete;OperatorInterval& operator=(const OperatorInterval&)=delete;
 template<class F> void finish(OperatorStage stage,F eval){
  if(!enabled)return;
  if(finished||static_cast<size_t>(stage)>=operator_stage_count)throw std::logic_error("invalid/repeated operator cut");
  auto host_end=clock();eval();auto end=clock();auto& m=counters[static_cast<size_t>(stage)];
  ++m.calls;m.host_ns+=host_end-start;m.wait_ns+=end-host_end;finished=true;
 }
};
