// Complete text-only Qwen3.5/3.6 hybrid reference, invoked only by its Rust executor.
#include <cmath>
#include <cstdio>
#include "attention_reference.h"
#include "projection_reference.h"
#include "operator_reference.h"
namespace {
struct Qwen {
 Weights& weights;rs_qwen_config c;mx::Stream s;bool dense=false,shift=false,reference=false,operators=false,full_reference=false;std::vector<int32_t> head_channels;
 Qwen(Weights& w,rs_qwen_config cfg,bool gpu):weights(w),c(cfg),s(mx::default_stream(mx::Device(gpu?mx::Device::gpu:mx::Device::cpu))){
  for(const auto& [key,value]:w.arrays)if(key.find("mtp.")!=std::string::npos || (key.ends_with("conv1d.weight")&&value.shape(-1)!=1))shift=true;
 }
 const mx::array& w(const std::string& key){return weights.arrays.at(key);}
 std::pair<int,int> quant(const std::string& p){auto it=weights.quant.find(p);return it==weights.quant.end()?std::pair<int,int>{c.group,c.bits}:it->second;}
#include "operator_probe.inc"
 mx::array silu(const mx::array& x){auto y=mx::multiply(x,mx::sigmoid(x,s),s);if(operators)check_operator("silu",scalar_reference::silu(cpu_values(x)),y);return y;}
 mx::array norm(const mx::array& x,const std::string& key,bool offset=true){auto weight=mx::astype(w(key),x.dtype(),s);if(shift&&offset)weight=mx::add(weight,mx::array(1.f),s);auto y=mx::fast::rms_norm(x,weight,c.epsilon,s);if(operators)check_operator(key+"/rms",scalar_reference::rms(cpu_values(x),cpu_values(weight),x.shape(-1),c.epsilon),y);return y;}
#include "projection_probe.inc"
 mx::array linear(const mx::array& x,const std::string& p){if(dense&&w(p+".weight").dtype()==mx::bfloat16){if(w(p+".weight").ndim()!=2)throw std::invalid_argument("MTP dense projection rank");auto matrix=mx::transpose(mx::astype(w(p+".weight"),mx::float32,s),{1,0},s);auto flat=mx::reshape(x,{-1,x.shape(-1)},s);if(flat.shape(0)>8)throw std::invalid_argument("MTP dense row cap");std::vector<mx::array> rows;for(int row=0;row<flat.shape(0);++row)rows.push_back(mx::matmul(mx::take(flat,row,0,s),matrix,s));auto shape=x.shape();shape.back()=w(p+".weight").shape(0);auto y=mx::reshape(mx::stack(rows,0,s),shape,s);if(reference)check_dense(x,y,w(p+".weight"),p);return y;}auto [g,b]=quant(p);auto y=mx::quantized_matmul(x,w(p+".weight"),w(p+".scales"),w(p+".biases"),true,g,b,"affine",s);if(reference)check_projection(x,y,w(p+".weight"),w(p+".scales"),w(p+".biases"),g,b,p);return y;}
 mx::array expert(const mx::array& x,const mx::array& idx,const std::string& p){
  auto [g,b]=quant(p);auto y=mx::gather_qmm(x,w(p+".weight"),w(p+".scales"),w(p+".biases"),{},idx,true,g,b,"affine",{},false,s);
  if(reference){
   auto chosen=mx::contiguous(mx::astype(idx,mx::int32,s),false,s);mx::eval(chosen);mx::synchronize(s);
   std::vector<int> probe_rows={0};if(full_reference){for(int row=1;row<idx.shape(0);++row)probe_rows.push_back(row);}else if(idx.shape(0)>1)probe_rows.push_back(idx.shape(0)-1);
   std::vector<int> probe_slots={0};if(full_reference){for(int slot=1;slot<idx.shape(-1);++slot)probe_slots.push_back(slot);}else if(idx.shape(-1)>1)probe_slots.push_back(idx.shape(-1)-1);
   for(int row:probe_rows)for(int slot:probe_slots){
    int expert_id=chosen.data<int32_t>()[row*idx.shape(-1)+slot],source=x.shape(1)==1?0:slot;
    auto input=mx::slice(x,{row,source,0,0},{row+1,source+1,1,x.shape(-1)},s);
    auto output=mx::slice(y,{row,slot,0,0},{row+1,slot+1,1,y.shape(-1)},s);
    check_projection(input,output,mx::take(w(p+".weight"),expert_id,0,s),mx::take(w(p+".scales"),expert_id,0,s),mx::take(w(p+".biases"),expert_id,0,s),g,b,p+"/row="+std::to_string(row)+"/expert="+std::to_string(expert_id)+"/slot="+std::to_string(slot));
   }
  }
  return y;
 }
 mx::array moe(const mx::array& x,const std::string& p){
  auto flat=mx::reshape(x,{-1,c.hidden},s);auto router=linear(flat,p+".gate");auto gates=mx::softmax(router,-1,true,s);
  auto idx=mx::argpartition(gates,-c.top,-1,s);idx=mx::slice(idx,{0,c.experts-c.top},{flat.shape(0),c.experts},s);
  auto scores=mx::take_along_axis(gates,idx,-1,s);scores=mx::divide(scores,mx::sum(scores,-1,true,s),s);
  if(operators)check_routing(router,gates,idx,scores,p);
  auto expanded=mx::expand_dims(flat,{-2,-3},s);
  auto up=expert(expanded,idx,p+".switch_mlp.up_proj");auto gate=expert(expanded,idx,p+".switch_mlp.gate_proj");
  auto selected=expert(mx::multiply(silu(gate),up,s),idx,p+".switch_mlp.down_proj");
  auto y=mx::sum(mx::multiply(mx::squeeze(selected,-2,s),mx::expand_dims(scores,-1,s),s),1,false,s);
  auto shared=linear(mx::multiply(silu(linear(flat,p+".shared_expert.gate_proj")),linear(flat,p+".shared_expert.up_proj"),s),p+".shared_expert.down_proj");
  auto shared_gate=linear(flat,p+".shared_expert_gate");auto merged=mx::add(y,mx::multiply(shared,mx::sigmoid(shared_gate,s),s),s);if(operators)check_mixture(selected,scores,y,shared,shared_gate,merged,p);
  return mx::reshape(merged,x.shape(),s);
 }
 mx::array attention(const mx::array& x,const std::string& p,int offset,mx::array old_k,mx::array old_v,std::vector<mx::array>& next,bool diagnose=false){
  int batch=x.shape(0),t=x.shape(1);auto qg=mx::split(mx::reshape(linear(x,p+".q_proj"),{batch,t,c.q_heads,2*c.head_dim},s),2,-1,s);
  auto q=mx::transpose(norm(qg[0],p+".q_norm.weight"),{0,2,1,3},s);
  auto k=mx::transpose(norm(mx::reshape(linear(x,p+".k_proj"),{batch,t,c.kv_heads,c.head_dim},s),p+".k_norm.weight"),{0,2,1,3},s);
  auto v=mx::transpose(mx::reshape(linear(x,p+".v_proj"),{batch,t,c.kv_heads,c.head_dim},s),{0,2,1,3},s);
  q=rope_checked(q,offset,p+"/q-rope");k=rope_checked(k,offset,p+"/k-rope");
  if(offset){k=mx::concatenate({old_k,k},2,s);v=mx::concatenate({old_v,v},2,s);}
  next.push_back(k);next.push_back(v);
  std::vector<mx::array> rows;
  for(int row=0;row<t;++row){
   auto qr=mx::slice(q,{0,0,row,0},{batch,c.q_heads,row+1,c.head_dim},s);
   auto kr=mx::slice(k,{0,0,0,0},{batch,c.kv_heads,offset+row+1,c.head_dim},s);
   auto vr=mx::slice(v,{0,0,0,0},{batch,c.kv_heads,offset+row+1,c.head_dim},s);
   auto grouped=mx::reshape(mx::astype(qr,mx::float32,s),{batch,c.kv_heads,c.q_heads/c.kv_heads,1,c.head_dim},s);
   auto keys=mx::expand_dims(mx::astype(kr,mx::float32,s),2,s),values=mx::expand_dims(mx::astype(vr,mx::float32,s),2,s);
   auto scores=mx::multiply(mx::sum(mx::multiply(grouped,keys,s),-1,false,s),mx::array(1.f/std::sqrt(float(c.head_dim))),s);
   auto probabilities=mx::expand_dims(mx::softmax(scores,-1,true,s),-1,s);
   rows.push_back(mx::reshape(mx::sum(mx::multiply(probabilities,values,s),-2,false,s),{batch,c.q_heads,1,c.head_dim},s));
  }
  auto y=mx::concatenate(rows,2,s);
  if(diagnose){
   std::vector<mx::array> probe;for(auto a:{q,k,v,y})probe.push_back(mx::contiguous(mx::astype(a,mx::float32,s),false,s));
   mx::eval(probe);mx::synchronize(s);
   const float *qp=probe[0].data<float>(),*kp=probe[1].data<float>(),*vp=probe[2].data<float>(),*yp=probe[3].data<float>();
   auto expected=attention_reference({qp,qp+probe[0].size()},{kp,kp+probe[1].size()},{vp,vp+probe[2].size()},c.q_heads,c.kv_heads,t,offset,c.head_dim);
   double square=0,reference=0,maximum=0;
   for(size_t i=0;i<expected.size();++i){double delta=yp[i]-expected[i];square+=delta*delta;reference+=expected[i]*expected[i];maximum=std::max(maximum,std::abs(delta));}
   std::fprintf(stderr,"ATTENTION-ORACLE layer=%s offset=%d tokens=%d nrmse=%.9g max_abs=%.9g\n",p.c_str(),offset,t,std::sqrt(square/std::max(reference,1e-30)),maximum);
   if(!std::isfinite(square)||std::sqrt(square/std::max(reference,1e-30))>1e-5)throw std::runtime_error("attention differs from scalar FP64 oracle");
  }
  y=mx::reshape(mx::transpose(y,{0,2,1,3},s),{batch,t,c.q_heads*c.head_dim},s);
  auto gate=mx::reshape(qg[1],{batch,t,c.q_heads*c.head_dim},s);auto gated=mx::multiply(y,mx::sigmoid(gate,s),s);if(operators)check_gate(y,gate,gated,p+"/output-gate");return linear(gated,p+".o_proj");
 }
 mx::array recurrent(const mx::array& x,const std::string& p,mx::array conv,mx::array state,std::vector<mx::array>& next){
  int batch=x.shape(0),t=x.shape(1),kd=c.key_heads*c.key_dim,vd=c.value_heads*c.value_dim,cd=2*kd+vd;
  auto qkv=linear(x,p+".in_proj_qkv"),z=mx::reshape(linear(x,p+".in_proj_z"),{batch,t,c.value_heads,c.value_dim},s);
  auto a=linear(x,p+".in_proj_a"),b=linear(x,p+".in_proj_b");
  auto input=mx::concatenate({conv,qkv},1,s);
  next.push_back(mx::contiguous(mx::slice(input,{0,t,0},{batch,t+c.conv-1,cd},s),false,s));
  auto cw=mx::astype(w(p+".conv1d.weight"),x.dtype(),s);if(cw.shape(-1)!=1)cw=mx::transpose(cw,{0,2,1},s);
  auto activated=silu(mx::conv1d(input,cw,1,0,1,cd,s));auto parts=mx::split(activated,{kd,2*kd},-1,s);
  auto q=mx::reshape(parts[0],{batch,t,c.key_heads,c.key_dim},s),k=mx::reshape(parts[1],{batch,t,c.key_heads,c.key_dim},s),v=mx::reshape(parts[2],{batch,t,c.value_heads,c.value_dim},s);
  q=mx::multiply(mx::fast::rms_norm(q,{},1e-6,s),mx::array(1.f/c.key_dim),s);
  k=mx::multiply(mx::fast::rms_norm(k,{},1e-6,s),mx::array(1.f/std::sqrt(float(c.key_dim))),s);
  auto beta=mx::sigmoid(b,s);auto softplus=mx::logaddexp(mx::add(a,w(p+".dt_bias"),s),mx::array(0.f),s);
  auto decay=mx::exp(mx::negative(mx::multiply(mx::exp(mx::astype(w(p+".A_log"),mx::float32,s),s),softplus,s),s),s);
  auto initial_state=state;
  auto readout=[&](){
  if(c.fused_recurrent){auto result=mx::fast::gated_delta_update(q,k,v,decay,beta,state,{},s);state=result[1];return result[0];}
  std::vector<mx::array> out;
  for(int i=0;i<t;++i){
   auto qt=mx::astype(mx::repeat(mx::take(q,i,1,s),c.value_heads/c.key_heads,1,s),mx::float32,s);
   auto kt=mx::astype(mx::repeat(mx::take(k,i,1,s),c.value_heads/c.key_heads,1,s),mx::float32,s);
   state=mx::multiply(state,mx::expand_dims(mx::take(decay,i,1,s),{2,3},s),s);
   auto memory=mx::sum(mx::multiply(state,mx::expand_dims(kt,-2,s),s),-1,false,s);
   auto delta=mx::multiply(mx::subtract(mx::astype(mx::take(v,i,1,s),mx::float32,s),memory,s),mx::astype(mx::expand_dims(mx::take(beta,i,1,s),-1,s),mx::float32,s),s);
   state=mx::add(state,mx::multiply(mx::expand_dims(kt,-2,s),mx::expand_dims(delta,-1,s),s),s);
   out.push_back(mx::astype(mx::sum(mx::multiply(state,mx::expand_dims(qt,-2,s),s),-1,false,s),x.dtype(),s));
  }
  return mx::stack(out,1,s);
  }();if(operators)check_delta(qkv,conv,cw,a,b,initial_state,activated,readout,state,next[0],p);
  next.push_back(state);auto y=norm(readout,p+".norm.weight",false);
  auto ungated=y;y=mx::astype(mx::multiply(mx::astype(y,mx::float32,s),silu(mx::astype(z,mx::float32,s)),s),x.dtype(),s);if(operators)check_gate(ungated,z,y,p+"/output-gate",true);
  return linear(mx::reshape(y,{batch,t,vd},s),p+".out_proj");
 }
};
size_t qwen_state_count(const rs_qwen_config& c,size_t offset){
 size_t full=c.layers/c.interval,linear=c.layers-full;
 return linear*(size_t(c.conv-1)*(2*c.key_heads*c.key_dim+c.value_heads*c.value_dim)+size_t(c.value_heads)*c.value_dim*c.key_dim)+full*2*c.kv_heads*offset*c.head_dim;
}
}
extern "C" int rs_qwen_quant(void* store,const char* module,int group,int bits,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!store||!module||!(group==32||group==64||group==128)||!(bits==2||bits==3||bits==4||bits==5||bits==6||bits==8))throw std::invalid_argument("invalid model quantization");
 static_cast<Weights*>(store)->quant[std::string(module)]={group,bits};
});}
static int qwen_forward_core(void* store,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t layer_count,const uint32_t* tokens,size_t count,size_t offset,const float* old_state,size_t old_count,float* new_state,size_t new_count,float* logits,size_t logits_count,int projection_checks,const uint32_t* projection_outputs,size_t projection_output_count,float* trace,size_t trace_count,char* error,size_t capacity,const std::vector<mx::array>* device_old=nullptr,std::vector<mx::array>* device_next=nullptr,bool all_logits=false,std::optional<mx::array>* hidden=nullptr){return checked(error,capacity,[&]{
 if(!store||!config||!layers||!tokens||(!new_state&&!device_next)||!logits)throw std::invalid_argument("invalid model descriptor");auto c=*config;
 if(c.hidden<=0||c.hidden>8192||c.layers<=0||c.layers>64||c.vocab<=0||c.vocab>300000||c.interval<=0||c.layers%c.interval||c.context<=0||c.context>512||c.chunk<=0||c.chunk>8||count==0||count>size_t(c.chunk)||offset>size_t(c.context)||offset+count>size_t(c.context)||logits_count!=size_t(c.vocab)*(all_logits?count:1))throw std::invalid_argument("model geometry/capacity");
 if(c.layer_group!=1&&c.layer_group!=4&&c.layer_group!=10&&c.layer_group!=40)throw std::invalid_argument("model layer group");
 if(c.fused_recurrent<0||c.fused_recurrent>1)throw std::invalid_argument("recurrent kernel mode");
 // Snapshot/trace controls use the original recurrence even in a tuned session.
 if(!device_next||trace_count||projection_checks)c.fused_recurrent=0;
 if(layer_count!=size_t(c.layers))throw std::invalid_argument("IR layer count");
 for(size_t i=0;i<layer_count;++i)if(layers[i].index!=i||layers[i].mixer>1||layers[i].mixer!=uint32_t((i+1)%c.interval==0))throw std::invalid_argument("IR layer program");
 for(auto d:{c.q_heads,c.kv_heads,c.head_dim,c.key_heads,c.value_heads,c.key_dim,c.value_dim,c.conv,c.experts,c.top,c.intermediate,c.shared})if(d<=0||d>8192)throw std::invalid_argument("model dimension bounds");
 if(c.activation_precision<0||c.activation_precision>1||c.q_heads%c.kv_heads||c.value_heads%c.key_heads||c.conv<2||c.top>c.experts||c.rope_dims<=0||c.rope_dims>c.head_dim||c.rope_dims%2||!std::isfinite(c.epsilon)||c.epsilon<=0||!std::isfinite(c.theta)||c.theta<=1)throw std::invalid_argument("model geometry mismatch");
 if(new_count!=qwen_state_count(c,offset+count)||(offset?old_count!=qwen_state_count(c,offset)||(!old_state&&!device_old):old_count!=0))throw std::invalid_argument("model state length");
 if(new_count>32*1024*1024)throw std::invalid_argument("model state byte cap");
 for(size_t i=0;i<count;++i)if(tokens[i]>=uint32_t(c.vocab))throw std::invalid_argument("token outside vocabulary");
 if(trace_count && (!trace||trace_count!=size_t(c.layers)*5*count*c.hidden||trace_count>64*5*8*8192))throw std::invalid_argument("trace capacity");
 if(projection_checks<0||projection_checks>7||((projection_checks&4)&&((projection_checks&3)!=3))||(projection_checks&&!trace_count))throw std::invalid_argument("projection diagnostic mode");
 if(projection_output_count>8||(projection_output_count&&(!projection_outputs||!(projection_checks&1))))throw std::invalid_argument("projection output capacity");
 for(size_t i=0;i<projection_output_count;++i){if(projection_outputs[i]>=uint32_t(c.vocab))throw std::invalid_argument("projection output vocabulary");for(size_t j=0;j<i;++j)if(projection_outputs[i]==projection_outputs[j])throw std::invalid_argument("duplicate projection output");}
 Qwen m(*static_cast<Weights*>(store),c,gpu!=0);m.reference=(projection_checks&1)!=0;m.operators=(projection_checks&2)!=0;m.full_reference=(projection_checks&4)!=0;for(size_t i=0;i<projection_output_count;++i)m.head_channels.push_back(int32_t(projection_outputs[i]));auto s=m.s;
 try{
 auto ids=mx::array(tokens,{int(count)},mx::uint32);std::string embed="language_model.model.embed_tokens";auto [group,bits]=m.quant(embed);
 auto embed_scales=mx::take(m.w(embed+".scales"),ids,0,s),embed_biases=mx::take(m.w(embed+".biases"),ids,0,s);
 if(c.activation_precision){embed_scales=mx::astype(embed_scales,mx::float32,s);embed_biases=mx::astype(embed_biases,mx::float32,s);}
 auto x=mx::reshape(mx::dequantize(mx::take(m.w(embed+".weight"),ids,0,s),embed_scales,embed_biases,group,bits,"affine",{},c.activation_precision?std::optional<mx::Dtype>{mx::float32}:std::nullopt,s),{1,int(count),c.hidden},s);
 if(m.operators)m.check_embedding(ids,x,embed,group,bits);
 size_t read=0,write=0,array_read=0;std::vector<mx::array> finite_checks,pending;
 // Diagnostics/snapshot controls keep the original completion boundaries.
 const int layer_group=(device_next&&!trace_count&&!projection_checks)?c.layer_group:1;
 auto state=[&](mx::Shape shape,mx::Dtype dtype){size_t n=1;for(auto d:shape)n*=d;auto a=mx::zeros(shape,dtype,s);if(offset){
   if(device_old){if(array_read>=device_old->size()||(*device_old)[array_read].shape()!=shape)throw std::invalid_argument("device state shape");a=mx::astype((*device_old)[array_read++],dtype,s);}
   else a=mx::astype(mx::array(old_state+read,shape,mx::float32),dtype,s);read+=n;
  }return a;};
 for(int layer=0;layer<c.layers;++layer){
  std::string prefix="language_model.model.layers."+std::to_string(layer);std::vector<mx::array> next;
  auto h=m.norm(x,prefix+".input_layernorm.weight");mx::array r=h;
  if(layers[layer].mixer==0){int cd=2*c.key_heads*c.key_dim+c.value_heads*c.value_dim;auto conv=state({1,c.conv-1,cd},x.dtype());auto recurrent=state({1,c.value_heads,c.value_dim,c.key_dim},mx::float32);r=m.recurrent(h,prefix+".linear_attn",conv,recurrent,next);}
  else{auto k=state({1,c.kv_heads,int(offset),c.head_dim},x.dtype());auto v=state({1,c.kv_heads,int(offset),c.head_dim},x.dtype());r=m.attention(h,prefix+".self_attn",int(offset),k,v,next,trace_count!=0);}
  auto residual=mx::add(x,r,s);if(m.operators)m.check_add(x,r,residual,prefix+"/attention-residual");auto mlp=m.moe(m.norm(residual,prefix+".post_attention_layernorm.weight"),prefix+".mlp");x=mx::add(residual,mlp,s);if(m.operators)m.check_add(residual,mlp,x,prefix+"/mlp-residual");
  std::vector<mx::array> evaluated={x};for(auto& a:next)evaluated.push_back(mx::contiguous(mx::astype(a,mx::float32,s),false,s));
  size_t state_end=evaluated.size();
  if(trace_count)for(auto a:{h,r,residual,mlp,x})evaluated.push_back(mx::contiguous(mx::astype(a,mx::float32,s),false,s));
  pending.insert(pending.end(),evaluated.begin(),evaluated.end());
  if((layer+1)%layer_group==0||layer+1==c.layers){mx::eval(pending);mx::synchronize(s);pending.clear();}
  x=evaluated[0];
  if(trace_count)for(size_t stage=0;stage<5;++stage){const auto& a=evaluated[state_end+stage];if(a.size()!=count*size_t(c.hidden))throw std::invalid_argument("trace shape");std::memcpy(trace+(size_t(layer)*5+stage)*count*c.hidden,a.data<float>(),a.size()*4);}
  for(size_t i=1;i<state_end;++i){const auto& a=evaluated[i];if(write+a.size()>new_count)throw std::invalid_argument("state write overflow");if(device_next){finite_checks.push_back(mx::all(mx::isfinite(a,s),s));device_next->push_back(a);}
   else std::memcpy(new_state+write,a.data<float>(),a.size()*4);write+=a.size();}
 }
 if(write!=new_count||read!=old_count)throw std::invalid_argument("state accounting mismatch");
 auto normalized=m.norm(x,"language_model.model.norm.weight");
 auto last=all_logits?normalized:mx::take(normalized,int(count)-1,1,s);
 auto output=mx::contiguous(mx::astype(m.linear(last,"language_model.lm_head"),mx::float32,s),false,s);
 std::vector<mx::array> final_outputs={output};if(hidden)final_outputs.push_back(normalized);final_outputs.insert(final_outputs.end(),finite_checks.begin(),finite_checks.end());mx::eval(final_outputs);mx::synchronize(s);
 for(const auto& finite:finite_checks)if(!finite.item<bool>())throw std::runtime_error("nonfinite device state");
 if(hidden){auto finite=mx::all(mx::isfinite(normalized,s),s);mx::eval(finite);mx::synchronize(s);if(!finite.item<bool>())throw std::runtime_error("nonfinite target hidden");*hidden=normalized;}
 if(output.size()!=logits_count)throw std::invalid_argument("logits shape mismatch");std::memcpy(logits,output.data<float>(),logits_count*4);
 }catch(...){try{mx::synchronize(s);}catch(...){}throw;}
});}

extern "C" int rs_qwen_forward(void* store,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t layer_count,const uint32_t* tokens,size_t count,size_t offset,const float* old_state,size_t old_count,float* new_state,size_t new_count,float* logits,size_t logits_count,int projection_checks,const uint32_t* projection_outputs,size_t projection_output_count,float* trace,size_t trace_count,char* error,size_t capacity){
 return qwen_forward_core(store,gpu,config,layers,layer_count,tokens,count,offset,old_state,old_count,new_state,new_count,logits,logits_count,projection_checks,projection_outputs,projection_output_count,trace,trace_count,error,capacity);
}
#include "qwen_device.inc"
