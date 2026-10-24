#include "selection.h"
#include <cassert>
#include <iostream>
#include <limits>
int main(){
 namespace mx=mlx::core;auto s=mx::default_stream(mx::Device(mx::Device::gpu));
 const float f=std::numeric_limits<float>::denorm_min();
 mx::array inputs({-0.f,+0.f,-f,-0.f, +0.f,-0.f,-f,-f, -2.f,-1.f,-1.f,-3.f, f,0.f,f,-f},{4,4},mx::float32);
 auto selected=device_total_argmax(inputs,s);mx::eval(selected);mx::synchronize(s);
 const uint32_t expected[]={1,0,1,0};
 for(int i=0;i<4;++i)assert(selected.data<uint32_t>()[i]==expected[i]);
 std::cout<<"PASS finite total-order argmax: signed zero, ties, subnormals, negative values\n";
}
