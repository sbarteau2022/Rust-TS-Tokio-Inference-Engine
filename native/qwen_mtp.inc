// Worker-confined MTP operator. No target/draft weight-map merging.
extern "C" int rs_qwen_device_mtp_attach(void* handle,void* weights,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!weights)throw std::invalid_argument("MTP attach descriptor");auto& session=*static_cast<DeviceSession*>(handle);
 if(!session.gpu||session.config.activation_precision!=1)throw std::invalid_argument("MTP profile requires qualified GPU/FP32 backend");
 if(session.mtp_weights||session.table.residents()||session.drafts.residents())throw std::invalid_argument("MTP attach must precede admission");session.mtp_weights=weights;
});}
static inline size_t mtp_kv_capacity(const rs_qwen_config& c,size_t end_pos){
 if(c.context<=512)return size_t(c.context);
 size_t active=std::max<size_t>(end_pos,1);
 size_t paged=((active+255)/256)*256;
 return std::min<size_t>(size_t(c.context),std::min<size_t>(1024,paged));
}
static size_t mtp_bytes(const rs_qwen_config& c,size_t end_pos,size_t count){
 return mtp_kv_capacity(c,end_pos)*2*2*256*4+count*2048*4;
}
static void mtp_forward_internal(DeviceSession& session,const DevicePayload* old,size_t position,const mx::array& source_hidden,size_t row,const uint32_t* tokens,size_t count,int diagnostics,DevicePayload& next,mx::array& scores){
 const auto& c=session.config;
 Qwen target(*static_cast<Weights*>(session.weights),c,session.gpu!=0),draft(*static_cast<Weights*>(session.mtp_weights),c,session.gpu!=0);
 draft.shift=false;draft.dense=true;draft.reference=diagnostics;draft.operators=diagnostics;target.reference=diagnostics;target.operators=diagnostics;
 if(!diagnostics&&session.gpu)draft.c.fused_recurrent=1;
 auto s=target.s;auto ids=mx::array(tokens,{int(count)},mx::uint32);std::string embed="language_model.model.embed_tokens";auto [group,bits]=target.quant(embed);
 auto scales=mx::astype(mx::take(target.w(embed+".scales"),ids,0,s),mx::float32,s),biases=mx::astype(mx::take(target.w(embed+".biases"),ids,0,s),mx::float32,s);
 auto token_embed=mx::reshape(mx::dequantize(mx::take(target.w(embed+".weight"),ids,0,s),scales,biases,group,bits,"affine",{},mx::float32,s),{1,int(count),c.hidden},s);
 if(diagnostics)target.check_embedding(ids,token_embed,embed,group,bits);
 auto hidden=mx::slice(source_hidden,{0,int(row),0},{1,int(row+count),c.hidden},s);
 std::string prefix="language_model.mtp";
 auto fused=mx::concatenate({draft.norm(token_embed,prefix+".pre_fc_norm_embedding.weight"),draft.norm(hidden,prefix+".pre_fc_norm_hidden.weight")},-1,s);
 auto x=draft.linear(fused,prefix+".fc");auto layer=prefix+".layers.0";
 size_t target_cap=mtp_kv_capacity(c,position+count);
 int attn_offset=int(position);
 mx::array old_k=old?old->arrays[0]:mx::zeros({1,c.kv_heads,int(target_cap),c.head_dim},mx::float32,s);
 mx::array old_v=old?old->arrays[1]:mx::zeros({1,c.kv_heads,int(target_cap),c.head_dim},mx::float32,s);
 // Sliding-window eviction at >1024 positions in 256K mode keeps MTP KV <= 4 MiB.
 if(c.context>512&&position+count>1024&&old){
  int keep=int(1024-count);
  int tail_start=std::max(0,old_k.shape(2)-keep);
  auto k_tail=mx::slice(old_k,{0,0,tail_start,0},{1,c.kv_heads,old_k.shape(2),c.head_dim},s);
  auto v_tail=mx::slice(old_v,{0,0,tail_start,0},{1,c.kv_heads,old_v.shape(2),c.head_dim},s);
  auto pad=mx::zeros({1,c.kv_heads,1024-k_tail.shape(2),c.head_dim},mx::float32,s);
  old_k=mx::concatenate({k_tail,pad},2,s);
  old_v=mx::concatenate({v_tail,pad},2,s);
  attn_offset=k_tail.shape(2);
  target_cap=1024;
 }
 auto attended=draft.attention(draft.norm(x,layer+".input_layernorm.weight"),layer+".self_attn",attn_offset,old_k,old_v,next.arrays,diagnostics!=0,int(target_cap));
 auto residual=mx::add(x,attended,s);
 if(!diagnostics&&session.gpu&&c.compiled_decode&&!std::getenv("MLX_DISABLE_COMPILE"))x=draft.compiled_post_moe(residual,layer);
 else x=mx::add(residual,draft.moe(draft.norm(residual,layer+".post_attention_layernorm.weight"),layer+".mlp"),s);
 auto normalized=draft.norm(x,prefix+".norm.weight");next.hidden=mx::contiguous(normalized,false,s);
 scores=mx::contiguous(target.linear(mx::take(normalized,int(count)-1,1,s),"language_model.lm_head"),false,s);
}
extern "C" int rs_qwen_device_mtp_step(void* handle,const StateId* previous,const StateId* source,int draft_source,size_t row,const uint32_t* tokens,size_t count,StateId* output,float* logits,size_t logits_count,int diagnostics,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!source||!tokens||!output||!logits||draft_source<0||draft_source>1||diagnostics<0||diagnostics>1||!count||count>8)throw std::invalid_argument("MTP descriptor");
 auto& session=*static_cast<DeviceSession*>(handle);const auto& c=session.config;
 if(!session.mtp_weights||logits_count!=size_t(c.vocab))throw std::invalid_argument("MTP unavailable or vocabulary capacity");
 size_t position=previous?previous->position:0;
 if(position>size_t(c.context)||count>size_t(c.context)-position)throw std::invalid_argument("MTP context capacity");
 for(size_t i=0;i<count;++i)if(tokens[i]>=uint32_t(c.vocab))throw std::invalid_argument("MTP token vocabulary");
 const DevicePayload* old=previous?&session.drafts.get(*previous):nullptr;
 const auto& input=draft_source?session.drafts.get(*source):session.table.get(*source);
 if(!input.hidden||input.hidden->ndim()!=3||input.hidden->shape(0)!=1||input.hidden->shape(2)!=c.hidden||input.hidden->dtype()!=mx::float32)throw std::invalid_argument("MTP hidden source");
 size_t rows=input.hidden->shape(1);
 if(row>rows||count>rows-row)throw std::invalid_argument("MTP hidden row capacity");
 if(draft_source){
  if(!previous||source->owner!=previous->owner||source->slot!=previous->slot||source->generation!=previous->generation||source->position!=previous->position||source->bytes!=previous->bytes||count!=1||row+1!=rows)throw std::invalid_argument("MTP autoregressive parent identity");
 }else if(source->position<rows||source->position-rows+row!=position)throw std::invalid_argument("MTP target hidden alignment");
 size_t old_expected_cap=previous?mtp_kv_capacity(c,position):0;
 if(old&&(old->arrays.size()!=2||old->arrays[0].shape()!=mx::Shape{1,c.kv_heads,int(old_expected_cap),c.head_dim}||old->arrays[1].shape()!=old->arrays[0].shape()))throw std::invalid_argument("MTP KV geometry");
 auto bytes=mtp_bytes(c,position+count,count);
 if(session.table.residents()+session.drafts.residents()>=session.slots||bytes>session.limit-session.table.bytes()-session.drafts.bytes())throw std::runtime_error("combined resident budget exhausted");
 auto stream=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));
 auto id=session.drafts.reserve(position+count,bytes);
 try {
  DevicePayload next;mx::array scores=mx::array(0.f);
  mtp_forward_internal(session,old,position,*input.hidden,row,tokens,count,diagnostics,next,scores);
  auto s=stream;
  std::vector<mx::array> evaluated={scores,*next.hidden};for(auto& a:next.arrays){if(a.dtype()!=mx::float32)a=mx::astype(a,mx::float32,s);evaluated.push_back(a);}
  const bool check_all=(diagnostics!=0||c.validation_mode>=2||(c.validation_mode==1&&(session.steps%uint64_t(c.validation_interval)==0)));
  std::vector<mx::array> finites={mx::all(mx::isfinite(scores,s),s)};
  if(check_all){
   finites.push_back(mx::all(mx::isfinite(*next.hidden,s),s));
   for(auto a:next.arrays)finites.push_back(mx::all(mx::isfinite(a,s),s));
  }
  std::vector<mx::array> all_eval=evaluated;all_eval.insert(all_eval.end(),finites.begin(),finites.end());
  mx::eval(all_eval);mx::synchronize(s);
  for(const auto& finite:finites)if(!finite.item<bool>())throw std::runtime_error("nonfinite MTP result");
  if(scores.size()!=logits_count||scores.dtype()!=mx::float32)throw std::runtime_error("MTP output geometry");
  std::memcpy(logits,scores.data<float>(),logits_count*4);session.drafts.publish(id,std::move(next));*output=id;
 }catch(...){try{mx::synchronize(stream);}catch(...){}session.drafts.rollback(id);throw;}
});}
extern "C" int rs_qwen_device_mtp_release(void* handle,const StateId* id,char* error,size_t capacity){return checked(error,capacity,[&]{if(!handle||!id)throw std::invalid_argument("MTP release descriptor");static_cast<DeviceSession*>(handle)->drafts.release(*id);});}
extern "C" int rs_qwen_device_mtp_snapshot(void* handle,const StateId* id,float* output,size_t count,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!id||!output||count!=id->bytes/4)throw std::invalid_argument("MTP snapshot capacity");auto& session=*static_cast<DeviceSession*>(handle);const auto& value=session.drafts.get(*id);size_t offset=0;
 auto s=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));
 for(const auto& a:value.arrays){if(a.size()>count-offset)throw std::invalid_argument("MTP snapshot shape");if(a.dtype()==mx::float32&&a.flags().row_contiguous){std::memcpy(output+offset,a.data<float>(),a.size()*4);}else{auto ca=mx::contiguous(mx::astype(a,mx::float32,s),false,s);mx::eval(ca);mx::synchronize(s);std::memcpy(output+offset,ca.data<float>(),ca.size()*4);}offset+=a.size();}
 if(!value.hidden||value.hidden->size()!=count-offset)throw std::invalid_argument("MTP hidden snapshot shape");std::memcpy(output+offset,value.hidden->data<float>(),value.hidden->size()*4);session.exports+=count*4;
});}

