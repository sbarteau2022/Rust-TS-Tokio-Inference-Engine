// No model execution: finite checks, exact greedy ties and hot-loop allocations.
#include <cstdlib>
#include <new>
static bool count_allocations=false;
static size_t allocations=0;
void* operator new(std::size_t n){if(count_allocations)++allocations;if(auto p=std::malloc(n?n:1))return p;throw std::bad_alloc();}
void operator delete(void* p) noexcept {std::free(p);}
void operator delete(void* p,std::size_t) noexcept {std::free(p);}
#define main cpp_control_unused_main
#include "native_cpp_bench.cpp"
#undef main
int main(){try{
 std::vector<float> logits(248320,-1.f);logits[17]=2.f;logits[23]=2.f;
 count_allocations=true;auto selected=winner(logits);count_allocations=false;
 require(selected==17&&allocations==0,"finite winner selection/allocations");
 std::fill(logits.begin(),logits.end(),-1.f);logits[0]=-0.f;logits[1]=0.f;
 require(winner(logits)==1,"signed zero tie");logits[0]=0.f;require(winner(logits)==0,"first index tie");
 size_t rejected=0;
 for(size_t index:{size_t(0),logits.size()-1})for(float value:{std::numeric_limits<float>::quiet_NaN(),std::numeric_limits<float>::infinity(),-std::numeric_limits<float>::infinity()}){
  float old=logits[index];logits[index]=value;bool threw=false;try{(void)winner(logits);}catch(const std::runtime_error&){threw=true;}logits[index]=old;require(threw,"nonfinite escaped finite check");++rejected;
 }
 for(size_t size:{size_t(0),size_t(248319),size_t(248321)}){bool threw=false;try{(void)winner(std::vector<float>(size));}catch(const std::runtime_error&){threw=true;}require(threw,"wrong vocabulary accepted");++rejected;}
 // Exercise publication itself, bypassing main's early existence check.
 char path[]="/tmp/cpp-publication-XXXXXX";int fd=::mkstemp(path);require(fd>=0,"temporary report");
 require(::write(fd,"keep",4)==4&&::close(fd)==0,"temporary report write");
 bool refused=false;try{publish_report(path,"overwrite");}catch(const std::runtime_error&){refused=true;}
 std::ifstream retained(path);std::string content;retained>>content;retained.close();::unlink(path);
 require(refused&&content=="keep","exclusive report publication");
 publish_report(path,"new");std::ifstream created(path);created>>content;created.close();::unlink(path);require(content=="new","new report publication");
 std::cout<<"{\"pass\":true,\"finite_scan_allocations\":"<<allocations<<",\"rejected\":"<<rejected<<",\"signed_zero_and_first_ties\":true,\"exclusive_publication\":true}\n";return 0;
 }catch(const std::exception& e){count_allocations=false;std::cerr<<"FAIL "<<e.what()<<'\n';return 1;}}
