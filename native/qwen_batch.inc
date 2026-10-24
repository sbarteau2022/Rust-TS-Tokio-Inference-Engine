// Equal-position one-token target decode. Explicit prototype, never selected implicitly.
extern "C" int rs_qwen_device_decode_batch(void* handle,const StateId* parents,const uint32_t* tokens,size_t batch,StateId* outputs,float* logits,size_t logits_count,char* error,size_t capacity){return checked(error,capacity,[&]{
 if(!handle||!parents||!tokens||!outputs||!logits||batch<1||batch>3)throw std::invalid_argument("batch descriptor");
 auto& session=*static_cast<DeviceSession*>(handle);const auto& c=session.config;size_t position=parents[0].position;
 if(!position||position>=size_t(c.context)||logits_count!=batch*size_t(c.vocab))throw std::invalid_argument("batch geometry");
 std::vector<const DevicePayload*> old;for(size_t row=0;row<batch;++row){
  if(parents[row].position!=position||tokens[row]>=uint32_t(c.vocab))throw std::invalid_argument("batch parent/token geometry");
  const auto& p=session.table.get(parents[row]);if(p.arrays.size()!=80)throw std::invalid_argument("batch parent inventory");old.push_back(&p);
 }
 const uint64_t charge=batch*qwen_state_count(c,position+1)*4;
 auto residents=session.table.residents()+session.drafts.residents();auto used=session.table.bytes()+session.drafts.bytes();
 if(residents+batch>session.slots||batch*charge>session.limit-used)throw std::runtime_error("batch resident budget exhausted");
 std::vector<StateId> reserved;
 try{
  for(size_t row=0;row<batch;++row)reserved.push_back(session.table.reserve(position+1,charge));
  if(c.operator_profile)throw std::invalid_argument("operator profile refuses batch graph");
  ForwardTiming timer(static_cast<Weights*>(session.weights)->timing,c.execution_timing);
  Qwen m(*static_cast<Weights*>(session.weights),c,session.gpu!=0);auto s=m.s;
  auto ids=mx::array(tokens,{int(batch)},mx::uint32);std::string embed="language_model.model.embed_tokens";auto [group,bits]=m.quant(embed);
  auto scales=mx::astype(mx::take(m.w(embed+".scales"),ids,0,s),mx::float32,s),biases=mx::astype(mx::take(m.w(embed+".biases"),ids,0,s),mx::float32,s);
  auto x=mx::reshape(mx::dequantize(mx::take(m.w(embed+".weight"),ids,0,s),scales,biases,group,bits,"affine",{},mx::float32,s),{int(batch),1,c.hidden},s);
  std::vector<mx::array> packed,pending;size_t array=0;
  for(int layer=0;layer<c.layers;++layer){
   std::vector<mx::array> pair;
   for(int slot=0;slot<2;++slot){std::vector<mx::array> rows;for(auto p:old)rows.push_back(p->arrays[array]);pair.push_back(mx::concatenate(rows,0,s));++array;}
   std::string prefix="language_model.model.layers."+std::to_string(layer);std::vector<mx::array> next;
   if(c.compiled_decode&&session.layers[layer].mixer==0){auto result=m.compiled_recurrent(x,prefix,pair[0],pair[1]);x=result[0];next={result[1],result[2]};}
   else{
   auto h=m.norm(x,prefix+".input_layernorm.weight");auto r=session.layers[layer].mixer==0?m.recurrent(h,prefix+".linear_attn",pair[0],pair[1],next):m.attention(h,prefix+".self_attn",int(position),pair[0],pair[1],next);
   auto residual=mx::add(x,r,s);
   if(c.compiled_decode&&session.layers[layer].mixer==1)x=m.compiled_post_moe(residual,prefix);
   else x=mx::add(residual,m.moe(m.norm(residual,prefix+".post_attention_layernorm.weight"),prefix+".mlp"),s);
   }
   std::vector<mx::array> evaluated={x};for(auto a:next)evaluated.push_back(a.dtype()==mx::float32?a:mx::astype(a,mx::float32,s));pending.insert(pending.end(),evaluated.begin(),evaluated.end());
   if(((layer+1)%c.layer_group==0||layer+1==c.layers)&&(layer+1<c.layers)){timer.graph();mx::eval(pending);mx::synchronize(s);timer.wait();pending.clear();}x=evaluated[0];
   for(size_t i=1;i<evaluated.size();++i)packed.push_back(evaluated[i]);
  }
  auto normalized=m.norm(x,"language_model.model.norm.weight");auto output=mx::contiguous(mx::astype(m.linear(mx::take(normalized,0,1,s),"language_model.lm_head"),mx::float32,s),false,s);
  if(packed.size()!=80||output.size()!=logits_count)throw std::runtime_error("batch result inventory");
  std::vector<DevicePayload> children;for(size_t row=0;row<batch;++row){DevicePayload child;for(auto a:packed){auto start=mx::Shape(a.ndim(),0),end=a.shape();start[0]=int(row);end[0]=int(row+1);child.arrays.push_back(mx::contiguous(mx::slice(a,start,end,s),false,s));}children.push_back(std::move(child));}
  std::vector<mx::array> finite={mx::all(mx::isfinite(output,s),s)};for(auto a:packed)finite.push_back(mx::all(mx::isfinite(a,s),s));
  std::vector<mx::array> evaluated=std::move(pending);evaluated.push_back(output);evaluated.insert(evaluated.end(),finite.begin(),finite.end());
  for(auto& child:children)evaluated.insert(evaluated.end(),child.arrays.begin(),child.arrays.end());
  timer.graph();mx::eval(evaluated);mx::synchronize(s);timer.wait();
  for(auto a:finite)if(!a.item<bool>())throw std::runtime_error("nonfinite batch result");
  for(size_t row=0;row<batch;++row)session.table.publish(reserved[row],std::move(children[row]));
  std::memcpy(logits,output.data<float>(),logits_count*4);for(size_t row=0;row<batch;++row)outputs[row]=reserved[row];session.steps+=batch;timer.complete();
 }catch(...){try{mx::synchronize(mx::default_stream(mx::Device(session.gpu?mx::Device::gpu:mx::Device::cpu)));}catch(...){}for(auto id:reserved)session.table.rollback(id);throw;}
});}
