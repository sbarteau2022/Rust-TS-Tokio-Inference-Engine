#pragma once
#include <algorithm>
#include <cmath>
#include <limits>
#include <stdexcept>
#include <vector>
// Diagnostic only: independent scalar FP64 GQA, bottom-right cached causal mask.
inline std::vector<double> attention_reference(const std::vector<float>& q,const std::vector<float>& k,const std::vector<float>& v,int heads,int kv_heads,int tokens,int offset,int dim){
 if(heads<=0||kv_heads<=0||heads%kv_heads||tokens<=0||offset<0||dim<=0)throw std::invalid_argument("oracle geometry");
 size_t n=size_t(offset)+tokens;
 if(q.size()!=size_t(heads)*tokens*dim||k.size()!=size_t(kv_heads)*n*dim||v.size()!=k.size())throw std::invalid_argument("oracle shape");
 for(const auto* values:{&q,&k,&v})for(float value:*values)if(!std::isfinite(value))throw std::invalid_argument("oracle nonfinite input");
 std::vector<double> result(q.size());
 for(int h=0;h<heads;++h)for(int row=0;row<tokens;++row){
  int kh=h/(heads/kv_heads);size_t visible=size_t(offset)+row+1;std::vector<double> scores(visible);double peak=-std::numeric_limits<double>::infinity();
  for(size_t j=0;j<visible;++j){double dot=0;for(int z=0;z<dim;++z)dot+=double(q[(size_t(h)*tokens+row)*dim+z])*k[(kh*n+j)*dim+z];scores[j]=dot/std::sqrt(double(dim));peak=std::max(peak,scores[j]);}
  double total=0;for(auto& score:scores){score=std::exp(score-peak);total+=score;}
  for(int z=0;z<dim;++z){double expected=0;for(size_t j=0;j<visible;++j)expected+=scores[j]/total*v[(kh*n+j)*dim+z];result[(size_t(h)*tokens+row)*dim+z]=expected;}
 }
 return result;
}