// Complete every draft payload before publication; full-validation mode must
// cover hidden and KV state just as the ordinary MTP step does.
static void mtp_transaction_eval(DeviceSession& session,DevicePayload& payload,const mx::array& scores,std::vector<mx::array> outputs){
 auto s=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));
 if(!payload.hidden||payload.arrays.size()!=2)throw std::runtime_error("MTP transaction payload inventory");
 outputs.push_back(scores);outputs.push_back(*payload.hidden);
 for(auto& a:payload.arrays){if(a.dtype()!=mx::float32)a=mx::astype(a,mx::float32,s);outputs.push_back(a);}
 std::vector<mx::array> finite={mx::all(mx::isfinite(scores,s),s)};
 const auto& c=session.config;
 if(c.validation_mode>=2||(c.validation_mode==1&&session.steps%uint64_t(c.validation_interval)==0)){
  finite.push_back(mx::all(mx::isfinite(*payload.hidden,s),s));
  for(const auto& a:payload.arrays)finite.push_back(mx::all(mx::isfinite(a,s),s));
 }
 outputs.insert(outputs.end(),finite.begin(),finite.end());mx::eval(outputs);mx::synchronize(s);
 for(const auto& a:finite)if(!a.item<bool>())throw std::runtime_error("nonfinite MTP transaction payload");
}

