// Invalid typed-IR wire descriptors must reject before reading a weight store or writing results.
#include "mlx_bridge.h"
#include <cassert>
#include <cstdint>
#include <vector>
int main(){
 rs_qwen_config c{};
 c.hidden=32;c.layers=4;c.vocab=64;c.q_heads=2;c.kv_heads=1;c.head_dim=16;
 c.key_heads=1;c.value_heads=2;c.key_dim=16;c.value_dim=16;c.conv=4;c.interval=4;
 c.experts=4;c.top=2;c.intermediate=32;c.shared=32;c.group=32;c.bits=4;
 c.context=512;c.chunk=8;c.rope_dims=4;c.activation_precision=1;c.epsilon=1e-6f;c.theta=10000.f;
 std::vector<float> state(2144,0.f),logits(64,0.f);uint32_t token=1;char error[1024]={};
 for(int i=0;i<1000;++i)for(int fault=0;fault<5;++fault){
  rs_qwen_layer layers[]={{0,0},{1,0},{2,0},{3,1}};size_t count=4;
  switch(fault){case 0:count=3;break;case 1:layers[0].index=1;break;case 2:layers[0].mixer=2;break;case 3:layers[0].mixer=1;break;default:count=SIZE_MAX;}
  auto result=rs_qwen_forward(reinterpret_cast<void*>(uintptr_t(1)),0,&c,layers,count,&token,1,0,nullptr,0,state.data(),state.size(),logits.data(),logits.size(),error,sizeof(error));
  assert(result==-1);for(float v:state)assert(v==0.f);for(float v:logits)assert(v==0.f);
 }
}
