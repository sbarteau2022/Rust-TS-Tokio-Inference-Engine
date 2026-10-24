// Worker-confined MTP operator. No target/draft weight-map merging.
extern "C" int rs_qwen_device_mtp_attach(void* handle,void* weights,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!weights)throw std::invalid_argument("MTP attach descriptor");auto& session=*static_cast<DeviceSession*>(handle);
 if(!session.gpu)throw std::invalid_argument("MTP profile requires qualified GPU backend");
 if(session.mtp_weights||session.table.residents()||session.drafts.residents())throw std::invalid_argument("MTP attach must precede admission");session.mtp_weights=weights;
});}
static size_t mtp_bytes(size_t position,size_t count){return position*2*2*256*4+count*2048*4;}
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
 if(old&&(old->arrays.size()!=2||old->arrays[0].shape()!=mx::Shape{1,c.kv_heads,int(position),c.head_dim}||old->arrays[1].shape()!=old->arrays[0].shape()))throw std::invalid_argument("MTP KV geometry");
 auto bytes=mtp_bytes(position+count,count);
 if(session.table.residents()+session.drafts.residents()>=session.slots||bytes>session.limit-session.table.bytes()-session.drafts.bytes())throw std::runtime_error("combined resident budget exhausted");
 auto stream=mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu));
 auto id=session.drafts.reserve(position+count,bytes);
 try {
  Qwen target(*static_cast<Weights*>(session.weights),c,session.gpu!=0),draft(*static_cast<Weights*>(session.mtp_weights),c,session.gpu!=0);
  draft.shift=false;draft.dense=true;draft.reference=diagnostics;draft.operators=diagnostics;target.reference=diagnostics;target.operators=diagnostics; // Sidecar is already sanitized; never infer shifts from key presence.
  if(!diagnostics&&session.gpu)draft.c.fused_recurrent=1;
  auto s=target.s;auto ids=mx::array(tokens,{int(count)},mx::uint32);std::string embed="language_model.model.embed_tokens";auto [group,bits]=target.quant(embed);
  auto scales=mx::astype(mx::take(target.w(embed+".scales"),ids,0,s),mx::float32,s),biases=mx::astype(mx::take(target.w(embed+".biases"),ids,0,s),mx::float32,s);
  auto token_embed=mx::reshape(mx::dequantize(mx::take(target.w(embed+".weight"),ids,0,s),scales,biases,group,bits,"affine",{},mx::float32,s),{1,int(count),c.hidden},s);
  if(diagnostics)target.check_embedding(ids,token_embed,embed,group,bits);
  auto hidden=mx::slice(*input.hidden,{0,int(row),0},{1,int(row+count),c.hidden},s);
  std::string prefix="language_model.mtp";
  auto fused=mx::concatenate({draft.norm(token_embed,prefix+".pre_fc_norm_embedding.weight"),draft.norm(hidden,prefix+".pre_fc_norm_hidden.weight")},-1,s);
  auto x=draft.linear(fused,prefix+".fc");auto layer=prefix+".layers.0";
  auto empty=mx::zeros({1,c.kv_heads,0,c.head_dim},mx::float32,s);DevicePayload next;
  auto attended=draft.attention(draft.norm(x,layer+".input_layernorm.weight"),layer+".self_attn",int(position),old?old->arrays[0]:empty,old?old->arrays[1]:empty,next.arrays,diagnostics!=0);
  auto residual=mx::add(x,attended,s);
  if(!diagnostics&&session.gpu&&c.compiled_decode&&!std::getenv("MLX_DISABLE_COMPILE"))x=draft.compiled_post_moe(residual,layer);
  else x=mx::add(residual,draft.moe(draft.norm(residual,layer+".post_attention_layernorm.weight"),layer+".mlp"),s);
  auto normalized=draft.norm(x,prefix+".norm.weight");next.hidden=mx::contiguous(normalized,false,s);
  auto scores=mx::contiguous(target.linear(mx::take(normalized,int(count)-1,1,s),"language_model.lm_head"),false,s);
  std::vector<mx::array> evaluated={scores,*next.hidden};for(auto& a:next.arrays){if(a.dtype()!=mx::float32)a=mx::astype(a,mx::float32,s);evaluated.push_back(a);}
  std::vector<mx::array> finites={mx::all(mx::isfinite(scores,s),s),mx::all(mx::isfinite(*next.hidden,s),s)};
  for(auto a:next.arrays)finites.push_back(mx::all(mx::isfinite(a,s),s));
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
