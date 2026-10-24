#include "state_table.h"
#include <atomic>
namespace {
struct DevicePayload {std::vector<mx::array> arrays;std::optional<mx::array> hidden;};
struct DeviceSession {
 void* weights;void* mtp_weights=nullptr;int gpu;size_t slots;uint64_t limit;rs_qwen_config config;std::vector<rs_qwen_layer> layers;
 StateTable<DevicePayload> table;StateTable<DevicePayload> drafts;
 uint64_t exports=0,uploads=0,steps=0;
 DeviceSession(void* w,int g,rs_qwen_config c,const rs_qwen_layer* l,size_t n,uint64_t owner,size_t slots,uint64_t bytes):weights(w),gpu(g),slots(slots),limit(bytes),config(c),layers(l,l+n),table(owner,slots,bytes),drafts(owner+1,slots,bytes){}
};
}
extern "C" void* rs_qwen_device_create(void* weights,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t count,size_t slots,uint64_t bytes,char* error,size_t capacity){
 void* result=nullptr;checked(error,capacity,[&]{
 if(!weights||!config||!layers||count!=40||gpu<0||gpu>1)throw std::invalid_argument("device session descriptor");const auto& c=*config;
 if(c.layer_group!=1&&c.layer_group!=4&&c.layer_group!=10&&c.layer_group!=40)throw std::invalid_argument("device layer group");
 if(c.fused_recurrent<0||c.fused_recurrent>1||(c.fused_recurrent&&!gpu))throw std::invalid_argument("device recurrent kernel");
 if(c.execution_timing<0||c.execution_timing>1)throw std::invalid_argument("device execution timing");
 if(c.compiled_decode&&c.activation_precision!=1)throw std::invalid_argument("compiled decode requires FP32");
 if(c.compiled_decode<0||c.compiled_decode>1||(c.compiled_decode&&(!gpu||std::getenv("MLX_DISABLE_COMPILE")||mx::default_device()!=mx::Device(mx::Device::gpu))))throw std::invalid_argument("compiled decode unavailable");
 if(c.operator_profile<0||c.operator_profile>1||(c.operator_profile&&(!gpu||c.compiled_decode||c.execution_timing)))throw std::invalid_argument("operator profile requires GPU/eager/timing off");
 if(c.expert_kernel<0||c.expert_kernel>2)throw std::invalid_argument("device expert kernel");
 if(c.validation_mode<0||c.validation_mode>2||c.validation_interval<1||c.validation_interval>1000000)throw std::invalid_argument("device validation mode");
 if(c.fused_attention<0||c.fused_attention>1||(c.fused_attention&&!gpu))throw std::invalid_argument("fused attention requires GPU");
 // This ABI is deliberately restricted to the qualified architecture and FP32 profile, with an experimental larger-context ABI bound; Rust serving remains context 512.
 if(c.hidden!=2048||c.layers!=40||c.vocab!=248320||c.q_heads!=16||c.kv_heads!=2||c.head_dim!=256||c.key_heads!=16||c.value_heads!=32||c.key_dim!=128||c.value_dim!=128||c.conv!=4||c.interval!=4||c.context<512||c.context>262144||c.chunk!=8||(c.activation_precision<0||c.activation_precision>1)||c.experts!=256||c.top!=8||c.intermediate!=512||c.shared!=512||c.rope_dims!=64||c.epsilon!=1e-6f||c.theta!=10000000.f||!(c.group==32||c.group==64||c.group==128)||!(c.bits==2||c.bits==3||c.bits==4||c.bits==5||c.bits==6||c.bits==8))throw std::invalid_argument("device session architecture/profile");
 for(size_t i=0;i<count;++i)if(layers[i].index!=i||layers[i].mixer!=uint32_t((i+1)%4==0))throw std::invalid_argument("device session program");
 static std::atomic<uint64_t> next{1};auto owner=next.load();for(;;){if(!owner||owner>=UINT64_MAX-1)throw std::overflow_error("session owner exhausted");if(next.compare_exchange_weak(owner,owner+2))break;}
 result=new DeviceSession(weights,gpu,c,layers,count,owner,slots,bytes);
 });return result;
}
static int device_step(void* handle,const StateId* previous,size_t position,const uint32_t* tokens,size_t count,StateId* output,float* logits,size_t logits_count,float* trace,size_t trace_count,char* error,size_t capacity,bool all_logits,bool capture_hidden=false,bool sampled=false,uint32_t* sampled_token=nullptr){return checked(error,capacity,[&]{
 if(!handle||!output||!tokens||(!logits&&!sampled))throw std::invalid_argument("device step descriptor");auto& session=*static_cast<DeviceSession*>(handle);const auto& c=session.config;
 if(!count||count>8||position>size_t(c.context)||count>size_t(c.context)-position||(sampled?logits_count!=0:logits_count!=size_t(c.vocab)*(all_logits?count:1)))throw std::invalid_argument("device step capacity");
 if((position==0)!=(!previous))throw std::invalid_argument("device previous state presence");
 const std::vector<mx::array>* old=nullptr;if(previous){if(previous->position!=position)throw std::invalid_argument("device position mismatch");old=&session.table.get(*previous).arrays;}
 auto state_count=qwen_state_count(c,position+count);auto hidden_bytes=capture_hidden?count*size_t(c.hidden)*4:0;
 if(session.table.residents()+session.drafts.residents()>=session.slots || state_count*4+hidden_bytes>session.limit-session.table.bytes()-session.drafts.bytes())throw std::runtime_error("combined resident budget exhausted");
 auto id=session.table.reserve(position+count,state_count*4+hidden_bytes);
 try{
  DevicePayload next;
  auto cfg=c;if(cfg.validation_mode==1)cfg.validation_mode=(session.steps%uint64_t(cfg.validation_interval)==0)?2:0;
  char inner[1024]={};auto code=qwen_forward_core(session.weights,session.gpu,&cfg,session.layers.data(),session.layers.size(),tokens,count,position,nullptr,position?qwen_state_count(c,position):0,nullptr,state_count,logits,logits_count,0,nullptr,0,trace,trace_count,inner,sizeof(inner),old,&next.arrays,all_logits,capture_hidden?&next.hidden:nullptr,sampled,sampled_token);
  if(code)throw std::runtime_error(inner);
  if(next.arrays.size()!=80)throw std::runtime_error("device state array inventory");
  session.table.publish(id,std::move(next));++session.steps;*output=id;
 }catch(...){session.table.rollback(id);throw;}
});}
extern "C" int rs_qwen_device_step(void* h,const StateId* p,size_t position,const uint32_t* t,size_t n,StateId* out,float* logits,size_t lc,float* trace,size_t tc,char* e,size_t ec){return device_step(h,p,position,t,n,out,logits,lc,trace,tc,e,ec,false);}
extern "C" int rs_qwen_device_step_sampled(void* h,const StateId* p,size_t position,const uint32_t* t,size_t n,StateId* out,uint32_t* token,char* e,size_t ec){return device_step(h,p,position,t,n,out,nullptr,0,nullptr,0,e,ec,false,false,true,token);}
extern "C" int rs_qwen_device_block(void* h,const StateId* p,size_t position,const uint32_t* t,size_t n,StateId* out,float* logits,size_t lc,char* e,size_t ec){return device_step(h,p,position,t,n,out,logits,lc,nullptr,0,e,ec,true);}
extern "C" int rs_qwen_device_hidden_step(void* h,const StateId* p,size_t position,const uint32_t* t,size_t n,StateId* out,float* logits,size_t lc,char* e,size_t ec){return device_step(h,p,position,t,n,out,logits,lc,nullptr,0,e,ec,false,true);}
extern "C" int rs_qwen_device_hidden_block(void* h,const StateId* p,size_t position,const uint32_t* t,size_t n,StateId* out,float* logits,size_t lc,char* e,size_t ec){return device_step(h,p,position,t,n,out,logits,lc,nullptr,0,e,ec,true,true);}
extern "C" int rs_qwen_device_release(void* handle,const StateId* id,char* error,size_t capacity){return checked(error,capacity,[&]{if(!handle||!id)throw std::invalid_argument("release descriptor");static_cast<DeviceSession*>(handle)->table.release(*id);});}
extern "C" int rs_qwen_device_snapshot(void* handle,const StateId* id,float* output,size_t count,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!id||!output)throw std::invalid_argument("snapshot descriptor");auto& session=*static_cast<DeviceSession*>(handle);const auto& arrays=session.table.get(*id).arrays;if(count!=qwen_state_count(session.config,id->position))throw std::invalid_argument("snapshot capacity");size_t offset=0;
 auto s=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));
 for(const auto& a:arrays){if(a.size()>count-offset)throw std::runtime_error("snapshot array capacity");if(a.dtype()==mx::float32&&a.flags().row_contiguous){std::memcpy(output+offset,a.data<float>(),a.size()*4);}else{auto ca=mx::contiguous(mx::astype(a,mx::float32,s),false,s);mx::eval(ca);mx::synchronize(s);std::memcpy(output+offset,ca.data<float>(),ca.size()*4);}offset+=a.size();}
 if(offset!=count)throw std::runtime_error("snapshot accounting");session.exports+=count*4;
});}
extern "C" int rs_qwen_device_stats(void* handle,uint64_t* output,size_t count,char* error,size_t capacity){return checked(error,capacity,[&]{if(!handle||!output||count!=17+operator_stage_count*3)throw std::invalid_argument("stats capacity");auto& s=*static_cast<DeviceSession*>(handle);output[0]=s.table.residents()+s.drafts.residents();output[1]=s.table.bytes()+s.drafts.bytes();output[2]=s.exports;output[3]=s.uploads;output[4]=s.steps;uint64_t active_mem=0,peak_mem=0;try{active_mem=mx::get_active_memory();peak_mem=mx::get_peak_memory();}catch(...){}output[5]=active_mem;output[6]=peak_mem;auto& t=static_cast<Weights*>(s.weights)->timing;output[7]=t.calls;output[8]=t.completed;output[9]=t.graph_ns;output[10]=t.wait_ns;output[11]=t.export_ns;output[12]=t.wall_ns;output[13]=t.groups;auto& w=*static_cast<Weights*>(s.weights);output[14]=w.compiled_blocks.size()+w.fast_compiled_blocks.size();output[15]=w.compiled_misses;output[16]=w.compiled_calls;for(size_t i=0;i<operator_stage_count;++i){auto& m=w.operators[i];output[17+i*3]=m.calls;output[18+i*3]=m.host_ns;output[19+i*3]=m.wait_ns;}});}
extern "C" void rs_qwen_device_destroy(void* handle){delete static_cast<DeviceSession*>(handle);}

extern "C" int rs_qwen_device_drain(void* handle,uint64_t* invalidated,char* error,size_t capacity){return checked(error,capacity,[&]{if(!handle||!invalidated)throw std::invalid_argument("drain descriptor");auto& s=*static_cast<DeviceSession*>(handle);*invalidated=s.table.residents()+s.drafts.residents();s.table.clear();s.drafts.clear();if(s.table.residents()||s.table.bytes()||s.drafts.residents()||s.drafts.bytes())throw std::runtime_error("device drain residual");});}

extern "C" int rs_qwen_device_hidden_snapshot(void* handle,const StateId* id,float* output,size_t count,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!id||!output)throw std::invalid_argument("hidden snapshot descriptor");auto& s=*static_cast<DeviceSession*>(handle);const auto& value=s.table.get(*id).hidden;
 if(!value||value->dtype()!=mx::float32||value->size()!=count)throw std::invalid_argument("missing hidden or snapshot capacity");
 std::memcpy(output,value->data<float>(),count*4);s.exports+=count*4;
});}

#include "qwen_mtp.inc"

#include "qwen_batch.inc"