// Fused C++20 speculative MTP transaction: runs autoregressive draft preparation,
// target hidden-block verification, greedy accept/bonus/replay, and draft refresh
// inside a single worker call with on-GPU top-2/margin selection and immediate
// intermediate state retirement to preserve refcount==1 for in-place buffer reuse.
static int mtp_transaction(
 void* handle,const StateId* target_previous,const StateId* draft_previous,
 uint32_t seed_token,uint32_t first_draft_token,float first_draft_margin,
 size_t width,size_t output_budget,const uint32_t* eos,size_t eos_count,int reuse,
 rs_qwen_mtp_tx_result* result,float* target_logits,size_t target_logits_count,
 float* draft_logits,size_t draft_logits_count,char* error,size_t capacity,
 const uint32_t* prepared_tokens,size_t prepared_count
){return checked(error,capacity,[&]{
 if(!handle||!target_previous||!draft_previous||!result||!target_logits||!draft_logits||width<1||width>8||!output_budget||(eos_count&&!eos)||(reuse!=0&&reuse!=1))
  throw std::invalid_argument("MTP transaction descriptor");
 auto& session=*static_cast<DeviceSession*>(handle);const auto& c=session.config;
 if(!session.mtp_weights||target_logits_count!=size_t(c.vocab)||draft_logits_count!=size_t(c.vocab))
  throw std::invalid_argument("MTP transaction vocabulary capacity");
 size_t position=target_previous->position;
 if(!position||position!=draft_previous->position||position>=size_t(c.context))
  throw std::invalid_argument("MTP transaction cursor alignment");
 if(seed_token>=uint32_t(c.vocab)||first_draft_token>=uint32_t(c.vocab)||!std::isfinite(first_draft_margin))
  throw std::invalid_argument("MTP transaction seed/draft vocabulary");
 for(size_t i=0;i<eos_count;++i)if(eos[i]>=uint32_t(c.vocab))throw std::invalid_argument("MTP transaction EOS vocabulary");
 auto is_eos=[&](uint32_t tok){for(size_t i=0;i<eos_count;++i)if(eos[i]==tok)return true;return false;};
 size_t remaining=std::min(size_t(c.context)-position,output_budget);
 size_t count=std::min(width,remaining);
 if((prepared_count&&!prepared_tokens)||(!prepared_count&&prepared_tokens)||prepared_count>count)throw std::invalid_argument("MTP prepared proposal capacity");
 if(prepared_count){
  if(prepared_tokens[0]!=seed_token)throw std::invalid_argument("MTP prepared seed mismatch");
  for(size_t i=0;i<prepared_count;++i)if(prepared_tokens[i]>=uint32_t(c.vocab)||(i+1<prepared_count&&is_eos(prepared_tokens[i])))throw std::invalid_argument("MTP prepared vocabulary/EOS");
  count=prepared_count;
 }
 // Refuse over-budget work before building a graph, not after GPU completion.
 const size_t worst_bytes=qwen_state_count(c,position+count)*4+count*size_t(c.hidden)*4+mtp_bytes(c,position+count,count);
 (void)session.table.get(*target_previous);(void)session.drafts.get(*draft_previous);
 if(session.table.residents()+session.drafts.residents()+2>session.slots||worst_bytes>session.limit-session.table.bytes()-session.drafts.bytes())throw std::runtime_error("combined resident budget exhausted");
 auto stream=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));

 // Phase 1: Autoregressive draft preparation on GPU using device_top2_margin.
 std::vector<uint32_t> proposed_tokens;proposed_tokens.reserve(count);
 proposed_tokens.push_back(seed_token);
 const DevicePayload* draft_base=&session.drafts.get(*draft_previous);
 std::optional<DevicePayload> temp_draft;
 size_t draft_pos=position;
 uint32_t next_tok=first_draft_token;
 float next_margin=first_draft_margin;
 uint32_t draft_calls=0;

 if(prepared_count)proposed_tokens.assign(prepared_tokens,prepared_tokens+prepared_count);
 while(!prepared_count&&proposed_tokens.size()<count && !is_eos(proposed_tokens.back())){
  if(proposed_tokens.size()>=2 && next_margin<1.25f)break;
  proposed_tokens.push_back(next_tok);
  if(proposed_tokens.size()<count && !is_eos(next_tok) && next_margin>=1.25f){
   const DevicePayload* cur_draft=temp_draft?&(*temp_draft):draft_base;
   if(!cur_draft->hidden||cur_draft->hidden->shape(1)<1)throw std::invalid_argument("MTP transaction draft hidden empty");
   size_t row=size_t(cur_draft->hidden->shape(1)-1);
   DevicePayload step_payload;mx::array scores=mx::array(0.f);
   mtp_forward_internal(session,cur_draft,draft_pos,*cur_draft->hidden,row,&next_tok,1,0,step_payload,scores);
   auto [win_arr,margin_arr]=device_top2_margin(scores,stream);
   mtp_transaction_eval(session,step_payload,scores,{win_arr,margin_arr});
   next_tok=win_arr.item<uint32_t>();
   next_margin=margin_arr.item<float>();
   temp_draft.emplace(std::move(step_payload));
   ++draft_pos;++draft_calls;
  }else{
   break;
  }
 }
 temp_draft.reset(); // Release temporary draft chain immediately so GPU buffers can be reused.

 // Phase 2: Target block verification across `proposed_tokens`.
 size_t proposed=proposed_tokens.size();
 result->target_calls=1;
 const auto& target_old=session.table.get(*target_previous).arrays;
 size_t verify_state_count=qwen_state_count(c,position+proposed);
 DevicePayload verify_payload;
 std::vector<float> block_logits(size_t(c.vocab));
 std::vector<uint32_t> row_winners(proposed);
 {
  auto cfg=c;if(cfg.validation_mode==1)cfg.validation_mode=(session.steps%uint64_t(cfg.validation_interval)==0)?2:0;
  char inner[1024]={};
  auto code=qwen_forward_core(
   session.weights,session.gpu,&cfg,session.layers.data(),session.layers.size(),
   proposed_tokens.data(),proposed,position,nullptr,qwen_state_count(c,position),
   nullptr,verify_state_count,block_logits.data(),block_logits.size(),
   0,nullptr,0,nullptr,0,inner,sizeof(inner),
   &target_old,&verify_payload.arrays,true,&verify_payload.hidden,false,nullptr,row_winners.data()
  );
  if(code)throw std::runtime_error(inner);
  ++session.steps;
 }

 // Every row is checked finite and selected on device with the same IEEE
 // total-order/lowest-index rule as Rust, including rejected suffix rows.

 std::vector<uint32_t> emitted;emitted.reserve(proposed+1);
 size_t accepted=0;bool terminal=false;
 for(size_t i=0;i<proposed;++i){
  uint32_t target_tok=(i==0)?seed_token:row_winners[i-1];
  emitted.push_back(target_tok);
  terminal=is_eos(target_tok);
  if(proposed_tokens[i]!=target_tok||terminal){
   accepted+=size_t(proposed_tokens[i]==target_tok);
   break;
  }
  ++accepted;
  if(i+1==proposed && emitted.size()<remaining){
   uint32_t bonus=row_winners.back();
   emitted.push_back(bonus);
   terminal=is_eos(bonus);
  }
 }
 if(accepted==proposed && emitted.size()>proposed){
  emitted.resize(proposed);
  terminal=!emitted.empty()&&is_eos(emitted.back());
 }
 size_t reused_target_tokens=(reuse&&accepted==proposed)?proposed:0;

 // Phase 3: Commit or replay target state & refresh MTP draft state.
 DevicePayload committed_target;
 uint32_t target_calls=1;
 if(reused_target_tokens>0){
  std::memcpy(target_logits,block_logits.data(),size_t(c.vocab)*4);
  committed_target=std::move(verify_payload);
 }else{
  verify_payload=DevicePayload{}; // Free rejected provisional state before replay!
  ++target_calls;result->target_calls=target_calls;
  size_t replay_len=emitted.size();
  size_t replay_state_count=qwen_state_count(c,position+replay_len);
  auto cfg=c;if(cfg.validation_mode==1)cfg.validation_mode=(session.steps%uint64_t(cfg.validation_interval)==0)?2:0;
  char inner[1024]={};
  auto code=qwen_forward_core(
   session.weights,session.gpu,&cfg,session.layers.data(),session.layers.size(),
   emitted.data(),replay_len,position,nullptr,qwen_state_count(c,position),
   nullptr,replay_state_count,target_logits,size_t(c.vocab),
   0,nullptr,0,nullptr,0,inner,sizeof(inner),
   &target_old,&committed_target.arrays,false,&committed_target.hidden,false,nullptr
  );
  if(code)throw std::runtime_error(inner);
  ++session.steps;
 }

 // Winner of final target_logits for shifted MTP refresh:
 uint32_t final_winner=0;
 {
  float best_val=target_logits[0];
  for(size_t i=1;i<size_t(c.vocab);++i){
   float v=target_logits[i];
   if(v>best_val||(v==best_val&&std::signbit(v)<std::signbit(best_val))){best_val=v;final_winner=uint32_t(i);}
  }
 }
 std::vector<uint32_t> shifted;shifted.reserve(emitted.size());
 for(size_t i=1;i<emitted.size();++i)shifted.push_back(emitted[i]);
 shifted.push_back(final_winner);

 DevicePayload committed_draft;mx::array refresh_scores=mx::array(0.f);
 const DevicePayload* old_draft_for_refresh=&session.drafts.get(*draft_previous);
 mtp_forward_internal(session,old_draft_for_refresh,position,*committed_target.hidden,0,shifted.data(),shifted.size(),0,committed_draft,refresh_scores);
 ++draft_calls;
 mtp_transaction_eval(session,committed_draft,refresh_scores,{});
 std::memcpy(draft_logits,refresh_scores.data<float>(),size_t(c.vocab)*4);

 // Reserve and publish both committed target and committed draft states atomically.
 size_t final_pos=position+emitted.size();
 size_t target_bytes=qwen_state_count(c,final_pos)*4+emitted.size()*size_t(c.hidden)*4;
 size_t draft_bytes_needed=mtp_bytes(c,final_pos,shifted.size());
 if(session.table.residents()+session.drafts.residents()+2>session.slots||
    target_bytes+draft_bytes_needed>session.limit-session.table.bytes()-session.drafts.bytes())
  throw std::runtime_error("combined resident budget exhausted");
 auto new_target_id=session.table.reserve(final_pos,target_bytes);
 std::optional<StateId> new_draft_id;
 try{
  new_draft_id=session.drafts.reserve(final_pos,draft_bytes_needed);
  session.table.publish(new_target_id,std::move(committed_target));
  session.drafts.publish(*new_draft_id,std::move(committed_draft));
  result->target_state=new_target_id;
  result->draft_state=*new_draft_id;
  result->emitted_count=uint32_t(emitted.size());
  for(size_t i=0;i<emitted.size();++i)result->emitted[i]=emitted[i];
  result->learned_proposed=uint32_t(proposed>0?proposed-1:0);
  result->learned_accepted=uint32_t(accepted>0?accepted-1:0);
  result->target_calls=target_calls;
  result->draft_calls=draft_calls;
  result->reused_target_tokens=uint32_t(reused_target_tokens);
  result->replayed_target_tokens=uint32_t(emitted.size()-reused_target_tokens);
  result->target_forward_tokens=uint32_t(proposed+emitted.size()-reused_target_tokens);
  result->terminal=uint32_t(terminal?1:0);
 }catch(...){
  if(new_draft_id)session.drafts.rollback(*new_draft_id);
  session.table.rollback(new_target_id);
  throw;
 }
});}

