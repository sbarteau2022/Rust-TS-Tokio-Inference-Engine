#pragma once
#include <mlx/mlx.h>
#include <cmath>
#include <functional>
#include <string>
#include <unordered_map>
#include <vector>
#include <stdexcept>
// Exact cached Koopah architecture; FP32 math lane, no serving process/transport.
namespace native_dspark {
namespace mx=mlx::core;
inline void expect(bool ok,const char* why){if(!ok)throw std::invalid_argument(why);}
struct Cache {size_t position=0;std::vector<mx::array> keys,values;};
struct Result {mx::array hidden,base_logits,confidence;std::vector<uint32_t> tokens;Cache context;};
class Draft {
 mx::Dtype dtype_;mx::Stream stream_;std::unordered_map<std::string,mx::array> weights_;std::optional<mx::array> fused_kv_;
 const mx::array& w(const std::string& key)const{return weights_.at(key);}
 mx::array linear(const mx::array& x,const std::string& key){return mx::matmul(x,mx::transpose(w(key+".weight"),{1,0},stream_),stream_);}
 mx::array norm(const mx::array& x,const std::string& key){return mx::fast::rms_norm(x,w(key+".weight"),1e-6f,stream_);}
 mx::array rope(const mx::array& x,int offset){return mx::fast::rope(x,128,false,10000000.f,1.f,offset,{},stream_);}
 void finite(const std::vector<mx::array>& arrays){std::vector<mx::array> checks;for(const auto& a:arrays)checks.push_back(mx::all(mx::isfinite(a,stream_),stream_));mx::eval(checks);mx::synchronize(stream_);for(const auto& a:checks)expect(a.item<bool>(),"DSpark nonfinite payload");}
 public:
 explicit Draft(bool gpu=true,bool fp32=true):dtype_(fp32?mx::float32:mx::bfloat16),stream_(mx::default_stream(mx::Device(gpu?mx::Device::gpu:mx::Device::cpu))){}
 void load(const std::string& path){
  expect(weights_.empty(),"DSpark already loaded");auto loaded=mx::load_safetensors(path,mx::Device::cpu).first;
  std::unordered_map<std::string,mx::Shape> shapes={{"embed_tokens.weight",{248320,2048}},{"lm_head.weight",{248320,2048}},{"fc.weight",{2048,16384}},{"hidden_norm.weight",{2048}},{"norm.weight",{2048}},{"markov_head.markov_w1.weight",{248320,256}},{"markov_head.markov_w2.weight",{248320,256}},{"confidence_head.proj.weight",{1,2304}},{"confidence_head.proj.bias",{1}}};
  for(int i=0;i<6;++i){auto p="layers."+std::to_string(i);for(auto name:{"input_layernorm","post_attention_layernorm"})shapes[p+"."+name+".weight"]={2048};for(auto name:{"q_norm","k_norm"})shapes[p+".self_attn."+name+".weight"]={128};for(auto name:{"q_proj","k_proj","v_proj","o_proj"})shapes[p+".self_attn."+name+".weight"]=std::string(name)=="q_proj"?mx::Shape{4096,2048}:std::string(name)=="o_proj"?mx::Shape{2048,4096}:mx::Shape{1024,2048};for(auto name:{"gate_proj","up_proj","down_proj"})shapes[p+".mlp."+name+".weight"]=std::string(name)=="down_proj"?mx::Shape{2048,6144}:mx::Shape{6144,2048};}
  expect(loaded.size()==shapes.size(),"DSpark weight inventory");
  for(const auto& [key,shape]:shapes){auto it=loaded.find(key);expect(it!=loaded.end()&&it->second.shape()==shape&&it->second.dtype()==mx::bfloat16,"DSpark tensor contract");}
  std::unordered_map<std::string,mx::array> prepared;
  for(auto& [key,value]:loaded){auto converted=mx::astype(value,dtype_,stream_);mx::eval(converted);finite({converted});prepared.emplace(key,std::move(converted));}
  weights_=std::move(prepared);
 }
 void validate_cache(const Cache& cache){
  expect(cache.position<=512,"DSpark cache position");expect(cache.keys.size()==cache.values.size()&&cache.keys.size()==(cache.position?6:0),"DSpark cache inventory");
  for(size_t i=0;i<cache.keys.size();++i)for(const auto* a:{&cache.keys[i],&cache.values[i]})expect(a->shape()==mx::Shape{1,8,int(cache.position),128}&&a->dtype()==dtype_,"DSpark cache geometry");
 }
 Cache ingest(const mx::array& taps,const Cache& previous){
  expect(!weights_.empty(),"DSpark weights unavailable");validate_cache(previous);
  expect(taps.ndim()==3&&taps.shape(0)==1&&taps.shape(2)==16384&&taps.dtype()==mx::float32,"DSpark target taps");
  int count=taps.shape(1);expect(count>=1&&size_t(count)<=512-previous.position,"DSpark context capacity");finite({taps});
  auto context=norm(linear(mx::astype(taps,dtype_,stream_),"fc"),"hidden_norm");
  if(!fused_kv_){std::vector<mx::array> matrices;for(int i=0;i<6;++i)for(auto name:{"k_proj","v_proj"})matrices.push_back(w("layers."+std::to_string(i)+".self_attn."+name+".weight"));fused_kv_=mx::concatenate(matrices,0,stream_);mx::eval(*fused_kv_);}
  auto projected=mx::split(mx::matmul(context,mx::transpose(*fused_kv_,{1,0},stream_),stream_),12,-1,stream_);
  Cache next;next.position=previous.position+count;
  for(int i=0;i<6;++i){auto a="layers."+std::to_string(i)+".self_attn";
   auto ck=rope(mx::transpose(norm(mx::reshape(projected[2*i],{1,count,8,128},stream_),a+".k_norm"),{0,2,1,3},stream_),int(previous.position));
   auto cv=mx::transpose(mx::reshape(projected[2*i+1],{1,count,8,128},stream_),{0,2,1,3},stream_);
   if(previous.position){ck=mx::concatenate({previous.keys[i],ck},2,stream_);cv=mx::concatenate({previous.values[i],cv},2,stream_);}next.keys.push_back(ck);next.values.push_back(cv);
  }
  std::vector<mx::array> arrays=next.keys;arrays.insert(arrays.end(),next.values.begin(),next.values.end());finite(arrays);return next;
 }
 Result propose(uint32_t anchor,const mx::array& taps,const Cache& previous,size_t width){
  expect(anchor<248320&&width>=2&&width<=8,"DSpark anchor/verification width");
  return propose_cached(anchor,ingest(taps,previous),width);
 }
 using Head=std::function<mx::array(const mx::array&)>;
 Result propose_cached(uint32_t anchor,const Cache& next,size_t width,const Head& head={}){
  expect(!weights_.empty(),"DSpark weights unavailable");validate_cache(next);
  expect(next.position>0&&anchor<248320&&width>=2&&width<=8,"DSpark cached proposal descriptor");
  int rows=int(width)-1;std::vector<uint32_t> inputs(rows,248077);inputs[0]=anchor;
  auto ids=mx::array(inputs.data(),{1,rows},mx::uint32);auto x=mx::take(w("embed_tokens.weight"),ids,0,stream_);
  for(int i=0;i<6;++i){auto p="layers."+std::to_string(i),a=p+".self_attn";auto h=norm(x,p+".input_layernorm");
   auto q=rope(mx::transpose(norm(mx::reshape(linear(h,a+".q_proj"),{1,rows,32,128},stream_),a+".q_norm"),{0,2,1,3},stream_),int(next.position));
   auto pk=rope(mx::transpose(norm(mx::reshape(linear(h,a+".k_proj"),{1,rows,8,128},stream_),a+".k_norm"),{0,2,1,3},stream_),int(next.position));
   auto pv=mx::transpose(mx::reshape(linear(h,a+".v_proj"),{1,rows,8,128},stream_),{0,2,1,3},stream_);
   auto y=mx::fast::scaled_dot_product_attention(q,mx::concatenate({next.keys[i],pk},2,stream_),mx::concatenate({next.values[i],pv},2,stream_),1.f/std::sqrt(128.f),"",{},{},false,stream_);
   x=mx::add(x,linear(mx::reshape(mx::transpose(y,{0,2,1,3},stream_),{1,rows,4096},stream_),a+".o_proj"),stream_);
   h=norm(x,p+".post_attention_layernorm");auto gate=linear(h,p+".mlp.gate_proj");x=mx::add(x,linear(mx::multiply(mx::multiply(gate,mx::sigmoid(gate,stream_),stream_),linear(h,p+".mlp.up_proj"),stream_),p+".mlp.down_proj"),stream_);
  }
  auto hidden=norm(x,"norm");auto base=head?head(hidden):linear(hidden,"lm_head");std::vector<mx::array> confidence;
  std::vector<mx::array> choices,score_rows;auto previous=mx::array(anchor,mx::uint32);
  for(int row=0;row<rows;++row){auto markov=mx::take(w("markov_head.markov_w1.weight"),previous,0,stream_);auto scores=mx::add(mx::take(base,row,1,stream_),linear(markov,"markov_head.markov_w2"),stream_);score_rows.push_back(scores);previous=mx::reshape(mx::argmax(scores,-1,false,stream_),{},stream_);choices.push_back(previous);
   auto features=mx::concatenate({mx::reshape(mx::take(hidden,row,1,stream_),{2048},stream_),markov},0,stream_);confidence.push_back(mx::sigmoid(mx::astype(mx::add(linear(features,"confidence_head.proj"),w("confidence_head.proj.bias"),stream_),mx::float32,stream_),stream_));}
  auto sampled_ids=mx::contiguous(mx::stack(choices,0,stream_),false,stream_);auto confidences=mx::reshape(mx::concatenate(confidence,0,stream_),{1,rows},stream_);std::vector<mx::array> all={hidden,base,confidences,sampled_ids};all.insert(all.end(),score_rows.begin(),score_rows.end());all.insert(all.end(),next.keys.begin(),next.keys.end());all.insert(all.end(),next.values.begin(),next.values.end());finite(all);mx::eval(all);mx::synchronize(stream_);std::vector<uint32_t> tokens(sampled_ids.data<uint32_t>(),sampled_ids.data<uint32_t>()+rows);return {hidden,base,confidences,std::move(tokens),next};
 }
};
}
