// Complete text-only Qwen3.5/3.6 hybrid reference, invoked only by its Rust executor.
#include <cmath>
#include <cstdio>
#include "attention_reference.h"
#include "projection_reference.h"
#include "operator_reference.h"
namespace {
struct Qwen {
 Weights& weights;rs_qwen_config c;mx::Stream s;bool dense=false,shift=false,reference=false,operators=false,full_reference=false,baseline=false;std::vector<int32_t> head_channels;
 Qwen(Weights& w,rs_qwen_config cfg,bool gpu):weights(w),c(cfg),s(mx::default_stream(mx::Device(gpu?mx::Device::gpu:mx::Device::cpu))),shift(w.has_shift){}
 const mx::array& w(const std::string& key){return weights.arrays.at(key);}
 OperatorInterval stage(){return OperatorInterval(weights.operators,c.operator_profile!=0);}
 void cut(OperatorInterval& timer,OperatorStage id,const std::vector<mx::array>& outputs){timer.finish(id,[&]{mx::eval(outputs);mx::synchronize(s);});}
 std::pair<int,int> quant(const std::string& p){auto it=weights.quant.find(p);return it==weights.quant.end()?std::pair<int,int>{c.group,c.bits}:it->second;}
#include "operator_probe.inc"
 mx::array silu(const mx::array& x){auto y=mx::multiply(x,mx::sigmoid(x,s),s);if(operators)check_operator("silu",scalar_reference::silu(cpu_values(x)),y);return y;}
 mx::array norm(const mx::array& x,const std::string& key,bool offset=true){
  if(!operators&&!baseline){
   auto ckey=key+(shift&&offset?".rms1":".rms0")+(x.dtype()==mx::float32?"_f32":"_bf16");
   auto it=weights.arrays.find(ckey);
   if(it==weights.arrays.end()){
    auto w_arr=mx::astype(w(key),x.dtype(),s);
    if(shift&&offset)w_arr=mx::add(w_arr,mx::array(1.f),s);
    w_arr=mx::contiguous(w_arr,false,s);mx::eval(w_arr);
    it=weights.arrays.emplace(std::move(ckey),std::move(w_arr)).first;
   }
   return mx::fast::rms_norm(x,it->second,c.epsilon,s);
  }
  auto weight=mx::astype(w(key),x.dtype(),s);if(shift&&offset)weight=mx::add(weight,mx::array(1.f),s);auto y=mx::fast::rms_norm(x,weight,c.epsilon,s);if(operators)check_operator(key+"/rms",scalar_reference::rms(cpu_values(x),cpu_values(weight),x.shape(-1),c.epsilon),y);return y;
 }
#include "projection_probe.inc"
 mx::array linear(const mx::array& x,const std::string& p){if(dense&&w(p+".weight").dtype()==mx::bfloat16){if(w(p+".weight").ndim()!=2)throw std::invalid_argument("MTP dense projection rank");auto tkey=p+".weight.T_f32";auto tit=weights.arrays.find(tkey);if(tit==weights.arrays.end()){auto m=mx::contiguous(mx::transpose(mx::astype(w(p+".weight"),mx::float32,s),{1,0},s),false,s);mx::eval(m);tit=weights.arrays.emplace(tkey,std::move(m)).first;}const auto& matrix=tit->second;auto flat=mx::reshape(x,{-1,x.shape(-1)},s);if(flat.shape(0)>8)throw std::invalid_argument("MTP dense row cap");auto shape=x.shape();shape.back()=w(p+".weight").shape(0);if(!reference)return mx::reshape(mx::matmul(flat,matrix,s),shape,s);std::vector<mx::array> rows;for(int row=0;row<flat.shape(0);++row)rows.push_back(mx::matmul(mx::take(flat,row,0,s),matrix,s));auto y=mx::reshape(mx::stack(rows,0,s),shape,s);if(reference)check_dense(x,y,w(p+".weight"),p);return y;}auto [g,b]=quant(p);auto y=mx::quantized_matmul(x,w(p+".weight"),w(p+".scales"),w(p+".biases"),true,g,b,"affine",s);if(reference)check_projection(x,y,w(p+".weight"),w(p+".scales"),w(p+".biases"),g,b,p);return y;}
 mx::array expert(const mx::array& x,const mx::array& idx,const std::string& p){
  auto [g,b]=quant(p);auto y=mx::gather_qmm(x,w(p+".weight"),w(p+".scales"),w(p+".biases"),{},idx,true,g,b,"affine",{},true,s);
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
  auto routing_timer=stage();
  auto flat=mx::reshape(x,{-1,c.hidden},s);auto router=linear(flat,p+".gate");auto gates=mx::softmax(router,-1,true,s);
  auto idx=mx::argpartition(gates,-c.top,-1,s);idx=mx::slice(idx,{0,c.experts-c.top},{flat.shape(0),c.experts},s);
  auto scores=mx::take_along_axis(gates,idx,-1,s);scores=mx::divide(scores,mx::sum(scores,-1,true,s),s);
  auto order=mx::argsort(idx,-1,s);idx=mx::take_along_axis(idx,order,-1,s);scores=mx::take_along_axis(scores,order,-1,s);
  if(operators)check_routing(router,gates,idx,scores,p);
  cut(routing_timer,OperatorStage::MoeRouting,{idx,scores});
  auto up_timer=stage();
  auto expanded=mx::expand_dims(flat,{-2,-3},s);
  auto up=expert(expanded,idx,p+".switch_mlp.up_proj");auto gate=expert(expanded,idx,p+".switch_mlp.gate_proj");
  cut(up_timer,OperatorStage::MoeUpGate,{up,gate});auto down_timer=stage();
  auto selected=expert(mx::multiply(silu(gate),up,s),idx,p+".switch_mlp.down_proj");
  cut(down_timer,OperatorStage::MoeDown,{selected});auto mixture_timer=stage();
  auto y=mx::sum(mx::multiply(mx::squeeze(selected,-2,s),mx::expand_dims(scores,-1,s),s),1,false,s);
  cut(mixture_timer,OperatorStage::MoeMixture,{y});auto shared_timer=stage();
  auto shared=linear(mx::multiply(silu(linear(flat,p+".shared_expert.gate_proj")),linear(flat,p+".shared_expert.up_proj"),s),p+".shared_expert.down_proj");
  auto shared_gate=linear(flat,p+".shared_expert_gate");cut(shared_timer,OperatorStage::MoeShared,{shared,shared_gate});auto merge_timer=stage();auto merged=mx::add(y,mx::multiply(shared,mx::sigmoid(shared_gate,s),s),s);cut(merge_timer,OperatorStage::MoeMerge,{merged});if(operators)check_mixture(selected,scores,y,shared,shared_gate,merged,p);
  return mx::reshape(merged,x.shape(),s);
 }
 mx::array attention(const mx::array& x,const std::string& p,int offset,mx::array old_k,mx::array old_v,std::vector<mx::array>& next,bool diagnose=false){
  auto projection_timer=stage();
  int batch=x.shape(0),t=x.shape(1);auto qg=mx::split(mx::reshape(linear(x,p+".q_proj"),{batch,t,c.q_heads,2*c.head_dim},s),2,-1,s);
  auto q=mx::transpose(norm(qg[0],p+".q_norm.weight"),{0,2,1,3},s);
  auto k=mx::transpose(norm(mx::reshape(linear(x,p+".k_proj"),{batch,t,c.kv_heads,c.head_dim},s),p+".k_norm.weight"),{0,2,1,3},s);
  auto v=mx::transpose(mx::reshape(linear(x,p+".v_proj"),{batch,t,c.kv_heads,c.head_dim},s),{0,2,1,3},s);
  q=rope_checked(q,offset,p+"/q-rope");k=rope_checked(k,offset,p+"/k-rope");
  if(offset){k=mx::concatenate({old_k,k},2,s);v=mx::concatenate({old_v,v},2,s);}
  next.push_back(k);next.push_back(v);
  cut(projection_timer,OperatorStage::AttentionProjection,{qg[1],q,k,v});auto reduction_timer=stage();
  mx::array y=q;
  if(!diagnose&&!reference&&!operators&&c.fused_attention){
   y=mx::fast::scaled_dot_product_attention(q,k,v,1.f/std::sqrt(float(c.head_dim)),t>1?"causal":"",{},{},false,s);
  }else{
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
   y=mx::concatenate(rows,2,s);
  }
  cut(reduction_timer,OperatorStage::AttentionReduction,{y});
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
  auto output_timer=stage();y=mx::reshape(mx::transpose(y,{0,2,1,3},s),{batch,t,c.q_heads*c.head_dim},s);
  auto gate=mx::reshape(qg[1],{batch,t,c.q_heads*c.head_dim},s);auto gated=mx::multiply(y,mx::sigmoid(gate,s),s);if(operators)check_gate(y,gate,gated,p+"/output-gate");auto output=linear(gated,p+".o_proj");cut(output_timer,OperatorStage::AttentionOutput,{output});return output;
 }
 mx::array recurrent(const mx::array& x,const std::string& p,mx::array conv,mx::array state,std::vector<mx::array>& next){
  auto projection_timer=stage();
  int batch=x.shape(0),t=x.shape(1),kd=c.key_heads*c.key_dim,vd=c.value_heads*c.value_dim,cd=2*kd+vd;
  auto qkv=linear(x,p+".in_proj_qkv"),z=mx::reshape(linear(x,p+".in_proj_z"),{batch,t,c.value_heads,c.value_dim},s);
  auto a=linear(x,p+".in_proj_a"),b=linear(x,p+".in_proj_b");
  cut(projection_timer,OperatorStage::RecurrentProjection,{qkv,z,a,b});auto prepare_timer=stage();
  auto input=mx::concatenate({conv,qkv},1,s);
  next.push_back(mx::contiguous(mx::slice(input,{0,t,0},{batch,t+c.conv-1,cd},s),false,s));
  mx::array cw=w(p+".conv1d.weight"),exp_a=w(p+".A_log");
  if(!operators&&!baseline&&x.dtype()==mx::float32){
   auto cw_key=p+".conv1d.weight.cw_f32";auto cw_it=weights.arrays.find(cw_key);
   if(cw_it==weights.arrays.end()){
    auto m=mx::astype(w(p+".conv1d.weight"),mx::float32,s);if(m.shape(-1)!=1)m=mx::transpose(m,{0,2,1},s);
    m=mx::contiguous(m,false,s);mx::eval(m);cw_it=weights.arrays.emplace(std::move(cw_key),std::move(m)).first;
   }
   cw=cw_it->second;
   auto ea_key=p+".A_log.exp_f32";auto ea_it=weights.arrays.find(ea_key);
   if(ea_it==weights.arrays.end()){
    auto m=mx::contiguous(mx::exp(mx::astype(w(p+".A_log"),mx::float32,s),s),false,s);mx::eval(m);
    ea_it=weights.arrays.emplace(std::move(ea_key),std::move(m)).first;
   }
   exp_a=ea_it->second;
  }else{
   cw=mx::astype(w(p+".conv1d.weight"),x.dtype(),s);if(cw.shape(-1)!=1)cw=mx::transpose(cw,{0,2,1},s);
   exp_a=mx::exp(mx::astype(w(p+".A_log"),mx::float32,s),s);
  }
  auto activated=silu(mx::conv1d(input,cw,1,0,1,cd,s));auto parts=mx::split(activated,{kd,2*kd},-1,s);
  auto q=mx::reshape(parts[0],{batch,t,c.key_heads,c.key_dim},s),k=mx::reshape(parts[1],{batch,t,c.key_heads,c.key_dim},s),v=mx::reshape(parts[2],{batch,t,c.value_heads,c.value_dim},s);
  q=mx::multiply(mx::fast::rms_norm(q,{},1e-6,s),mx::array(1.f/c.key_dim),s);
  k=mx::multiply(mx::fast::rms_norm(k,{},1e-6,s),mx::array(1.f/std::sqrt(float(c.key_dim))),s);
  auto beta=mx::sigmoid(b,s);auto softplus=mx::logaddexp(mx::add(a,w(p+".dt_bias"),s),mx::array(0.f),s);
  auto decay=mx::exp(mx::negative(mx::multiply(exp_a,softplus,s),s),s);
  auto initial_state=state;
  cut(prepare_timer,OperatorStage::RecurrentPrepare,{q,k,v,decay,beta,next[0]});auto update_timer=stage();
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
  }();cut(update_timer,OperatorStage::RecurrentUpdate,{readout,state});if(operators)check_delta(qkv,conv,cw,a,b,initial_state,activated,readout,state,next[0],p);
  auto output_timer=stage();
  next.push_back(state);auto y=norm(readout,p+".norm.weight",false);
  auto ungated=y;y=mx::astype(mx::multiply(mx::astype(y,mx::float32,s),silu(mx::astype(z,mx::float32,s)),s),x.dtype(),s);if(operators)check_gate(ungated,z,y,p+"/output-gate",true);
  auto output=linear(mx::reshape(y,{batch,t,vd},s),p+".out_proj");cut(output_timer,OperatorStage::RecurrentOutput,{output});return output;
 }
 std::vector<mx::array> compiled_recurrent(const mx::array& x,const std::string& prefix,const mx::array& conv,const mx::array& state,bool multi_token=false){
  if(x.ndim()!=3||(multi_token?(x.shape(1)<1||x.shape(1)>8):(x.shape(1)!=1))||x.shape(0)<1||x.shape(0)>3||x.dtype()!=mx::float32||reference||operators)throw std::invalid_argument("compiled recurrent geometry/profile");
  // This weight owner fixes model, quant overrides, stream and architecture for its lifetime.
  if(!multi_token&&x.shape(1)==1){
   auto key=prefix+"/B"+std::to_string(x.shape(0))+"/dtype"+std::to_string(int(x.dtype().val()))+"/group"+std::to_string(c.group)+"/bits"+std::to_string(c.bits)+"/recurrent"+std::to_string(c.fused_recurrent);
   auto found=weights.compiled_blocks.find(key);
   if(found==weights.compiled_blocks.end()){
    if(weights.compiled_blocks.size()>=90)throw std::runtime_error("compiled recurrent cache cap");
    auto cfg=c;cfg.compiled_decode=0;cfg.execution_timing=0;cfg.operator_profile=0;
    auto fn=mx::compile(std::function<std::vector<mx::array>(const std::vector<mx::array>&)>([owner=&weights,cfg,prefix](const std::vector<mx::array>& in){
     if(in.size()!=3)throw std::invalid_argument("compiled block input inventory");
     Qwen m(*owner,cfg,true);std::vector<mx::array> next;
     auto h=m.norm(in[0],prefix+".input_layernorm.weight");auto r=m.recurrent(h,prefix+".linear_attn",in[1],in[2],next);
     auto residual=mx::add(in[0],r,m.s);auto output=mx::add(residual,m.moe(m.norm(residual,prefix+".post_attention_layernorm.weight"),prefix+".mlp"),m.s);
     if(next.size()!=2)throw std::runtime_error("compiled block output inventory");
     return std::vector<mx::array>{output,next[0],next[1]};
    }),false);
    found=weights.compiled_blocks.emplace(std::move(key),std::move(fn)).first;++weights.compiled_misses;
   }
   auto result=found->second({x,conv,state});if(result.size()!=3)throw std::runtime_error("compiled block result inventory");++weights.compiled_calls;return result;
  }
  auto key=prefix+"/B"+std::to_string(x.shape(0))+"/T"+std::to_string(x.shape(1))+"/dtype"+std::to_string(int(x.dtype().val()))+"/group"+std::to_string(c.group)+"/bits"+std::to_string(c.bits)+"/recurrent"+std::to_string(c.fused_recurrent);
  auto found=weights.fast_compiled_blocks.find(key);
  if(found==weights.fast_compiled_blocks.end()){
   if(weights.fast_compiled_blocks.size()>=1024)throw std::runtime_error("expanded compiler cache cap");
   auto cfg=c;cfg.compiled_decode=0;cfg.execution_timing=0;cfg.operator_profile=0;
   auto fn=mx::compile(std::function<std::vector<mx::array>(const std::vector<mx::array>&)>([owner=&weights,cfg,prefix](const std::vector<mx::array>& in){
    if(in.size()!=3)throw std::invalid_argument("compiled block input inventory");
    Qwen m(*owner,cfg,true);std::vector<mx::array> next;
    auto h=m.norm(in[0],prefix+".input_layernorm.weight");auto r=m.recurrent(h,prefix+".linear_attn",in[1],in[2],next);
    auto residual=mx::add(in[0],r,m.s);auto output=mx::add(residual,m.moe(m.norm(residual,prefix+".post_attention_layernorm.weight"),prefix+".mlp"),m.s);
    if(next.size()!=2)throw std::runtime_error("compiled block output inventory");
    return std::vector<mx::array>{output,next[0],next[1]};
   }),false);
   found=weights.fast_compiled_blocks.emplace(std::move(key),std::move(fn)).first;++weights.compiled_misses;
  }
  auto result=found->second({x,conv,state});if(result.size()!=3)throw std::runtime_error("compiled block result inventory");++weights.compiled_calls;return result;
 }
 mx::array compiled_post_moe(const mx::array& residual,const std::string& prefix){
  if(residual.ndim()!=3||residual.shape(1)<1||residual.shape(1)>8||residual.shape(0)<1||residual.shape(0)>3||residual.dtype()!=mx::float32||reference||operators)throw std::invalid_argument("compiled post-moe geometry/profile");
  auto key=prefix+"/post_moe/B"+std::to_string(residual.shape(0))+"/T"+std::to_string(residual.shape(1))+"/group"+std::to_string(c.group)+"/bits"+std::to_string(c.bits)+"/dense"+std::to_string(int(dense));
  auto found=weights.fast_compiled_blocks.find(key);
  if(found==weights.fast_compiled_blocks.end()){
   if(weights.fast_compiled_blocks.size()>=1024)throw std::runtime_error("expanded compiler cache cap");
   auto cfg=c;cfg.compiled_decode=0;cfg.execution_timing=0;cfg.operator_profile=0;bool is_dense=dense,is_shift=shift;
   auto fn=mx::compile(std::function<std::vector<mx::array>(const std::vector<mx::array>&)>([owner=&weights,cfg,prefix,is_dense,is_shift](const std::vector<mx::array>& in){
    if(in.size()!=1)throw std::invalid_argument("compiled post-moe input inventory");
    Qwen m(*owner,cfg,true);m.dense=is_dense;m.shift=is_shift;
    auto post=m.norm(in[0],prefix+".post_attention_layernorm.weight");
    auto mlp=m.moe(post,prefix+".mlp");
    return std::vector<mx::array>{mx::add(in[0],mlp,m.s)};
   }),false);
   found=weights.fast_compiled_blocks.emplace(std::move(key),std::move(fn)).first;++weights.compiled_misses;
  }
  auto result=found->second({residual});if(result.size()!=1)throw std::runtime_error("compiled post-moe result inventory");++weights.compiled_calls;return result[0];
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
static int qwen_forward_core(void* store,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t layer_count,const uint32_t* tokens,size_t count,size_t offset,const float* old_state,size_t old_count,float* new_state,size_t new_count,float* logits,size_t logits_count,int projection_checks,const uint32_t* projection_outputs,size_t projection_output_count,float* trace,size_t trace_count,char* error,size_t capacity,const std::vector<mx::array>* device_old=nullptr,std::vector<mx::array>* device_next=nullptr,bool all_logits=false,std::optional<mx::array>* hidden=nullptr,bool sample_token=false,uint32_t* sampled_token=nullptr){return checked(error,capacity,[&]{
 if(!store||!config||!layers||!tokens||(!new_state&&!device_next)||(!logits&&!sample_token))throw std::invalid_argument("invalid model descriptor");auto c=*config;
 if(c.hidden<=0||c.hidden>8192||c.layers<=0||c.layers>64||c.vocab<=0||c.vocab>300000||c.interval<=0||c.layers%c.interval||c.context<=0||c.context>262144||c.chunk<=0||c.chunk>8||count==0||count>size_t(c.chunk)||offset>size_t(c.context)||offset+count>size_t(c.context)||(sample_token?logits_count!=0:logits_count!=size_t(c.vocab)*(all_logits?count:1)))throw std::invalid_argument("model geometry/capacity");
 if(c.layer_group!=1&&c.layer_group!=4&&c.layer_group!=10&&c.layer_group!=40)throw std::invalid_argument("model layer group");
 if(c.fused_recurrent<0||c.fused_recurrent>1)throw std::invalid_argument("recurrent kernel mode");
 if(c.execution_timing<0||c.execution_timing>1)throw std::invalid_argument("execution timing mode");
 if(c.compiled_decode<0||c.compiled_decode>1)throw std::invalid_argument("compiled decode mode");
 if(c.operator_profile<0||c.operator_profile>1||(c.operator_profile&&(c.compiled_decode||c.execution_timing)))throw std::invalid_argument("operator profile mode");
 if(sample_token&&(!device_next||all_logits||trace_count||projection_checks||hidden||!sampled_token))throw std::invalid_argument("sampled step mode");
 if(c.validation_mode<0||c.validation_mode>2||c.validation_interval<1||c.validation_interval>1000000)throw std::invalid_argument("validation mode");
 // Snapshot/trace controls use the original recurrence even in a tuned session.
 if(!device_next||trace_count||projection_checks){c.fused_recurrent=0;c.compiled_decode=0;c.operator_profile=0;c.fused_attention=0;}
 if(layer_count!=size_t(c.layers))throw std::invalid_argument("IR layer count");
 for(size_t i=0;i<layer_count;++i)if(layers[i].index!=i||layers[i].mixer>1||layers[i].mixer!=uint32_t((i+1)%c.interval==0))throw std::invalid_argument("IR layer program");
 for(auto d:{c.q_heads,c.kv_heads,c.head_dim,c.key_heads,c.value_heads,c.key_dim,c.value_dim,c.conv,c.experts,c.top,c.intermediate,c.shared})if(d<=0||d>8192)throw std::invalid_argument("model dimension bounds");
 if(c.activation_precision<0||c.activation_precision>1||c.q_heads%c.kv_heads||c.value_heads%c.key_heads||c.conv<2||c.top>c.experts||c.rope_dims<=0||c.rope_dims>c.head_dim||c.rope_dims%2||!std::isfinite(c.epsilon)||c.epsilon<=0||!std::isfinite(c.theta)||c.theta<=1)throw std::invalid_argument("model geometry mismatch");
 if(new_count!=qwen_state_count(c,offset+count)||(offset?old_count!=qwen_state_count(c,offset)||(!old_state&&!device_old):old_count!=0))throw std::invalid_argument("model state length");
 if(new_count>3ULL*1024*1024*1024)throw std::invalid_argument("model state byte cap");
 for(size_t i=0;i<count;++i)if(tokens[i]>=uint32_t(c.vocab))throw std::invalid_argument("token outside vocabulary");
 if(trace_count && (!trace||trace_count!=size_t(c.layers)*5*count*c.hidden||trace_count>64*5*8*8192))throw std::invalid_argument("trace capacity");
 if(projection_checks<0||projection_checks>7||((projection_checks&4)&&((projection_checks&3)!=3))||(projection_checks&&!trace_count))throw std::invalid_argument("projection diagnostic mode");
 if(projection_output_count>8||(projection_output_count&&(!projection_outputs||!(projection_checks&1))))throw std::invalid_argument("projection output capacity");
 for(size_t i=0;i<projection_output_count;++i){if(projection_outputs[i]>=uint32_t(c.vocab))throw std::invalid_argument("projection output vocabulary");for(size_t j=0;j<i;++j)if(projection_outputs[i]==projection_outputs[j])throw std::invalid_argument("duplicate projection output");}
 ForwardTiming timer(static_cast<Weights*>(store)->timing,c.execution_timing&&device_next&&!trace_count&&!projection_checks);
 Qwen m(*static_cast<Weights*>(store),c,gpu!=0);m.baseline=!device_next||trace_count||projection_checks;m.reference=(projection_checks&1)!=0;m.operators=(projection_checks&2)!=0;m.full_reference=(projection_checks&4)!=0;for(size_t i=0;i<projection_output_count;++i)m.head_channels.push_back(int32_t(projection_outputs[i]));auto s=m.s;
 const bool run_checks=device_next&&(c.validation_mode>=2||c.operator_profile);
 try{
 auto embedding_timer=m.stage();
 auto ids=mx::array(tokens,{int(count)},mx::uint32);std::string embed="language_model.model.embed_tokens";auto [group,bits]=m.quant(embed);
 auto embed_scales=mx::take(m.w(embed+".scales"),ids,0,s),embed_biases=mx::take(m.w(embed+".biases"),ids,0,s);
 if(c.activation_precision){embed_scales=mx::astype(embed_scales,mx::float32,s);embed_biases=mx::astype(embed_biases,mx::float32,s);}
 auto x=mx::reshape(mx::dequantize(mx::take(m.w(embed+".weight"),ids,0,s),embed_scales,embed_biases,group,bits,"affine",{},c.activation_precision?std::optional<mx::Dtype>{mx::float32}:std::nullopt,s),{1,int(count),c.hidden},s);
 if(m.operators)m.check_embedding(ids,x,embed,group,bits);
 m.cut(embedding_timer,OperatorStage::Embedding,{x});
 size_t read=0,write=0,array_read=0;std::vector<mx::array> finite_checks,pending;
 // Diagnostics/snapshot controls keep the original completion boundaries.
 const int layer_group=(device_next&&!trace_count&&!projection_checks)?c.layer_group:1;
 const bool multi_compile=(all_logits||hidden!=nullptr);
 auto state=[&](mx::Shape shape,mx::Dtype dtype){size_t n=1;for(auto d:shape)n*=d;auto a=mx::zeros(shape,dtype,s);if(offset){
   if(device_old){if(array_read>=device_old->size()||(*device_old)[array_read].shape()!=shape)throw std::invalid_argument("device state shape");a=((*device_old)[array_read].dtype()==dtype)?(*device_old)[array_read]:mx::astype((*device_old)[array_read],dtype,s);++array_read;}
   else a=mx::astype(mx::array(old_state+read,shape,mx::float32),dtype,s);read+=n;
  }return a;};
 for(int layer=0;layer<c.layers;++layer){
  std::string prefix="language_model.model.layers."+std::to_string(layer);std::vector<mx::array> next;
  mx::array h=x,r=x,residual=x,mlp=x;
  if(c.compiled_decode&&(count==1||multi_compile)&&layers[layer].mixer==0){
   int cd=2*c.key_heads*c.key_dim+c.value_heads*c.value_dim;auto conv=state({1,c.conv-1,cd},x.dtype());auto recurrent=state({1,c.value_heads,c.value_dim,c.key_dim},mx::float32);
   auto result=m.compiled_recurrent(x,prefix,conv,recurrent,multi_compile&&count>1);x=result[0];next={result[1],result[2]};
  }else{
  auto norm_timer=m.stage();h=m.norm(x,prefix+".input_layernorm.weight");r=h;m.cut(norm_timer,OperatorStage::InputNorm,{h});
  if(layers[layer].mixer==0){int cd=2*c.key_heads*c.key_dim+c.value_heads*c.value_dim;auto conv=state({1,c.conv-1,cd},x.dtype());auto recurrent=state({1,c.value_heads,c.value_dim,c.key_dim},mx::float32);r=m.recurrent(h,prefix+".linear_attn",conv,recurrent,next);}
  else{auto k=state({1,c.kv_heads,int(offset),c.head_dim},x.dtype());auto v=state({1,c.kv_heads,int(offset),c.head_dim},x.dtype());r=m.attention(h,prefix+".self_attn",int(offset),k,v,next,trace_count!=0);}
  if(c.compiled_decode&&layers[layer].mixer==1&&!trace_count&&!projection_checks&&!c.operator_profile){
   residual=mx::add(x,r,s);x=m.compiled_post_moe(residual,prefix);
  }else{
   auto residual_timer=m.stage();residual=mx::add(x,r,s);if(m.operators)m.check_add(x,r,residual,prefix+"/attention-residual");auto post=m.norm(residual,prefix+".post_attention_layernorm.weight");m.cut(residual_timer,OperatorStage::ResidualNorm,{residual,post});mlp=m.moe(post,prefix+".mlp");auto output_timer=m.stage();x=mx::add(residual,mlp,s);if(m.operators)m.check_add(residual,mlp,x,prefix+"/mlp-residual");m.cut(output_timer,OperatorStage::LayerOutput,{x});
  }
  }
  std::vector<mx::array> evaluated={x};
  for(auto& a:next){
   if(device_next&&!trace_count)evaluated.push_back(a.dtype()==mx::float32?a:mx::astype(a,mx::float32,s));
   else evaluated.push_back(mx::contiguous(mx::astype(a,mx::float32,s),false,s));
  }
  size_t state_end=evaluated.size();
  if(trace_count)for(auto a:{h,r,residual,mlp,x})evaluated.push_back(mx::contiguous(mx::astype(a,mx::float32,s),false,s));
  pending.insert(pending.end(),evaluated.begin(),evaluated.end());
  const bool flush_group=((layer+1)%layer_group==0||layer+1==c.layers)&&(layer+1<c.layers||!device_next||trace_count||c.operator_profile);
  if(flush_group){timer.graph();mx::eval(pending);mx::synchronize(s);timer.wait();pending.clear();}
  x=evaluated[0];
  if(trace_count)for(size_t stage=0;stage<5;++stage){const auto& a=evaluated[state_end+stage];if(a.size()!=count*size_t(c.hidden))throw std::invalid_argument("trace shape");std::memcpy(trace+(size_t(layer)*5+stage)*count*c.hidden,a.data<float>(),a.size()*4);}
  for(size_t i=1;i<state_end;++i){const auto& a=evaluated[i];if(write+a.size()>new_count)throw std::invalid_argument("state write overflow");if(device_next){if(run_checks)finite_checks.push_back(c.operator_profile?a:mx::all(mx::isfinite(a,s),s));device_next->push_back(a);}
   else std::memcpy(new_state+write,a.data<float>(),a.size()*4);write+=a.size();}
 }
 if(write!=new_count||read!=old_count)throw std::invalid_argument("state accounting mismatch");
 auto projection_timer=m.stage();
 auto normalized=m.norm(x,"language_model.model.norm.weight");
 auto last=all_logits?normalized:mx::take(normalized,int(count)-1,1,s);
 auto output=mx::contiguous(mx::astype(m.linear(last,"language_model.lm_head"),mx::float32,s),false,s);
 m.cut(projection_timer,OperatorStage::FinalProjection,{output});
 auto validation_timer=m.stage();
 if(c.operator_profile){for(auto& a:finite_checks)a=mx::all(mx::isfinite(a,s),s);}
 if(run_checks)finite_checks.push_back(mx::all(mx::isfinite(output,s),s));
 if(hidden&&run_checks)finite_checks.push_back(mx::all(mx::isfinite(normalized,s),s));
 m.cut(validation_timer,OperatorStage::StateValidation,finite_checks);
 std::optional<mx::array> sampled_arr;if(sample_token)sampled_arr=mx::argmax(output,-1,false,s);
 std::vector<mx::array> final_outputs=std::move(pending);final_outputs.push_back(output);if(sample_token)final_outputs.push_back(*sampled_arr);if(hidden)final_outputs.push_back(normalized);final_outputs.insert(final_outputs.end(),finite_checks.begin(),finite_checks.end());timer.graph();mx::eval(final_outputs);mx::synchronize(s);timer.wait();
 for(const auto& finite:finite_checks)if(!finite.item<bool>())throw std::runtime_error("nonfinite device state");
 if(hidden)*hidden=normalized;
 if(sample_token){*sampled_token=sampled_arr->item<uint32_t>();}else{if(output.size()!=logits_count)throw std::invalid_argument("logits shape mismatch");std::memcpy(logits,output.data<float>(),logits_count*4);}timer.complete();
 }catch(...){try{mx::synchronize(s);}catch(...){}throw;}
});}

extern "C" int rs_qwen_forward(void* store,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t layer_count,const uint32_t* tokens,size_t count,size_t offset,const float* old_state,size_t old_count,float* new_state,size_t new_count,float* logits,size_t logits_count,int projection_checks,const uint32_t* projection_outputs,size_t projection_output_count,float* trace,size_t trace_count,char* error,size_t capacity){
 return qwen_forward_core(store,gpu,config,layers,layer_count,tokens,count,offset,old_state,old_count,new_state,new_count,logits,logits_count,projection_checks,projection_outputs,projection_output_count,trace,trace_count,error,capacity);
}
#include "qwen_device.inc"
