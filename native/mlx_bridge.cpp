#include "mlx_bridge.h"
#include <mlx/mlx.h>
#include <algorithm>
#include <cstring>
#include <cstdlib>
#include "operator_timing.h"
#include <stdexcept>
#include <vector>
namespace mx = mlx::core;
namespace {
struct Graph {
  mx::Stream stream;
  std::vector<mx::array> nodes;
  explicit Graph(bool gpu) : stream(mx::default_stream(mx::Device(gpu ? mx::Device::gpu : mx::Device::cpu))) {}
  ~Graph() { try { mx::synchronize(stream); } catch (...) {} }
};
void report(char* error, size_t capacity, const char* message) noexcept {
  if (error && capacity) {
    const size_t length = std::min(std::strlen(message), capacity - 1);
    std::memcpy(error, message, length);
    error[length] = 0;
  }
}
template <class F> int checked(char* error, size_t capacity, F f) noexcept {
  try { f(); return 0; }
  catch (const std::exception& e) { report(error, capacity, e.what()); }
  catch (...) { report(error, capacity, "unknown native MLX exception"); }
  return -1;
}
}
extern "C" void* rs_mlx_create(int gpu, char* error, size_t capacity) {
  try { return new Graph(gpu != 0); }
  catch (const std::exception& e) { report(error, capacity, e.what()); }
  catch (...) { report(error, capacity, "MLX context creation failed"); }
  return nullptr;
}
extern "C" void rs_mlx_destroy(void* graph) { delete static_cast<Graph*>(graph); }
extern "C" const char* rs_mlx_version() { return mx::version(); }
extern "C" int rs_mlx_input(void* graph, int dtype, const void* data,
                             const int32_t* shape, size_t rank, char* error, size_t capacity) {
  return checked(error, capacity, [&] {
    if (!graph || !data || !shape || rank == 0 || rank > 4) throw std::invalid_argument("invalid input descriptor");
    auto& g = *static_cast<Graph*>(graph);
    if (g.nodes.size() >= 256) throw std::invalid_argument("node cap");
    mx::Shape dims(shape, shape + rank);
    if (std::any_of(dims.begin(), dims.end(), [](auto d) { return d <= 0 || d > 8192; })) throw std::invalid_argument("invalid dimension");
    // Iterator constructors copy caller data; no Rust allocation is borrowed by a lazy graph.
    if (dtype == 0) g.nodes.emplace_back(static_cast<const float*>(data), dims, mx::float32);
    else if (dtype == 1) g.nodes.emplace_back(static_cast<const int32_t*>(data), dims, mx::int32);
    else if (dtype == 2) g.nodes.emplace_back(static_cast<const uint32_t*>(data), dims, mx::uint32);
    else throw std::invalid_argument("invalid input dtype");
  });
}
extern "C" int rs_mlx_op(void* graph, int op, const uint32_t* inputs, size_t count,
                          int group, float epsilon, char* error, size_t capacity) {
  return checked(error, capacity, [&] {
    if (!graph || !inputs) throw std::invalid_argument("invalid operation descriptor");
    auto& g = *static_cast<Graph*>(graph);
    if (g.nodes.size() >= 256) throw std::invalid_argument("node cap");
    const bool quantized = op==3 || op==102 || op==103 || op==104 || op==105 || op==106 || op==108;
    const size_t expected = quantized ? 4 : 2;
    if (count != expected) throw std::invalid_argument("invalid operand count");
    for (size_t i = 0; i < count; ++i) if (inputs[i] >= g.nodes.size()) throw std::invalid_argument("invalid node reference");
    auto& a = g.nodes.at(inputs[0]);
    auto& b = g.nodes.at(inputs[1]);
    switch (op) {
      case 0: g.nodes.push_back(mx::add(a, b, g.stream)); break;
      case 1: g.nodes.push_back(mx::multiply(a, b, g.stream)); break;
      case 2: g.nodes.push_back(mx::matmul(a, b, g.stream)); break;
      case 3: g.nodes.push_back(mx::quantized_matmul(a, b, g.nodes.at(inputs[2]), g.nodes.at(inputs[3]), true, group, 4, "affine", g.stream)); break;
      case 4: g.nodes.push_back(mx::fast::rms_norm(a, b, epsilon, g.stream)); break;
      default:
        if (quantized) g.nodes.push_back(mx::quantized_matmul(a,b,g.nodes.at(inputs[2]),g.nodes.at(inputs[3]),true,group,op-100,"affine",g.stream));
        else throw std::invalid_argument("unknown operation");
    }
  });
}
extern "C" int rs_mlx_finish(void* graph, uint32_t output, void* data, size_t elements,
                              char* error, size_t capacity) {
  return checked(error, capacity, [&] {
    if (!graph || !data) throw std::invalid_argument("invalid output descriptor");
    auto& g = *static_cast<Graph*>(graph);
    const auto& node = g.nodes.at(output);
    if (node.size() != elements) throw std::invalid_argument("output size mismatch");
    auto value = mx::contiguous(node.itemsize()==2?mx::astype(node,mx::float32,g.stream):node, false, g.stream);
    mx::eval(value);
    mx::synchronize(g.stream);
    if (value.dtype() == mx::float32) std::memcpy(data, value.data<float>(), elements * 4);
    else if (value.dtype() == mx::int32) std::memcpy(data, value.data<int32_t>(), elements * 4);
    else if (value.dtype() == mx::uint32) std::memcpy(data, value.data<uint32_t>(), elements * 4);
    else throw std::invalid_argument("unsupported output dtype");
  });
}
extern "C" void rs_mlx_cleanup() { try { mx::clear_streams(); } catch (...) {} }
namespace {
struct Weights { ExecutionMetrics timing; OperatorCounters operators; std::unordered_map<std::string,mx::array> arrays; std::unordered_map<std::string,std::pair<int,int>> quant; uint64_t compiled_calls=0,compiled_misses=0,fused_expert_bytes=0; bool has_shift=false;
 std::unordered_map<std::string,std::function<std::vector<mx::array>(const std::vector<mx::array>&)>> compiled_blocks;
 std::unordered_map<std::string,std::function<std::vector<mx::array>(const std::vector<mx::array>&)>> fast_compiled_blocks;
};
}
extern "C" void* rs_mlx_weights_create(char* error, size_t capacity) {
  try { return new Weights; } catch (const std::exception& e) {report(error,capacity,e.what());} catch (...) {report(error,capacity,"weight store creation failed");} return nullptr;
}
extern "C" void rs_mlx_weights_destroy(void* store) { delete static_cast<Weights*>(store); }
extern "C" int rs_mlx_weights_load(void* store, const char* file, char* error, size_t capacity) {
  return checked(error,capacity,[&] {
    if (!store || !file) throw std::invalid_argument("invalid weight load descriptor");
    auto& weights=*static_cast<Weights*>(store);
    auto loaded=mx::load_safetensors(std::string(file));
    for (auto& [name,value]:loaded.first) {
      if (weights.arrays.contains(name)) throw std::invalid_argument("duplicate native parameter");
      if (name.find("mtp.")!=std::string::npos || (name.ends_with("conv1d.weight")&&value.shape(-1)!=1)) weights.has_shift=true;
      weights.arrays.emplace(name,std::move(value));
    }
  });
}
extern "C" int rs_mlx_parameter(void* graph, void* store, const char* name,
                                 const int32_t* shape,size_t rank,int dtype,char* error,size_t capacity) {
  return checked(error,capacity,[&] {
    if (!graph || !store || !name || !shape || rank==0 || rank>4) throw std::invalid_argument("invalid parameter descriptor");
    auto& g=*static_cast<Graph*>(graph);auto& weights=*static_cast<Weights*>(store);
    const auto& value=weights.arrays.at(name);
    if (g.nodes.size()>=256 || value.ndim()!=rank || !std::equal(shape,shape+rank,value.shape().begin())) throw std::invalid_argument("parameter shape mismatch");
    const auto expected=dtype==0?mx::float32:dtype==1?mx::int32:dtype==2?mx::uint32:dtype==3?mx::float16:mx::bfloat16;
    if (dtype<0 || dtype>4 || value.dtype()!=expected) throw std::invalid_argument("parameter dtype mismatch");
    g.nodes.push_back(value);
  });
}
extern "C" int rs_mlx_kernel(void* graph,int op,const uint32_t* inputs,size_t count,
                              const int32_t* ints,size_t ni,const float* floats,size_t nf,char* error,size_t capacity) {
 return checked(error,capacity,[&] {
  if (!graph || !inputs) throw std::invalid_argument("invalid kernel descriptor");
  auto& g=*static_cast<Graph*>(graph);auto stream=g.stream;
  auto at=[&](size_t n)->const mx::array& {if(n>=count)throw std::invalid_argument("kernel operand missing");return g.nodes.at(inputs[n]);};
  auto integer=[&](size_t n){if(!ints||n>=ni)throw std::invalid_argument("kernel integer missing");return ints[n];};
  auto scalar=[&](size_t n){if(!floats||n>=nf)throw std::invalid_argument("kernel scalar missing");return floats[n];};
  std::vector<mx::array> outputs;
  if(op==1) outputs.push_back(mx::fast::rope(at(0),integer(0),false,scalar(0),1.0f,integer(1),{},stream));
  else if(op==2) outputs.push_back(mx::fast::scaled_dot_product_attention(at(0),at(1),at(2),scalar(0),integer(0)?"causal":"",{},{},false,stream));
  else if(op==3) {
    auto state=at(5);std::vector<mx::array> values;
    const auto q=at(0),k=at(1),v=at(2),decay=at(3),beta=at(4);
    const int repeats=v.shape(2)/q.shape(2);
    for(int t=0;t<q.shape(1);++t) {
      auto qt=mx::repeat(mx::take(q,t,1,stream),repeats,1,stream);
      auto kt=mx::repeat(mx::take(k,t,1,stream),repeats,1,stream);
      state=mx::multiply(state,mx::expand_dims(mx::take(decay,t,1,stream),{2,3},stream),stream);
      auto memory=mx::sum(mx::multiply(state,mx::expand_dims(kt,-2,stream),stream),-1,false,stream);
      auto delta=mx::multiply(mx::subtract(mx::take(v,t,1,stream),memory,stream),mx::expand_dims(mx::take(beta,t,1,stream),-1,stream),stream);
      state=mx::add(state,mx::multiply(mx::expand_dims(kt,-2,stream),mx::expand_dims(delta,-1,stream),stream),stream);
      values.push_back(mx::sum(mx::multiply(state,mx::expand_dims(qt,-2,stream),stream),-1,false,stream));
    }
    outputs={mx::stack(values,1,stream),state};
  } else if(op==4) {
    auto probabilities=mx::softmax(at(0),-1,true,stream);const int top=integer(0);
    auto indices=mx::argpartition(probabilities,-top,-1,stream);
    indices=mx::slice(indices,{0,indices.shape(1)-top},{indices.shape(0),indices.shape(1)},stream);
    auto scores=mx::take_along_axis(probabilities,indices,-1,stream);
    scores=mx::divide(scores,mx::sum(scores,-1,true,stream),stream);
    outputs={indices,scores};
  } else if(op==5) {
    auto value=mx::gather_qmm(mx::expand_dims(at(0),{-2,-3},stream),at(1),at(2),at(3),{},at(4),true,integer(0),integer(1),"affine",{},false,stream);
    outputs.push_back(mx::squeeze(value,-2,stream));
  } else if(op==6) outputs.push_back(mx::sum(mx::multiply(at(0),mx::expand_dims(at(1),-1,stream),stream),1,false,stream));
  else if(op==7) outputs.push_back(mx::multiply(at(0),mx::sigmoid(at(0),stream),stream));
  else if(op==8) outputs.push_back(mx::concatenate({mx::reshape(at(0),{-1},stream),mx::reshape(at(1),{-1},stream)},0,stream));
  else throw std::invalid_argument("unsupported native kernel");
  if(g.nodes.size()+outputs.size()>256)throw std::invalid_argument("kernel node cap");
  g.nodes.insert(g.nodes.end(),outputs.begin(),outputs.end());
 });
}

#include "qwen_model.inc"
extern "C" int rs_mlx_weight_bytes(void* store,const char* name,const int32_t* shape,size_t rank,int dtype,size_t offset,uint8_t* output,size_t count,char* error,size_t capacity){
 return checked(error,capacity,[&]{
  if(!store||!name||!shape||rank==0||rank>4||dtype<0||dtype>4||!output||count==0||count>4*1024*1024)throw std::invalid_argument("weight byte export descriptor");
  auto& value=static_cast<Weights*>(store)->arrays.at(name);
  const mx::Dtype types[]={mx::float32,mx::int32,mx::uint32,mx::float16,mx::bfloat16};
  if(value.ndim()!=rank||value.dtype()!=types[dtype]||offset>value.nbytes()||count>value.nbytes()-offset)throw std::invalid_argument("weight byte export bounds/type");
  for(size_t i=0;i<rank;++i)if(shape[i]!=value.shape(i))throw std::invalid_argument("weight byte export shape");
  auto cpu=mx::contiguous(value,false,mx::Device::cpu);mx::eval(cpu);mx::synchronize(mx::default_stream(mx::Device::cpu));
  std::memcpy(output,cpu.data<uint8_t>()+offset,count);
 });
}
