#include "operator_timing.h"
#include <cassert>
#include <iostream>
#include <type_traits>
uint64_t tick=0;uint64_t clock_tick(){tick+=100;return tick;}
int main(){
 static_assert(operator_stage_count==19);static_assert(!std::is_copy_constructible_v<OperatorInterval>);
 OperatorCounters counters{};bool called=false;
 {OperatorInterval off(counters,false,clock_tick);off.finish(OperatorStage::Embedding,[&]{called=true;});}
 assert(!called&&tick==0&&counters[0].calls==0);
 {OperatorInterval on(counters,true,clock_tick);on.finish(OperatorStage::Embedding,[&]{called=true;});
  bool rejected=false;try{on.finish(OperatorStage::Embedding,[]{});}catch(const std::logic_error&){rejected=true;}assert(rejected);}
 assert(called&&counters[0].calls==1&&counters[0].host_ns==100&&counters[0].wait_ns==100);
 {OperatorInterval failed(counters,true,clock_tick);try{failed.finish(OperatorStage::MoeDown,[]{throw std::runtime_error("injected");});}catch(const std::runtime_error&){} }
 assert(counters[static_cast<size_t>(OperatorStage::MoeDown)].calls==0);
 {OperatorInterval invalid(counters,true,clock_tick);bool rejected=false;try{invalid.finish(OperatorStage::Count,[]{});}catch(const std::logic_error&){rejected=true;}assert(rejected);}
 std::cout<<"PASS 19-stage inventory, disabled zero clocks/evals, exact intervals, failed cuts, duplicate/invalid cuts, noncopyable ownership\n";
}
