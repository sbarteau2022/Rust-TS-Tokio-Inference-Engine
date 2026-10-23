// Standalone native feasibility gate. This is not full-model qualification.
#include <mlx/mlx.h>
#include <algorithm>
#include <cmath>
#include <iostream>
#include <stdexcept>
namespace mx=mlx::core;
struct Error {double max=0,relative=0;};
Error compare(const mx::array& a,const mx::array& b) {
 if(a.shape()!=b.shape())throw std::runtime_error("comparison shape");
 mx::eval({a,b});mx::synchronize();double d=0,r=0,m=0;
 for(size_t i=0;i<a.size();++i){double x=a.data<float>()[i],y=b.data<float>()[i];if(!std::isfinite(x)||!std::isfinite(y))throw std::runtime_error("nonfinite");double e=x-y;d+=e*e;r+=y*y;m=std::max(m,std::abs(e));}
 Error e{m,std::sqrt(d/std::max(r,1e-30))};if(e.max>1e-3||e.relative>1e-5)throw std::runtime_error("numerical gate");return e;
}
mx::array signal(const mx::Shape& shape,float scale,float phase) {
 size_t size=1;for(auto d:shape)size*=d;
 return mx::reshape(mx::multiply(mx::sin(mx::add(mx::arange(int(size),mx::float32),mx::array(phase))),mx::array(scale)),shape);
}
int main(){try {
 mx::set_default_device(mx::Device::gpu);
 for(int B:{1,3})for(int T:{1,8}) {
  auto q=signal({B,T,16,128},.03f,.1f),k=signal({B,T,16,128},.03f,.7f),v=signal({B,T,32,128},.1f,.3f);
  auto g=mx::add(signal({B,T,32},.01f,.4f),mx::array(.95f)),beta=mx::add(signal({B,T,32},.02f,.5f),mx::array(.5f));
  auto parent=signal({B,32,128,128},.001f,.6f);mx::eval(parent);mx::synchronize();
  std::vector<float> saved(parent.data<float>(),parent.data<float>()+parent.size());
  auto state=parent;std::vector<mx::array> rows;
  for(int i=0;i<T;++i){auto qt=mx::repeat(mx::take(q,i,1),2,1),kt=mx::repeat(mx::take(k,i,1),2,1);
   state=mx::multiply(state,mx::expand_dims(mx::take(g,i,1),{2,3}));
   auto memory=mx::sum(mx::multiply(state,mx::expand_dims(kt,-2)),-1,false);
   auto delta=mx::multiply(mx::subtract(mx::take(v,i,1),memory),mx::expand_dims(mx::take(beta,i,1),-1));
   state=mx::add(state,mx::multiply(mx::expand_dims(kt,-2),mx::expand_dims(delta,-1)));
   rows.push_back(mx::sum(mx::multiply(state,mx::expand_dims(qt,-2)),-1,false));}
  auto expected=mx::stack(rows,1);
  auto first=mx::fast::gated_delta_update(q,k,v,g,beta,parent);
  auto repeat=mx::fast::gated_delta_update(q,k,v,g,beta,parent);
  auto output=compare(first[0],expected),next=compare(first[1],state);
  compare(first[0],repeat[0]);compare(first[1],repeat[1]);
  if(!std::equal(saved.begin(),saved.end(),parent.data<float>()))throw std::runtime_error("immutable parent changed");
  std::cout<<"B="<<B<<" T="<<T<<" output_max="<<output.max<<" output_nrmse="<<output.relative<<" state_max="<<next.max<<" state_nrmse="<<next.relative<<" parent_unchanged=true\n";
 }
 std::cout<<"PASS native fused delta component; no full-model or throughput claim\n";return 0;
 }catch(const std::exception& e){std::cerr<<e.what()<<"\n";return 1;}}
