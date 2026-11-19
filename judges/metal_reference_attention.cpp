// Independent FP64 oracle, causal isolation, and descriptor adversarial checks.
#include "../native/metal_reference_attention.h"
#include "../native/attention_reference.h"
#include <iostream>
#include <random>
#include <cassert>
#include <limits>
namespace mx=mlx::core;
int main(){
 auto s=mx::default_stream(mx::Device(mx::Device::gpu));
 std::mt19937 rng(20261024);std::uniform_real_distribution<float> dist(-0.5f,0.5f);
 size_t comparisons=0,rejections=0;
 for(int batch:{1,3})for(int t:{1,2,4,8})for(int offset:{0,17,504}){
  if(offset+t>512)continue;
  const int heads=16,kv=2,dim=256,length=offset+t;
  std::vector<float> q(batch*heads*t*dim),k(batch*kv*length*dim),v(k.size());
  for(auto* a:{&q,&k,&v})for(auto& x:*a)x=dist(rng);
  auto qa=mx::array(q.data(),{batch,heads,t,dim});auto ka=mx::array(k.data(),{batch,kv,length,dim});auto va=mx::array(v.data(),{batch,kv,length,dim});
  auto result=mx::contiguous(metal_reference_attention(qa,ka,va,offset,s),false,s);mx::eval(result);mx::synchronize(s);
  for(int b=0;b<batch;++b){
   std::vector<float> qb(q.begin()+b*heads*t*dim,q.begin()+(b+1)*heads*t*dim),kb(k.begin()+b*kv*length*dim,k.begin()+(b+1)*kv*length*dim),vb(v.begin()+b*kv*length*dim,v.begin()+(b+1)*kv*length*dim);
   auto reference=attention_reference(qb,kb,vb,heads,kv,t,offset,dim);
   double square=0,energy=0,maximum=0;
   for(size_t i=0;i<reference.size();++i){double x=result.data<float>()[b*reference.size()+i],d=x-reference[i];assert(std::isfinite(x));square+=d*d;energy+=reference[i]*reference[i];maximum=std::max(maximum,std::abs(d));}
   assert(std::sqrt(square/std::max(energy,1e-30))<=1e-5&&maximum<=1e-4);++comparisons;
  }
 }
 auto q=mx::zeros({1,16,2,256},mx::float32,s),k=mx::zeros({1,2,2,256},mx::float32,s);
 for(int bad_offset:{-1,1,512,513,std::numeric_limits<int>::min(),std::numeric_limits<int>::max()}){try{(void)metal_reference_attention(q,k,k,bad_offset,s);assert(false);}catch(const std::invalid_argument&){++rejections;}}
 for(auto bad:{mx::zeros({1,2,2,128},mx::float32,s),mx::zeros({1,2,2,256},mx::bfloat16,s),mx::zeros({2,2,2,256},mx::float32,s)}){
  try{(void)metal_reference_attention(q,bad,bad,0,s);assert(false);}catch(const std::invalid_argument&){++rejections;}
 }
 // First row cannot see second-row values; the second row sees both.
 auto values=mx::concatenate({mx::ones({1,2,1,256},mx::float32,s),mx::full({1,2,1,256},7.f,mx::float32,s)},2,s);
 auto out=mx::contiguous(metal_reference_attention(q,k,values,0,s),false,s);mx::eval(out);mx::synchronize(s);
 for(int h=0;h<16;++h)for(int d=0;d<256;++d){assert(out.data<float>()[(h*2)*256+d]==1.f);assert(out.data<float>()[(h*2+1)*256+d]==4.f);}
 std::cout<<"{\"pass\":true,\"oracle_rows\":"<<comparisons<<",\"invalid_descriptors\":"<<rejections<<",\"causal_isolation\":true}\n";
}
