#include "dspark.h"
#include <iostream>
#include <fstream>
#include <iomanip>
#include <limits>
#include <cstring>
namespace mx=mlx::core;
using native_dspark::expect;
int main(int argc,char** argv){try{
 expect(argc==4||(argc==5&&std::string(argv[4])=="bf16"),"native_dspark_gate MODEL INPUTS EXPECTED [bf16]");native_dspark::Draft draft(true,argc==4);draft.load(argv[1]);
 auto inputs=mx::load_safetensors(argv[2],mx::Device::cpu).first,expected=mx::load_safetensors(argv[3],mx::Device::cpu).first;native_dspark::Cache cache;double maximum=0,relative=0;size_t comparisons=0;
 auto compare=[&](const mx::array& a,const mx::array& b){expect(a.shape()==b.shape(),"oracle shape");auto x=mx::contiguous(mx::astype(a,mx::float32)),y=mx::contiguous(mx::astype(b,mx::float32));mx::eval({x,y});double sq=0,energy=0,peak=0;for(size_t j=0;j<x.size();++j){double u=x.data<float>()[j],v=y.data<float>()[j];expect(std::isfinite(u)&&std::isfinite(v),"oracle nonfinite");sq+=(u-v)*(u-v);energy+=v*v;peak=std::max(peak,std::abs(u-v));}double norm=std::sqrt(sq/std::max(energy,1e-30));maximum=std::max(maximum,peak);relative=std::max(relative,norm);if(norm>1e-5||peak>1e-3){std::cerr<<"relative="<<norm<<" max="<<peak<<'\n';throw std::runtime_error("DSpark FP32 oracle mismatch");}++comparisons;};
 int widths[]={2,4,8,2};
 for(int i=0;i<4;++i){std::vector<std::vector<float>> parents;for(const auto* arrays:{&cache.keys,&cache.values})for(auto a:*arrays){auto packed=mx::contiguous(mx::astype(a,mx::float32));mx::eval(packed);parents.emplace_back(packed.data<float>(),packed.data<float>()+packed.size());}
 std::string tag=std::to_string(i);auto result=draft.propose(100+i,inputs.at("taps"+tag),cache,widths[i]);compare(result.hidden,expected.at("hidden"+tag));compare(result.base_logits,expected.at("logits"+tag));compare(result.confidence,expected.at("confidence"+tag));auto ids=mx::array(result.tokens.data(),{1,int(result.tokens.size())},mx::uint32);auto same=mx::all(mx::equal(ids,expected.at("tokens"+tag)));expect(same.item<bool>(),"DSpark greedy token mismatch");for(int layer=0;layer<6;++layer){compare(result.context.keys[layer],expected.at("key"+tag+"_"+std::to_string(layer)));compare(result.context.values[layer],expected.at("value"+tag+"_"+std::to_string(layer)));}size_t parent_index=0;for(const auto* arrays:{&cache.keys,&cache.values})for(auto a:*arrays){auto packed=mx::contiguous(mx::astype(a,mx::float32));mx::eval(packed);expect(std::memcmp(packed.data<float>(),parents[parent_index].data(),packed.nbytes())==0,"parent KV mutation");++parent_index;}expect(cache.position==(i==0?0:i==1?3:i==2?5:6),"parent cache mutation");cache=std::move(result.context);}
 size_t rejected=0;auto refuses=[&](auto action){bool threw=false;try{action();}catch(const std::exception&){threw=true;}expect(threw,"invalid DSpark descriptor accepted");++rejected;};auto taps=inputs.at("taps0");
 for(size_t width:{size_t(0),size_t(1),size_t(9)})refuses([&]{draft.propose(1,taps,cache,width);});
 refuses([&]{draft.propose(248320,taps,cache,4);});refuses([&]{draft.propose(1,mx::zeros({1,1,2048}),cache,4);});
 refuses([&]{auto bad=cache;bad.position=513;draft.propose(1,taps,bad,4);});refuses([&]{auto bad=cache;bad.keys.pop_back();draft.propose(1,taps,bad,4);});
 refuses([&]{draft.propose(1,mx::full({1,1,16384},std::numeric_limits<float>::quiet_NaN()),native_dspark::Cache{},4);});
 std::cout<<std::setprecision(17)<<"{\"pass\":true,\"comparisons\":"<<comparisons<<",\"max_relative_l2\":"<<relative<<",\"max_absolute\":"<<maximum<<",\"greedy_ids_exact\":true,\"invalid_descriptors\":"<<rejected<<",\"immutable_parents\":true,\"context_boundary\":"<<cache.position<<",\"own_head_component_only\":true,\"target_integration\":false}\n";
 return 0;}catch(const std::exception& e){std::cerr<<"FAIL "<<e.what()<<'\n';return 1;}}
