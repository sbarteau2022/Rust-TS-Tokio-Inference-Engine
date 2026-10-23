#include "state_table.h"
#include <cassert>
#include <iostream>
#include <string>
static size_t checks=0;
template<class F> void rejects(F fn){bool rejected=false;try{fn();}catch(const std::exception&){rejected=true;}assert(rejected);++checks;}
int main(){
 rejects([]{StateTable<int> t(0,1,4);});rejects([]{StateTable<int> t(1,0,4);});rejects([]{StateTable<int> t(1,65,4);});
 StateTable<std::string> t(1,2,8),foreign(2,2,8);
 auto a=t.reserve(1,4);rejects([&]{t.get(a);});t.publish(a,"prefix");
 auto b=t.reserve(2,4);t.publish(b,"branch");
 rejects([&]{t.reserve(3,1);});rejects([&]{foreign.get(a);});rejects([&]{t.publish(a,"again");});
 for(int field=0;field<5;++field){auto bad=a;switch(field){case 0:++bad.owner;break;case 1:bad.slot=UINT64_MAX;break;case 2:++bad.generation;break;case 3:++bad.position;break;case 4:++bad.bytes;break;}rejects([&]{t.get(bad);});rejects([&]{t.release(bad);});}
 t.release(a);rejects([&]{t.release(a);});auto replacement=t.reserve(1,4);assert(replacement.slot==a.slot&&replacement.generation!=a.generation);t.publish(replacement,"new");rejects([&]{t.get(a);});
 assert(t.get(b)=="branch");t.release(b);t.release(replacement);assert(!t.bytes()&&!t.residents());
 for(size_t n=0;n<32000;++n){auto id=t.reserve(n+1,8);rejects([&]{t.reserve(1,1);});t.rollback(id);rejects([&]{t.get(id);});assert(!t.bytes());checks+=1;}
 rejects([&]{t.reserve(0,4);});rejects([&]{t.reserve(1,0);});rejects([&]{t.reserve(1,UINT64_MAX);});
 auto id=t.reserve(1,8);t.publish(id,"live");t.clear();assert(!t.bytes()&&!t.residents());rejects([&]{t.get(id);});
 StateTable<int,2> exhausted(3,1,4);for(int n=0;n<2;++n){auto x=exhausted.reserve(1,4);exhausted.publish(x,n);exhausted.release(x);}rejects([&]{exhausted.reserve(1,4);});
 std::cout<<"PASS state table checks="<<checks<<" ASan/UBSan\n";
}
