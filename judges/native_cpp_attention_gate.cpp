// Independent mode-0 versus mode-2 full target/draft payload gate. Not throughput.
#define main cpp_control_unused_main
#include "native_cpp_bench.cpp"
#undef main
#include <cstring>
namespace {
struct Error {double relative=0,maximum=0;};
Error compare(const std::vector<float>& actual,const std::vector<float>& reference){
 require(actual.size()==reference.size()&&!actual.empty(),"gate vector geometry");double square=0,energy=0,maximum=0;
 for(size_t i=0;i<actual.size();++i){require(std::isfinite(actual[i])&&std::isfinite(reference[i]),"gate nonfinite");double delta=double(actual[i])-reference[i];square+=delta*delta;energy+=double(reference[i])*reference[i];maximum=std::max(maximum,std::abs(delta));}
 Error e{std::sqrt(square/std::max(energy,1e-30)),maximum};require(e.relative<=1e-5&&e.maximum<=1e-3,"full payload numerical gate: relative="+std::to_string(e.relative)+" max="+std::to_string(e.maximum));return e;
}
}
int main(int argc,char** argv){try{
 require(argc==3,"native_cpp_attention_gate PLAN NEW_REPORT.json");require(!std::filesystem::exists(argv[2]),"report exists");auto plan=read_plan(argv[1]);
 Handle weights(rs_mlx_weights_create(error,sizeof(error)),rs_mlx_weights_destroy);require(bool(weights),error);for(const auto& path:plan.shards)check(rs_mlx_weights_load(weights.get(),path.c_str(),error,sizeof(error)));
 Handle mtp(rs_mlx_weights_create(error,sizeof(error)),rs_mlx_weights_destroy);require(bool(mtp),error);check(rs_mlx_weights_load(mtp.get(),plan.mtp.c_str(),error,sizeof(error)));
 for(const auto& q:plan.overrides){check(rs_qwen_quant(weights.get(),q.name.c_str(),q.group,q.bits,error,sizeof(error)));if(q.name.starts_with("language_model.mtp."))check(rs_qwen_quant(mtp.get(),q.name.c_str(),q.group,q.bits,error,sizeof(error)));}
 rs_qwen_config cfg={2048,40,248320,16,2,256,16,32,128,128,4,4,256,8,512,512,64,4,512,8,64,1,40,1,0,0,0,0,2,32,0,1e-6f,10000000.f};rs_qwen_layer layers[40];for(unsigned i=0;i<40;++i)layers[i]={i,uint32_t((i+1)%4==0)};
 Handle reference(rs_qwen_device_create(weights.get(),1,&cfg,layers,40,16,1536ull*1024*1024,error,sizeof(error)),rs_qwen_device_destroy);require(bool(reference),error);cfg.fused_attention=2;
 Handle candidate(rs_qwen_device_create(weights.get(),1,&cfg,layers,40,16,1536ull*1024*1024,error,sizeof(error)),rs_qwen_device_destroy);require(bool(candidate),error);
 check(rs_qwen_device_mtp_attach(reference.get(),mtp.get(),error,sizeof(error)));check(rs_qwen_device_mtp_attach(candidate.get(),mtp.get(),error,sizeof(error)));
 size_t cases=0;double max_relative=0,max_absolute=0;
 for(size_t width:{1,2,4,8}){
  State rt(reference.get(),false),ct(candidate.get(),false),rd(reference.get(),true),cd(candidate.get(),true);std::vector<float> rl(248320),cl(248320),rld(248320),cld(248320);
  for(size_t iteration=0;iteration<(width==8?64:4);++iteration){
   std::vector<uint32_t> ids;for(size_t i=0;i<width;++i)ids.push_back(plan.prompts[0].input[(iteration*width+i)%plan.prompts[0].input.size()]);
   StateId rnext{},cnext{};size_t position=rt.position();
   std::vector<float> old_r,old_c;
   if(rt.id){size_t size=(rt.id->bytes-width*2048*4)/4;old_r.resize(size);old_c.resize(size);check(rs_qwen_device_snapshot(reference.get(),rt.ptr(),old_r.data(),size,error,sizeof(error)));check(rs_qwen_device_snapshot(candidate.get(),ct.ptr(),old_c.data(),size,error,sizeof(error)));}
   State pending_r(reference.get(),false),pending_c(candidate.get(),false);
   check(rs_qwen_device_hidden_step(reference.get(),rt.ptr(),position,ids.data(),width,&rnext,rl.data(),rl.size(),error,sizeof(error)));pending_r.id=rnext;
   check(rs_qwen_device_hidden_step(candidate.get(),ct.ptr(),position,ids.data(),width,&cnext,cl.data(),cl.size(),error,sizeof(error)));pending_c.id=cnext;
   if(rt.id){std::vector<float> after(old_r.size());check(rs_qwen_device_snapshot(reference.get(),rt.ptr(),after.data(),after.size(),error,sizeof(error)));require(std::memcmp(after.data(),old_r.data(),after.size()*4)==0,"reference parent mutation");check(rs_qwen_device_snapshot(candidate.get(),ct.ptr(),after.data(),after.size(),error,sizeof(error)));require(std::memcmp(after.data(),old_c.data(),after.size()*4)==0,"candidate parent mutation");}
   pending_r.id.reset();rt.replace(rnext);pending_c.id.reset();ct.replace(cnext);
   require(winner(rl)==winner(cl),"target selected token drift");
   size_t elements=(rnext.bytes-width*2048*4)/4;require(elements==(cnext.bytes-width*2048*4)/4,"target payload geometry");std::vector<float> a(elements),b(elements);
   check(rs_qwen_device_snapshot(reference.get(),rt.ptr(),a.data(),a.size(),error,sizeof(error)));check(rs_qwen_device_snapshot(candidate.get(),ct.ptr(),b.data(),b.size(),error,sizeof(error)));
   auto accumulate=[&](Error e){max_relative=std::max(max_relative,e.relative);max_absolute=std::max(max_absolute,e.maximum);};accumulate(compare(b,a));accumulate(compare(cl,rl));
   a.resize(width*2048);b.resize(width*2048);check(rs_qwen_device_hidden_snapshot(reference.get(),rt.ptr(),a.data(),a.size(),error,sizeof(error)));check(rs_qwen_device_hidden_snapshot(candidate.get(),ct.ptr(),b.data(),b.size(),error,sizeof(error)));accumulate(compare(b,a));
   std::vector<uint32_t> shifted(ids.begin()+1,ids.end());shifted.push_back(winner(rl));StateId rn{},cn{};
   check(rs_qwen_device_mtp_step(reference.get(),rd.ptr(),rt.ptr(),0,0,shifted.data(),width,&rn,rld.data(),rld.size(),0,error,sizeof(error)));rd.replace(rn);
   check(rs_qwen_device_mtp_step(candidate.get(),cd.ptr(),ct.ptr(),0,0,shifted.data(),width,&cn,cld.data(),cld.size(),0,error,sizeof(error)));cd.replace(cn);
   require(winner(rld)==winner(cld),"draft selected token drift");require(rn.bytes==cn.bytes,"draft payload geometry");a.resize(rn.bytes/4);b.resize(cn.bytes/4);
   check(rs_qwen_device_mtp_snapshot(reference.get(),rd.ptr(),a.data(),a.size(),error,sizeof(error)));check(rs_qwen_device_mtp_snapshot(candidate.get(),cd.ptr(),b.data(),b.size(),error,sizeof(error)));accumulate(compare(b,a));accumulate(compare(cld,rld));++cases;
  }
 }
 for(void* session:{reference.get(),candidate.get()}){uint64_t stats[74]{};check(rs_qwen_device_stats(session,stats,74,error,sizeof(error)));require(stats[0]==0&&stats[1]==0,"gate final residency");}
 std::ostringstream out;out<<std::setprecision(17)<<"{\"pass\":true,\"cases\":"<<cases<<",\"max_relative_l2\":"<<max_relative<<",\"max_absolute\":"<<max_absolute<<",\"target_logits_state_hidden\":true,\"context_boundary\":512,\"immutable_parents\":true,\"draft_logits_kv_hidden\":true,\"zero_drain\":true,\"not_throughput\":true}\n";require(bool(out),"gate report serialization");publish_report(argv[2],out.str());candidate.reset();reference.reset();mtp.reset();weights.reset();rs_mlx_cleanup();return 0;
 }catch(const std::exception& e){std::cerr<<"FAIL "<<e.what()<<'\n';rs_mlx_cleanup();return 1;}}
