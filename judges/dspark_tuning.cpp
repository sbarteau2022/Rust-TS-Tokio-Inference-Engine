#include "dspark_tuning.h"
#include <cassert>
#include <iostream>
using native_dspark::Tuning;
int main(){
 auto t=Tuning::parse({"bf16","--width=8","--confidence=0.9","--adaptive"});assert(t.head==2&&t.width==8&&t.confidence==.9f&&t.initial()==4);
 assert(t.next(4,3,3)==8&&t.next(8,7,5)==4&&t.next(4,0,0)==2&&t.next(2,1,0)==2&&t.next(8,7,7)==8);
 assert(Tuning::parse({"--width=2"}).next(2,0,0)==2);
 size_t refused=0;
 for(auto args:std::vector<std::vector<std::string>>{{"--width=0"},{"--width=9"},{"--width=4junk"},{"--width=4","--width=8"},{"--confidence=nan"},{"--confidence=-1"},{"--confidence=1"},{"--confidence=0.9","--confidence=none"},{"--adaptive","--adaptive"},{"bf16","checkpoint-head"},{"--unknown"},{"--width="}}){bool rejected=false;try{Tuning::parse(args);}catch(const std::invalid_argument&){rejected=true;}assert(rejected);++refused;}
 for(auto values:std::vector<std::vector<size_t>>{{1,0,0},{3,0,0},{9,1,0},{4,1,2},{4,4,4}}){bool rejected=false;try{t.next(values[0],values[1],values[2]);}catch(const std::invalid_argument&){rejected=true;}assert(rejected);++refused;}
 std::cout<<"PASS tuning transitions and invalid descriptors="<<refused<<'\n';
}
