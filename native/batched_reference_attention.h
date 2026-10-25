#pragma once
#include <mlx/mlx.h>
#include <cmath>
#include <limits>
// Bounded FP32 causal reference reduction, with independent query rows batched.
inline mlx::core::array batched_reference_attention(const mlx::core::array& q,const mlx::core::array& k,const mlx::core::array& v,int offset,mlx::core::Stream s){
 namespace mx=mlx::core;
 if(q.ndim()!=4||k.ndim()!=4||v.shape()!=k.shape()||q.dtype()!=mx::float32||k.dtype()!=mx::float32||v.dtype()!=mx::float32)throw std::invalid_argument("batched attention rank/dtype");
 const int batch=q.shape(0),heads=q.shape(1),t=q.shape(2),dim=q.shape(3),kv=k.shape(1),length=k.shape(2);
 if(batch<1||batch>3||t<1||t>8||offset<0||length!=offset+t||length>512||kv<1||heads%kv||k.shape(0)!=batch||k.shape(3)!=dim||dim!=256)throw std::invalid_argument("batched attention geometry");
 const int groups=heads/kv;
 auto queries=mx::reshape(q,{batch,kv,groups,t,1,dim},s);
 auto keys=mx::reshape(k,{batch,kv,1,1,length,dim},s);
 auto values=mx::reshape(v,{batch,kv,1,1,length,dim},s);
 auto scores=mx::multiply(mx::sum(mx::multiply(queries,keys,s),-1,false,s),mx::array(1.f/std::sqrt(float(dim))),s);
 auto query_positions=mx::reshape(mx::arange(offset,offset+t,mx::int32,s),{t,1},s);
 auto key_positions=mx::arange(length,mx::int32,s);
 auto masked=mx::where(mx::less_equal(key_positions,query_positions,s),scores,mx::array(-std::numeric_limits<float>::infinity()),s);
 auto probabilities=mx::expand_dims(mx::softmax(masked,-1,true,s),-1,s);
 return mx::reshape(mx::sum(mx::multiply(probabilities,values,s),-2,false,s),{batch,heads,t,dim},s);
}
