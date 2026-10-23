#include "execution_timing.h"
#include <cassert>
#include <iostream>
#include <type_traits>
uint64_t tick=0;
uint64_t fake_clock(){tick+=100;return tick;}
int main(){
 static_assert(!std::is_copy_constructible_v<ForwardTiming>);
 ExecutionMetrics off;
 {ForwardTiming t(off,false,fake_clock);t.graph();t.wait();t.complete();}
 assert(tick==0&&off.calls==0&&off.wall_ns==0&&off.groups==0);
 ExecutionMetrics on;
 {ForwardTiming t(on,true,fake_clock);t.graph();t.wait();t.complete();
  bool rejected=false;try{t.complete();}catch(const std::logic_error&){rejected=true;}assert(rejected);
 }
 assert(on.calls==1&&on.completed==1&&on.graph_ns==100&&on.wait_ns==100&&on.export_ns==100&&on.wall_ns==400&&on.groups==1);
 {ForwardTiming t(on,true,fake_clock);t.graph();t.wait();} // abandoned/failed attempt never claims success
 assert(on.calls==2&&on.completed==1&&on.wall_ns==700&&on.graph_ns==200&&on.wait_ns==200&&on.groups==2);
 assert(on.graph_ns+on.wait_ns+on.export_ns<=on.wall_ns);
 std::cout<<"PASS timing off has zero clock calls; exact interval accounting; failed attempts; duplicate completion rejection; noncopyable owner\n";
}
