// Direct control of the existing native kernel; not the user's separate fork.
#include "mlx_bridge.h"
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <filesystem>
#include <sstream>
#include <fcntl.h>
#include <unistd.h>
#include <cerrno>
#include <iomanip>
#include <iostream>
#include <limits>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <vector>
namespace {
char error[1024]{};
using Clock=std::chrono::steady_clock;
void require(bool ok,const std::string& text){if(!ok)throw std::runtime_error(text);}
void check(int code){require(code==0,error);}
using Handle=std::unique_ptr<void,void(*)(void*)>;
struct Override {std::string name;int group,bits;};
struct Prompt {std::string name;std::vector<uint32_t> input,reference;};
struct Plan {std::string identity;int group=0,bits=0;std::vector<std::string> shards;std::string mtp;std::vector<Override> overrides;std::vector<uint32_t> eos;std::vector<Prompt> prompts;};
size_t bounded(std::istream& in,size_t low,size_t high){uint64_t n=0;in>>n;require(bool(in)&&n>=low&&n<=high,"plan count bounds");return size_t(n);}
std::string text(std::istream& in){std::string value;in>>std::quoted(value);require(bool(in)&&!value.empty()&&value.size()<=4096&&value.find_first_of("\r\n\t")==std::string::npos,"plan text bounds");return value;}
std::vector<uint32_t> tokens(std::istream& in,size_t count){std::vector<uint32_t> result;for(size_t i=0;i<count;++i)result.push_back(uint32_t(bounded(in,0,248319)));return result;}
Plan read_plan(const char* path){
 require(std::filesystem::file_size(path)<=262144,"plan file size bound");std::ifstream in(path);require(bool(in),"cannot open plan");require(text(in)=="RS_CPP_BENCH_V1","plan version");Plan p;p.identity=text(in);
 require(p.identity.size()==64&&p.identity.find_first_not_of("0123456789abcdef")==std::string::npos,"plan identity");
 in>>p.group>>p.bits;require(bool(in)&&p.group==64&&p.bits==4,"qualified target quantization");
 size_t shards=bounded(in,4,4);for(size_t i=0;i<shards;++i){auto path=text(in);require(path.front()=='/',"absolute shard path");p.shards.push_back(path);}
 p.mtp=text(in);require(p.mtp.front()=='/',"absolute sidecar path");
 size_t n=bounded(in,0,4096);for(size_t i=0;i<n;++i){Override q;q.name=text(in);in>>q.group>>q.bits;require(bool(in)&&q.name.starts_with("language_model.")&&(q.group==32||q.group==64||q.group==128)&&(q.bits==2||q.bits==3||q.bits==4||q.bits==5||q.bits==6||q.bits==8),"quantization override");p.overrides.push_back(q);}
 p.eos=tokens(in,bounded(in,1,8));n=bounded(in,4,4);for(size_t i=0;i<n;++i){Prompt prompt;prompt.name=text(in);require(prompt.name.find_first_not_of("abcdefghijklmnopqrstuvwxyz0123456789-")==std::string::npos,"prompt label");size_t count=bounded(in,1,479);size_t outputs=bounded(in,32,32);prompt.input=tokens(in,count);prompt.reference=tokens(in,outputs);p.prompts.push_back(std::move(prompt));}
 in>>std::ws;require(in.eof(),"trailing plan data");return p;
}
uint32_t winner(const std::vector<float>& logits){
 require(logits.size()==248320,"vocabulary");uint32_t best=0;
 for(size_t i=0;i<logits.size();++i){require(std::isfinite(logits[i]),"nonfinite logits");if(logits[i]>logits[best]||(logits[i]==logits[best]&&std::signbit(logits[i])<std::signbit(logits[best])))best=uint32_t(i);}
 return best;
}
float margin(const std::vector<float>& logits,uint32_t best){float second=-std::numeric_limits<float>::infinity();for(size_t i=0;i<logits.size();++i)if(i!=best)second=std::max(second,logits[i]);return logits[best]-second;}
struct State {
 void* session=nullptr;bool draft=false;std::optional<StateId> id;
 State(void* s,bool d):session(s),draft(d){}
 State(const State&)=delete;State& operator=(const State&)=delete;
 void reset(){if(id){auto old=*id;id.reset();check(draft?rs_qwen_device_mtp_release(session,&old,error,sizeof(error)):rs_qwen_device_release(session,&old,error,sizeof(error)));}}
 void replace(StateId next){auto old=id;id=next;if(old)check(draft?rs_qwen_device_mtp_release(session,&*old,error,sizeof(error)):rs_qwen_device_release(session,&*old,error,sizeof(error)));}
 ~State(){if(id){if(draft)rs_qwen_device_mtp_release(session,&*id,error,sizeof(error));else rs_qwen_device_release(session,&*id,error,sizeof(error));}}
 const StateId* ptr()const{return id?&*id:nullptr;}
 size_t position()const{return id?size_t(id->position):0;}
};
struct Row {bool mtp;size_t round,prompt,tokens,proposed=0,accepted=0,target_calls=0,draft_calls=0,target_forward_tokens=0;double prefill=0,decode=0,first_receipt_ms=0;std::vector<uint32_t> ids;};
Row run(void* session,const Plan& plan,size_t prompt_index,size_t round,bool mtp,size_t cap){
 const auto& prompt=plan.prompts[prompt_index];State target(session,false),draft(session,true);std::vector<float> target_logits(248320),draft_logits(248320);Row row{mtp,round,prompt_index,cap};
 auto start=Clock::now();
 for(size_t offset=0;offset<prompt.input.size();offset+=8){
  size_t count=std::min<size_t>(8,prompt.input.size()-offset);StateId next{};
  if(mtp)check(rs_qwen_device_hidden_step(session,target.ptr(),offset,prompt.input.data()+offset,count,&next,target_logits.data(),target_logits.size(),error,sizeof(error)));
  else check(rs_qwen_device_step(session,target.ptr(),offset,prompt.input.data()+offset,count,&next,target_logits.data(),target_logits.size(),nullptr,0,error,sizeof(error)));
  target.replace(next);
  if(mtp){std::vector<uint32_t> shifted(prompt.input.begin()+offset+1,prompt.input.begin()+offset+count);shifted.push_back(offset+count<prompt.input.size()?prompt.input[offset+count]:winner(target_logits));StateId dnext{};
   check(rs_qwen_device_mtp_step(session,draft.ptr(),target.ptr(),0,0,shifted.data(),count,&dnext,draft_logits.data(),draft_logits.size(),0,error,sizeof(error)));draft.replace(dnext);
  }
 }
 row.prefill=std::chrono::duration<double>(Clock::now()-start).count();start=Clock::now();std::vector<uint32_t> output;
 while(output.size()<cap){
  const bool first_receipt=output.empty();
  uint32_t seed=winner(target_logits);
  if(mtp){auto first=winner(draft_logits);rs_qwen_mtp_tx_result receipt{};
   check(rs_qwen_device_mtp_transaction(session,target.ptr(),draft.ptr(),seed,first,margin(draft_logits,first),4,cap-output.size(),plan.eos.data(),plan.eos.size(),1,&receipt,target_logits.data(),target_logits.size(),draft_logits.data(),draft_logits.size(),error,sizeof(error)));
   // Own every published handle before checking a receipt, including failures.
   target.replace(receipt.target_state);draft.replace(receipt.draft_state);
   require(receipt.emitted_count>=1&&receipt.emitted_count<=9&&receipt.emitted_count<=cap-output.size(),"receipt capacity");
   require(receipt.learned_accepted<=receipt.learned_proposed&&receipt.learned_proposed<=3&&receipt.target_calls>=1&&receipt.target_calls<=2,"receipt counters");
   output.insert(output.end(),receipt.emitted,receipt.emitted+receipt.emitted_count);row.proposed+=receipt.learned_proposed;row.accepted+=receipt.learned_accepted;row.target_calls+=receipt.target_calls;row.draft_calls+=receipt.draft_calls;row.target_forward_tokens+=receipt.target_forward_tokens;
   require(!receipt.terminal||output.size()==cap,"unexpected early terminal");
  }else{StateId next{};check(rs_qwen_device_step(session,target.ptr(),target.position(),&seed,1,&next,target_logits.data(),target_logits.size(),nullptr,0,error,sizeof(error)));target.replace(next);output.push_back(seed);++row.target_calls;++row.target_forward_tokens;}
  if(first_receipt)row.first_receipt_ms=1000*(row.prefill+std::chrono::duration<double>(Clock::now()-start).count());
 }
 row.decode=std::chrono::duration<double>(Clock::now()-start).count();
 require(std::equal(output.begin(),output.end(),prompt.reference.begin()),"frozen output mismatch: "+prompt.name);
 row.ids=output;target.reset();draft.reset();return row;
}
}
int main(int argc,char** argv){try{
 require(argc==3,"native_cpp_bench PLAN NEW_REPORT.json");{std::ifstream existing(argv[2]);require(!existing.good(),"report already exists");}
 auto plan=read_plan(argv[1]);
 Handle weights(rs_mlx_weights_create(error,sizeof(error)),rs_mlx_weights_destroy);require(bool(weights),error);
 for(const auto& path:plan.shards)check(rs_mlx_weights_load(weights.get(),path.c_str(),error,sizeof(error)));
 Handle mtp(rs_mlx_weights_create(error,sizeof(error)),rs_mlx_weights_destroy);require(bool(mtp),error);check(rs_mlx_weights_load(mtp.get(),plan.mtp.c_str(),error,sizeof(error)));
 for(const auto& q:plan.overrides){check(rs_qwen_quant(weights.get(),q.name.c_str(),q.group,q.bits,error,sizeof(error)));if(q.name.starts_with("language_model.mtp."))check(rs_qwen_quant(mtp.get(),q.name.c_str(),q.group,q.bits,error,sizeof(error)));}
 rs_qwen_config c={2048,40,248320,16,2,256,16,32,128,128,4,4,256,8,512,512,64,4,512,8,64,1,40,1,0,0,0,0,2,32,0,1e-6f,10000000.f};
 rs_qwen_layer layers[40];for(unsigned i=0;i<40;++i)layers[i]={i,uint32_t((i+1)%4==0)};
 Handle session(rs_qwen_device_create(weights.get(),1,&c,layers,40,16,1536ull*1024*1024,error,sizeof(error)),rs_qwen_device_destroy);require(bool(session),error);check(rs_qwen_device_mtp_attach(session.get(),mtp.get(),error,sizeof(error)));
 (void)run(session.get(),plan,0,0,false,8);(void)run(session.get(),plan,0,0,true,8);
 std::vector<Row> rows;for(size_t round=0;round<2;++round)for(size_t prompt=0;prompt<4;++prompt)for(bool mtp:round==0?std::vector<bool>{false,true}:std::vector<bool>{true,false})rows.push_back(run(session.get(),plan,prompt,round,mtp,32));
 uint64_t stats[74]{};check(rs_qwen_device_stats(session.get(),stats,74,error,sizeof(error)));require(stats[0]==0&&stats[1]==0,"nonzero final residency");uint64_t drained=99;check(rs_qwen_device_drain(session.get(),&drained,error,sizeof(error)));require(drained==0,"unexpected final drain");
 std::ostringstream out;out<<std::setprecision(17)<<"{\n\"pass\":true,\"identity\":\""<<plan.identity<<"\",\"execution\":\"standalone-cpp-existing-native-kernel\",\"http_streaming\":false,\"lanes\":1,\"continuous_batching\":false,\"rust_runtime\":false,\"turboquant_execution\":false,\"dspark_execution\":false,\"production_qualified\":false,\"output_congruence\":true,\"logical_bytes_at_shutdown\":0,\"mlx_peak_bytes\":"<<stats[6]<<",\"draft_width\":4,\"validation\":\"full\",\"layer_group\":40,\"context\":512,\"rows\":[\n";
 for(size_t i=0;i<rows.size();++i){const auto& r=rows[i];if(i)out<<",\n";out<<"{\"profile\":\""<<(r.mtp?"mtp-width4-reuse":"target-greedy")<<"\",\"round\":"<<r.round<<",\"prompt\":\""<<plan.prompts[r.prompt].name<<"\",\"tokens\":"<<r.tokens<<",\"decode_seconds\":"<<r.decode<<",\"decode_tokens_per_second\":"<<r.tokens/r.decode<<",\"first_receipt_ms\":"<<r.first_receipt_ms<<",\"prefill_seconds\":"<<r.prefill<<",\"learned_proposed\":"<<r.proposed<<",\"learned_accepted\":"<<r.accepted<<",\"target_calls\":"<<r.target_calls<<",\"draft_calls\":"<<r.draft_calls<<",\"target_forward_tokens\":"<<r.target_forward_tokens<<",\"ids\":[";for(size_t j=0;j<r.ids.size();++j){if(j)out<<",";out<<r.ids[j];}out<<"]}";}
 out<<"\n],\"timing_scope\":\"warmed standalone C++ native calls; model load/tokenization excluded; receipt groups are not streaming ITL\"}\n";require(bool(out),"report serialization failed");const auto contents=out.str();int fd=::open(argv[2],O_WRONLY|O_CREAT|O_EXCL,0644);require(fd>=0,"cannot create new report");size_t written=0;while(written<contents.size()){auto n=::write(fd,contents.data()+written,contents.size()-written);if(n<0&&errno==EINTR)continue;if(n<=0){::close(fd);throw std::runtime_error("report write failed");}written+=size_t(n);}require(::close(fd)==0,"report close failed");session.reset();mtp.reset();weights.reset();rs_mlx_cleanup();return 0;
 }catch(const std::exception& e){std::cerr<<"FAIL "<<e.what()<<'\n';rs_mlx_cleanup();return 1;}}