extern "C" int rs_qwen_device_mtp_transaction(void* handle,const StateId* target_previous,const StateId* draft_previous,uint32_t seed_token,uint32_t first_draft_token,float first_draft_margin,size_t width,size_t output_budget,const uint32_t* eos,size_t eos_count,int reuse,rs_qwen_mtp_tx_result* result,float* target_logits,size_t target_logits_count,float* draft_logits,size_t draft_logits_count,char* error,size_t capacity){return mtp_transaction(handle,target_previous,draft_previous,seed_token,first_draft_token,first_draft_margin,width,output_budget,eos,eos_count,reuse,result,target_logits,target_logits_count,draft_logits,draft_logits_count,error,capacity,nullptr,0);}

// Controller-bound proposal tokens are immutable inputs; no second draft pass.
extern "C" int rs_qwen_device_mtp_transaction_prepared(void* handle,const StateId* target_previous,const StateId* draft_previous,uint32_t seed_token,uint32_t first_draft_token,float first_draft_margin,size_t width,size_t output_budget,const uint32_t* eos,size_t eos_count,int reuse,rs_qwen_mtp_tx_result* result,float* target_logits,size_t target_logits_count,float* draft_logits,size_t draft_logits_count,char* error,size_t capacity,const uint32_t* prepared_tokens,size_t prepared_count){return mtp_transaction(handle,target_previous,draft_previous,seed_token,first_draft_token,first_draft_margin,width,output_budget,eos,eos_count,reuse,result,target_logits,target_logits_count,draft_logits,draft_logits_count,error,capacity,prepared_tokens,prepared_count);}
