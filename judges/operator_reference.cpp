#include "operator_reference.h"
#include <cassert>
#include <functional>
#include <iostream>
int main(){
 int checks=0;auto near=[&](double a,double b){assert(std::abs(a-b)<1e-11*(1+std::abs(b)));++checks;};
 auto rms=scalar_reference::rms({3,4},{2,.5f},2,1);near(rms[0],6/std::sqrt(13.5));near(rms[1],2/std::sqrt(13.5));
 auto silu=scalar_reference::silu({-1000,0,1000});near(silu[0],0);near(silu[1],0);near(silu[2],1000);
 auto rope=scalar_reference::rope({1,2,9,10},1,1,4,2,1,10000);near(rope[0],std::cos(1.)-2*std::sin(1.));near(rope[1],std::sin(1.)+2*std::cos(1.));near(rope[2],9);near(rope[3],10);
 auto four=scalar_reference::rope({1,2,3,4},1,1,4,4,3,10000);near(four[0],std::cos(3.)-3*std::sin(3.));near(four[1],2*std::cos(.03)-4*std::sin(.03));near(four[2],std::sin(3.)+3*std::cos(3.));near(four[3],2*std::sin(.03)+4*std::cos(.03));
 // Scalar recurrence with two value heads sharing one key head; decay=beta=1/2.
 std::vector<float> qkv={1,1,2,3},history(4,0),cw={0,1,0,1,0,1,0,1};
 auto delta=scalar_reference::delta(qkv,history,cw,{0,0},{0,0},{0,0},{0,0},{2,4},1,1,2,1,1,2);
 double q=1/(1+std::exp(-1.));q/=std::sqrt(q*q+1e-6);
 for(int h=0;h<2;++h){double v=(h+2.)/(1+std::exp(-(h+2.)));double old=(h+1.);double expected=old+.5*(v-old*q)*q;near(delta.state[h],expected);near(delta.output[h],expected*q);}
 for(size_t i=0;i<qkv.size();++i)near(delta.history[i],qkv[i]);
 // Nonzero causal history and both convolution taps influence each channel.
 auto conv=scalar_reference::delta({1,1,2,3},{4,5,6,7},{.25f,.75f,.25f,.75f,.25f,.75f,.25f,.75f},{0,0},{0,0},{0,0},{0,0},{0,0},1,1,2,1,1,2);
 for(int i=0;i<4;++i){double x=.25*(i+4)+.75*qkv[i];near(conv.activated[i],x/(1+std::exp(-x)));}
 auto reject=[&](const std::function<void()>& f){bool failed=false;try{f();}catch(const std::invalid_argument&){failed=true;}assert(failed);++checks;};
 reject([]{scalar_reference::rms({1},{},0,1e-6);});reject([]{scalar_reference::rms({1},{1,2},1,1e-6);});reject([]{scalar_reference::rms({NAN},{},1,1e-6);});reject([]{scalar_reference::rms({1},{},1,0);});
 reject([]{scalar_reference::rope({1,2},1,1,2,1,0,10000);});reject([]{scalar_reference::rope({1,2},1,1,2,2,512,10000);});reject([]{scalar_reference::silu({INFINITY});});reject([]{scalar_reference::rope({1,2},1,1,2,2,INT32_MAX,10000);});
 reject([&]{scalar_reference::delta(qkv,history,cw,{0,0},{0,0},{0,0},{0,0},{2,4},1,0,2,1,1,2);});
 reject([&]{scalar_reference::delta(qkv,history,cw,{0,0},{0,0},{0,0},{0,0},{2},1,1,2,1,1,2);});
 auto routing=scalar_reference::routing({0,1,2},{2,1},1,3,2);near(routing.weights[0],std::exp(1.)/(std::exp(1.)+1));near(routing.weights[1],1/(std::exp(1.)+1));
 auto tied=scalar_reference::routing({1,1,1},{2,0},1,3,2);near(tied.weights[0],.5);near(tied.weights[1],.5);
 reject([]{scalar_reference::routing({0,1,2},{1,0},1,3,2);});reject([]{scalar_reference::routing({0,1,2},{2,2},1,3,2);});reject([]{scalar_reference::routing({0,1,2},{2,3},1,3,2);});reject([]{scalar_reference::routing({0,1,2},{2,-1},1,3,2);});
 std::cout<<"PASS scalar operator oracle checks="<<checks<<" RMS SiLU RoPE causal-convolution GDN shared-head state analytic/malformed sanitizer=address,undefined\n";
}
