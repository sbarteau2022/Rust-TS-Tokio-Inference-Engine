#pragma once
#include <chrono>
#include <cstdint>
#include <stdexcept>
// Worker-confined host intervals. No field is a GPU timestamp or token counter.
struct ExecutionMetrics {uint64_t calls=0,completed=0,graph_ns=0,wait_ns=0,export_ns=0,wall_ns=0,groups=0;};
struct ForwardTiming {
 using Clock=uint64_t(*)();
 ExecutionMetrics& metrics;bool enabled,finished=false;Clock clock;uint64_t start=0,last=0;
 static uint64_t now(){return uint64_t(std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now().time_since_epoch()).count());}
 ForwardTiming(ExecutionMetrics& m,bool on,Clock read=now):metrics(m),enabled(on),clock(read){if(enabled){start=last=clock();++metrics.calls;}}
 ForwardTiming(const ForwardTiming&)=delete;ForwardTiming& operator=(const ForwardTiming&)=delete;
 void open(){if(finished)throw std::logic_error("timing interval after completion");}
 void graph(){if(enabled){open();auto n=clock();metrics.graph_ns+=n-last;last=n;}}
 void wait(){if(enabled){open();auto n=clock();metrics.wait_ns+=n-last;last=n;++metrics.groups;}}
 void complete(){if(enabled){open();auto n=clock();metrics.export_ns+=n-last;last=n;++metrics.completed;finished=true;}}
 ~ForwardTiming(){if(enabled)metrics.wall_ns+=clock()-start;}
};
