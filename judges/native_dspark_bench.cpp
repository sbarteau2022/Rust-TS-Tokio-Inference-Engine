#define main cpp_control_unused_main
#include "native_cpp_bench.cpp"
#undef main
#include "dspark_tuning.h"
struct DSState {
 void* engine;std::optional<StateId> id;
 explicit DSState(void* e):engine(e){}
 ~DSState(){if(id)rs_qwen_dspark_release(engine,&*id,error,sizeof(error));}
 const StateId* ptr()const{return id?&*id:nullptr;}
 void replace(StateId next){auto old=id;id=next;if(old)check(rs_qwen_dspark_release(engine,&*old,error,sizeof(error)));}
};
int main(int argc,char** argv){try{
 require(argc>=4,"native_dspark_bench PLAN DRAFTER NEW_REPORT [bf16|checkpoint-head|checkpoint-bf16] [--width=2|4|8] [--confidence=none|0.8|0.9|0.95|0.98] [--adaptive]");
 std::vector<std::string> args;for(int i=4;i<argc;++i)args.emplace_back(argv[i]);auto tuning=native_dspark::Tuning::parse(args);
#ifdef RS_DSPARK_CONFIDENCE
 if(!tuning.confidence_explicit)tuning.confidence=.98f;
#endif
 int head_mode=tuning.head;require(!std::filesystem::exists(argv[3]),"report exists");auto plan=read_plan(argv[1]);
 Handle weights(rs_mlx_weights_create(error,sizeof(error)),rs_mlx_weights_destroy);require(bool(weights),error);for(const auto& path:plan.shards)check(rs_mlx_weights_load(weights.get(),path.c_str(),error,sizeof(error)));for(const auto& q:plan.overrides)check(rs_qwen_quant(weights.get(),q.name.c_str(),q.group,q.bits,error,sizeof(error)));
 rs_qwen_config cfg={2048,40,248320,16,2,256,16,32,128,128,4,4,256,8,512,512,64,4,512,8,64,1,40,1,0,0,0,0,2,32,0,1e-6f,10000000.f};rs_qwen_layer layers[40];for(unsigned i=0;i<40;++i)layers[i]={i,uint32_t((i+1)%4==0)};
 Handle target(rs_qwen_device_create(weights.get(),1,&cfg,layers,40,16,1536ull*1024*1024,error,sizeof(error)),rs_qwen_device_destroy);require(bool(target),error);
 Handle draft(rs_qwen_dspark_create(target.get(),argv[2],head_mode,8,128ull*1024*1024,error,sizeof(error)),rs_qwen_dspark_destroy);require(bool(draft),error);
 // Independent trace layout verifies zero-based post-layer taps and charging.
 {
  auto ids=plan.prompts[0].input;ids.resize(std::min<size_t>(8,ids.size()));size_t n=ids.size();std::vector<float> logits(248320),trace(40*5*n*2048),features(n*16384);
  State reference(target.get(),false),captured(target.get(),false);StateId r{},c{};
  check(rs_qwen_device_step(target.get(),nullptr,0,ids.data(),n,&r,logits.data(),logits.size(),trace.data(),trace.size(),error,sizeof(error)));reference.id=r;
  check(rs_qwen_device_dspark_step(target.get(),nullptr,0,ids.data(),n,&c,logits.data(),logits.size(),0,error,sizeof(error)));captured.id=c;
  require(c.bytes-r.bytes==features.size()*4,"feature state byte charge");check(rs_qwen_device_dspark_snapshot(target.get(),captured.ptr(),features.data(),features.size(),error,sizeof(error)));
  int taps[]={1,6,11,16,22,27,32,37};double square=0,energy=0,maximum=0;
  for(size_t row=0;row<n;++row)for(size_t tap=0;tap<8;++tap)for(size_t d=0;d<2048;++d){double expected=trace[(taps[tap]*5+4)*n*2048+row*2048+d],actual=features[row*16384+tap*2048+d];require(std::isfinite(actual)&&std::isfinite(expected),"tap nonfinite");double delta=actual-expected;square+=delta*delta;energy+=expected*expected;maximum=std::max(maximum,std::abs(delta));}
  require(std::sqrt(square/std::max(energy,1e-30))<=1e-5&&maximum<=1e-3,"tap trace mismatch");
  StateId first{};check(rs_qwen_dspark_ingest(draft.get(),captured.ptr(),nullptr,&first,error,sizeof(error)));DSState context(draft.get());context.id=first;
  StateId sentinel{9,9,9,9,9};require(rs_qwen_dspark_ingest(draft.get(),captured.ptr(),context.ptr(),&sentinel,error,sizeof(error))!=0&&sentinel.owner==9,"reingest rollback");
  uint64_t stats[2]{};check(rs_qwen_dspark_stats(draft.get(),stats,2,error,sizeof(error)));require(stats[0]==1&&stats[1]==n*6*2*8*128*4,"failed ingest altered cache budget");
  StateId fake=first;++fake.owner;std::vector<uint32_t> output(3,999);std::vector<float> confidence(3,999);
  require(rs_qwen_dspark_propose(draft.get(),&fake,1,4,output.data(),confidence.data(),3,error,sizeof(error))!=0&&output[0]==999,"foreign context accepted");
  {
   State child_a(target.get(),false),child_b(target.get(),false),grandchild(target.get(),false);StateId a{},b{},g{};uint32_t token_a=100,token_b=101;
   check(rs_qwen_device_dspark_step(target.get(),captured.ptr(),captured.position(),&token_a,1,&a,logits.data(),logits.size(),0,error,sizeof(error)));child_a.id=a;
   check(rs_qwen_device_dspark_step(target.get(),captured.ptr(),captured.position(),&token_b,1,&b,logits.data(),logits.size(),0,error,sizeof(error)));child_b.id=b;
   DSState branch(draft.get());StateId branch_id{};check(rs_qwen_dspark_ingest(draft.get(),child_a.ptr(),context.ptr(),&branch_id,error,sizeof(error)));branch.id=branch_id;
   check(rs_qwen_device_dspark_step(target.get(),child_b.ptr(),child_b.position(),&token_b,1,&g,logits.data(),logits.size(),0,error,sizeof(error)));grandchild.id=g;
   require(rs_qwen_dspark_ingest(draft.get(),grandchild.ptr(),branch.ptr(),&sentinel,error,sizeof(error))!=0&&sentinel.owner==9,"cross-branch context accepted");
   std::vector<StateId> clones;for(size_t i=0;i<6;++i){StateId clone{};check(rs_qwen_dspark_ingest(draft.get(),child_a.ptr(),context.ptr(),&clone,error,sizeof(error)));clones.push_back(clone);}
   uint64_t before[2]{},after[2]{};check(rs_qwen_dspark_stats(draft.get(),before,2,error,sizeof(error)));require(before[0]==8,"pressure slot setup");
   require(rs_qwen_dspark_ingest(draft.get(),child_a.ptr(),context.ptr(),&sentinel,error,sizeof(error))!=0&&sentinel.owner==9,"slot exhaustion accepted");check(rs_qwen_dspark_stats(draft.get(),after,2,error,sizeof(error)));require(before[0]==after[0]&&before[1]==after[1],"failed reserve changed budget");
   for(auto clone:clones)check(rs_qwen_dspark_release(draft.get(),&clone,error,sizeof(error)));
  }
  captured.reset();require(rs_qwen_dspark_propose(draft.get(),context.ptr(),1,4,output.data(),confidence.data(),3,error,sizeof(error))!=0&&output[0]==999,"dead target context accepted");
 }
 size_t generated_total=0,fallback_total=0,proposed_total=0,accepted_total=0,tokens_total=0;double elapsed_total=0,proposal_seconds=0;std::ostringstream rows;
 for(size_t prompt=0;prompt<plan.prompts.size();++prompt){State state(target.get(),false);DSState context(draft.get());std::vector<float> logits(248320);
  const auto& fixture=plan.prompts[prompt];
  for(size_t offset=0;offset<fixture.input.size();offset+=8){size_t n=std::min<size_t>(8,fixture.input.size()-offset);State pending(target.get(),false);StateId next{},cache{};check(rs_qwen_device_dspark_step(target.get(),state.ptr(),offset,fixture.input.data()+offset,n,&next,logits.data(),logits.size(),0,error,sizeof(error)));pending.id=next;check(rs_qwen_dspark_ingest(draft.get(),pending.ptr(),context.ptr(),&cache,error,sizeof(error)));context.replace(cache);state.replace(next);pending.id.reset();}
  std::vector<uint32_t> output;size_t window=tuning.initial(),proposed_count=0,accepted_count=0;auto start=Clock::now();
  while(output.size()<32){size_t width=std::min<size_t>(window,32-output.size());uint32_t seed=winner(logits);std::vector<uint32_t> proposed{seed};
   if(width>1){std::vector<uint32_t> ids(width-1);std::vector<float> confidence(width-1);auto proposal_start=Clock::now();check(rs_qwen_dspark_propose(draft.get(),context.ptr(),seed,width,ids.data(),confidence.data(),ids.size(),error,sizeof(error)));proposal_seconds+=std::chrono::duration<double>(Clock::now()-proposal_start).count();for(float score:confidence)require(std::isfinite(score)&&score>=0&&score<=1,"confidence bounds");generated_total+=ids.size();
    size_t admitted=0;while(admitted<ids.size()&&(tuning.confidence<0||confidence[admitted]>=tuning.confidence))++admitted;
    proposed.insert(proposed.end(),ids.begin(),ids.begin()+admitted);if(!admitted)++fallback_total;
    width=proposed.size();}
   State verified(target.get(),false);StateId verified_id{};std::vector<float> block(width*248320);check(rs_qwen_device_dspark_step(target.get(),state.ptr(),state.position(),proposed.data(),width,&verified_id,block.data(),block.size(),1,error,sizeof(error)));verified.id=verified_id;
   std::vector<uint32_t> emitted;size_t accepted=0;std::vector<float> row(248320);
   for(size_t i=0;i<width;++i){uint32_t authoritative=seed;if(i){std::copy_n(block.data()+(i-1)*248320,248320,row.data());authoritative=winner(row);}emitted.push_back(authoritative);if(authoritative!=proposed[i])break;++accepted;}
   proposed_count+=width-1;accepted_count+=accepted?accepted-1:0;window=tuning.next(window,width-1,accepted?accepted-1:0);
   State committed(target.get(),false);if(accepted==width){committed.id=verified.id;verified.id.reset();std::copy_n(block.data()+(width-1)*248320,248320,logits.data());}
   else{verified.reset();StateId replay{};check(rs_qwen_device_dspark_step(target.get(),state.ptr(),state.position(),emitted.data(),emitted.size(),&replay,logits.data(),logits.size(),0,error,sizeof(error)));committed.id=replay;}
   StateId next_context{};check(rs_qwen_dspark_ingest(draft.get(),committed.ptr(),context.ptr(),&next_context,error,sizeof(error)));context.replace(next_context);state.replace(*committed.id);committed.id.reset();output.insert(output.end(),emitted.begin(),emitted.end());require(std::equal(output.begin(),output.end(),fixture.reference.begin()),"DSpark frozen output mismatch");
  }
  double seconds=std::chrono::duration<double>(Clock::now()-start).count();proposed_total+=proposed_count;accepted_total+=accepted_count;tokens_total+=output.size();elapsed_total+=seconds;
  if(prompt)rows<<',';rows<<"{\"prompt\":\""<<fixture.name<<"\",\"tokens\":32,\"seconds\":"<<seconds<<",\"proposed\":"<<proposed_count<<",\"accepted\":"<<accepted_count<<",\"ids\":[";for(size_t i=0;i<output.size();++i){if(i)rows<<',';rows<<output[i];}rows<<"]}";
 }
 uint64_t ds[2]{},stats[74]{};check(rs_qwen_dspark_stats(draft.get(),ds,2,error,sizeof(error)));check(rs_qwen_device_stats(target.get(),stats,74,error,sizeof(error)));require(ds[0]==0&&ds[1]==0&&stats[0]==0&&stats[1]==0,"DSpark final residency");
 std::ostringstream out;out<<std::setprecision(17)<<"{\"pass\":true,\"identity\":\""<<plan.identity<<"\",\"dspark_execution\":true,\"tuning\":{\"width\":"<<tuning.width<<",\"confidence\":"<<tuning.confidence<<",\"adaptive\":"<<(tuning.adaptive?"true":"false")<<"},\"proposal_seconds\":"<<proposal_seconds<<",\"draft_precision\":\""<<(head_mode<2?"fp32":"bf16")<<"\",\"combined_head\":"<<((head_mode&1)?"false":"true")<<",\"checkpoint_head\":"<<((head_mode&1)?"true":"false")<<",\"turboquant_execution\":false,\"production_qualified\":false,\"http_streaming\":false,\"tokens\":"<<tokens_total<<",\"seconds\":"<<elapsed_total<<",\"decode_tok_s\":"<<tokens_total/elapsed_total<<",\"generated_proposals\":"<<generated_total<<",\"single_token_fallbacks\":"<<fallback_total<<",\"learned_proposed\":"<<proposed_total<<",\"learned_accepted\":"<<accepted_total<<",\"target_taps_trace_checked\":true,\"branch_and_budget_pressure\":true,\"frozen_outputs_exact\":true,\"zero_drain\":true,\"rows\":["<<rows.str()<<"]}\n";publish_report(argv[3],out.str());draft.reset();target.reset();weights.reset();rs_mlx_cleanup();return 0;
 }catch(const std::exception& e){std::cerr<<"FAIL "<<e.what()<<'\n';rs_mlx_cleanup();return 1;}}
