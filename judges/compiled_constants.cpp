#include "compiled_constants.h"
#include <cassert>
#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <unordered_map>
#include <unordered_set>
namespace mx=mlx::core;
int main(int argc,char** argv){
 if(std::getenv("MLX_DISABLE_COMPILE"))throw std::invalid_argument("compiler disabled");
 mx::set_default_device(mx::Device(mx::Device::gpu));
 if(argc!=2)throw std::invalid_argument("unique fixture directory required");
 const std::filesystem::path directory(argv[1]);
 if(!std::filesystem::create_directory(directory))throw std::invalid_argument("fixture directory exists");
 const auto file=(directory/"weights.safetensors").string();
 auto s=mx::default_stream(mx::Device(mx::Device::gpu));
 {auto w=mx::ones({2048,2048},mx::float32,s);mx::eval(w);mx::save_safetensors(file,{{"layer.weight",w}});}
 mx::clear_cache();
 auto run=[&](bool prepare){
  auto arrays=mx::load_safetensors(file,mx::Device(mx::Device::cpu)).first;
  std::unordered_set<std::string> prefixes;
  if(prepare){
   for(const auto& prefix: {"", "layer.", "missing"}){
    bool refused=false;try{rs::prepare_compiled_parameters(arrays,prefixes,prefix,s);}catch(const std::invalid_argument&){refused=true;}
    assert(refused&&prefixes.empty());
   }
   assert(rs::prepare_compiled_parameters(arrays,prefixes,"layer",s));
   assert(!rs::prepare_compiled_parameters(arrays,prefixes,"layer",s));
   assert(prefixes.size()==1&&!arrays.at("layer.weight").has_primitive());
   arrays.emplace("broken.weight",mx::array({1},mx::float32,nullptr,{}));
   bool failed=false;try{rs::prepare_compiled_parameters(arrays,prefixes,"broken",s);}catch(const std::exception&){failed=true;}
   assert(failed&&!prefixes.contains("broken"));arrays.erase("broken.weight");
   for(int i=0;i<63;++i)prefixes.insert("reserved"+std::to_string(i));
   arrays.emplace("overflow.weight",arrays.at("layer.weight"));
   bool capped=false;try{rs::prepare_compiled_parameters(arrays,prefixes,"overflow",s);}catch(const std::runtime_error&){capped=true;}
   assert(capped&&!prefixes.contains("overflow"));
   assert(!rs::prepare_compiled_parameters(arrays,prefixes,"layer",s));
  }
  auto fn=mx::compile(std::function<std::vector<mx::array>(const std::vector<mx::array>&)>([&arrays,s](const std::vector<mx::array>& in){return std::vector<mx::array>{mx::matmul(in.at(0),arrays.at("layer.weight"),s)};}));
  for(int rows: {1,2,4,8,1,2}){
   auto out=fn({mx::ones({rows,2048},mx::float32,s)}).at(0);mx::eval(out);mx::synchronize(s);
   assert(out.size()==rows*2048);for(size_t i=0;i<out.size();++i)assert(out.data<float>()[i]==2048.f);
  }
  return mx::get_active_memory();
 };
 const auto lazy=run(false);mx::synchronize(s);mx::clear_cache();
 const auto shared=run(true);mx::synchronize(s);mx::clear_cache();
 // Four shape caches plus the live original must not multiply the payload.
 assert(shared<2*2048ULL*2048*4);
 assert(lazy>=3*shared);
 std::cout<<"PASS lazy_bytes="<<lazy<<" shared_bytes="<<shared<<" four_shapes repeated exact_outputs invalid_prefixes\n";
 std::filesystem::remove(file);std::filesystem::remove(directory);
}
