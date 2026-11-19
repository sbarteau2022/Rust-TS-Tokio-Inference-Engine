#include "dspark.h"
namespace {
struct DSparkContext {native_dspark::Cache cache;StateId target;};
struct DSparkSession {
 DeviceSession* target;int head_mode;native_dspark::Draft draft;StateTable<DSparkContext> contexts;
 DSparkSession(DeviceSession* t,const char* path,int head,size_t slots,uint64_t bytes,uint64_t owner):target(t),head_mode(head&1),draft(true,head<2),contexts(owner,slots,bytes){draft.load(path);}
};
}
extern "C" int rs_qwen_device_dspark_step(void* h,const StateId* p,size_t pos,const uint32_t* ids,size_t n,StateId* out,float* logits,size_t lc,int all,char* e,size_t ec){if(all<0||all>1)return checked(e,ec,[]{throw std::invalid_argument("DSpark logits mode");});return device_step(h,p,pos,ids,n,out,logits,lc,nullptr,0,e,ec,all!=0,false,false,nullptr,true);}
extern "C" void* rs_qwen_dspark_create(void* target,const char* path,int head,size_t slots,uint64_t bytes,char* e,size_t ec){void* out=nullptr;checked(e,ec,[&]{
 if(!target||!path||head<0||head>3||!slots||slots>64||!bytes)throw std::invalid_argument("DSpark session descriptor");auto* t=static_cast<DeviceSession*>(target);
 if(!t->gpu||t->config.activation_precision!=1||t->config.context!=512||t->config.validation_mode!=2)throw std::invalid_argument("DSpark requires GPU/FP32/full-validation/context512");
 auto owner=allocate_native_owner();
 out=new DSparkSession(t,path,head,slots,bytes,owner);
 });return out;}
extern "C" void rs_qwen_dspark_destroy(void* h){delete static_cast<DSparkSession*>(h);}
extern "C" int rs_qwen_dspark_ingest(void* h,const StateId* target,const StateId* previous,StateId* out,char* e,size_t ec){return checked(e,ec,[&]{
 if(!h||!target||!out)throw std::invalid_argument("DSpark ingest descriptor");auto& s=*static_cast<DSparkSession*>(h);const auto& payload=s.target->table.get(*target);
 if(!payload.dspark_features)throw std::invalid_argument("target has no DSpark features");auto& taps=*payload.dspark_features;
 native_dspark::Cache empty;const auto* prior=previous?&s.contexts.get(*previous):nullptr;const auto& old=prior?prior->cache:empty;
 if(prior&&(!payload.parent||std::memcmp(&*payload.parent,&prior->target,sizeof(StateId))))throw std::invalid_argument("DSpark target branch identity");
 if(target->position<size_t(taps.shape(1))||target->position-taps.shape(1)!=old.position)throw std::invalid_argument("DSpark context alignment");
 uint64_t bytes=target->position*6*2*8*128*4;auto id=s.contexts.reserve(target->position,bytes);
 try{auto next=s.draft.ingest(taps,old);s.contexts.publish(id,DSparkContext{std::move(next),*target});*out=id;}catch(...){s.contexts.rollback(id);throw;}
});}
extern "C" int rs_qwen_dspark_propose(void* h,const StateId* context,uint32_t anchor,size_t width,uint32_t* ids,float* confidence,size_t capacity,char* e,size_t ec){return checked(e,ec,[&]{
 if(!h||!context||!ids||!confidence||width<2||width>8||capacity!=width-1)throw std::invalid_argument("DSpark proposal descriptor");auto& s=*static_cast<DSparkSession*>(h);const auto& committed=s.contexts.get(*context);(void)s.target->table.get(committed.target);const auto& cache=committed.cache;
 if(width>512-cache.position)throw std::invalid_argument("DSpark verifier context capacity");
 Qwen target(*static_cast<Weights*>(s.target->weights),s.target->config,true);
 native_dspark::Draft::Head head;if(!s.head_mode)head=[&](const mx::array& hidden){return target.linear(hidden,"language_model.lm_head");};
 auto result=s.draft.propose_cached(anchor,cache,width,head);
 auto scores=mx::contiguous(result.confidence,false,target.s);mx::eval(scores);std::copy(result.tokens.begin(),result.tokens.end(),ids);std::memcpy(confidence,scores.data<float>(),capacity*4);
});}
extern "C" int rs_qwen_dspark_release(void* h,const StateId* id,char* e,size_t ec){return checked(e,ec,[&]{if(!h||!id)throw std::invalid_argument("DSpark release descriptor");static_cast<DSparkSession*>(h)->contexts.release(*id);});}
extern "C" int rs_qwen_dspark_stats(void* h,uint64_t* out,size_t n,char* e,size_t ec){return checked(e,ec,[&]{if(!h||!out||n!=2)throw std::invalid_argument("DSpark stats descriptor");auto& s=*static_cast<DSparkSession*>(h);out[0]=s.contexts.residents();out[1]=s.contexts.bytes();});}
extern "C" int rs_qwen_device_dspark_snapshot(void* h,const StateId* id,float* out,size_t count,char* e,size_t ec){return checked(e,ec,[&]{
 if(!h||!id||!out)throw std::invalid_argument("DSpark feature snapshot descriptor");auto& s=*static_cast<DeviceSession*>(h);const auto& payload=s.table.get(*id);
 if(!payload.dspark_features||count!=payload.dspark_features->size())throw std::invalid_argument("DSpark feature snapshot capacity");auto packed=mx::contiguous(*payload.dspark_features,false,mx::Device::cpu);mx::eval(packed);std::memcpy(out,packed.data<float>(),count*4);
});}
