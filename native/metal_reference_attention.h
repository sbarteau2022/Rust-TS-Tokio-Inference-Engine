#pragma once
#include <mlx/mlx.h>
#include <stdexcept>
// Safe FP32 arithmetic; independent scalar/FP64 snapshot controls remain intact.
inline mlx::core::array metal_reference_attention(const mlx::core::array& q,const mlx::core::array& k,const mlx::core::array& v,int offset,mlx::core::Stream stream){
 namespace mx=mlx::core;
 if(q.ndim()!=4||k.ndim()!=4||v.shape()!=k.shape()||q.dtype()!=mx::float32||k.dtype()!=mx::float32||v.dtype()!=mx::float32)throw std::invalid_argument("Metal reference attention rank/dtype");
 int batch=q.shape(0),heads=q.shape(1),rows=q.shape(2),dim=q.shape(3),kv=k.shape(1),length=k.shape(2);
 if(batch<1||batch>3||heads<1||heads>32||kv<1||heads%kv||rows<1||rows>8||dim!=256||offset<0||offset>512||length!=offset+rows||length>512||k.shape(0)!=batch||k.shape(3)!=dim)throw std::invalid_argument("Metal reference attention geometry");
 static const auto dot=mx::fast::metal_kernel("fp32_reference_attention_dot",{"q","k","position"},{"scores"},R"METAL(
  uint Batch=q_shape[0],Heads=q_shape[1],Rows=q_shape[2],KvHeads=k_shape[1],Length=k_shape[2],Offset=position[0];
  uint lane=thread_position_in_grid.x;
  uint key=thread_position_in_grid.y;
  uint row=thread_position_in_grid.z;
  uint query=row%Rows;
  uint head=(row/Rows)%Heads;
  uint batch=row/(Rows*Heads);
  if(key>=Length||row>=Batch*Heads*Rows)return;
  if(key>Offset+query){if(lane==0)scores[row*Length+key]=-INFINITY;return;}
  uint qbase=row*256;
  uint kbase=((batch*KvHeads+head/(Heads/KvHeads))*Length+key)*256;
  float value=0.0f;
  for(uint d=lane;d<256;d+=32){float product=float(q[qbase+d])*float(k[kbase+d]);value=value+product;}
  value=simd_sum(value);
  if(lane==0)scores[row*Length+key]=value*0.0625f;
 )METAL");
 static const auto weighted=mx::fast::metal_kernel("fp32_reference_attention_value",{"probabilities","v","position"},{"output"},R"METAL(
  uint Batch=probabilities_shape[0],Heads=probabilities_shape[1],Rows=probabilities_shape[2],KvHeads=v_shape[1],Length=v_shape[2],Offset=position[0];
  uint d=thread_position_in_grid.x;
  uint row=thread_position_in_grid.y;
  if(d>=256||row>=Batch*Heads*Rows)return;
  uint query=row%Rows;
  uint head=(row/Rows)%Heads;
  uint batch=row/(Rows*Heads);
  uint vbase=(batch*KvHeads+head/(Heads/KvHeads))*Length*256;
  float value=0.0f;
  for(uint key=0;key<=Offset+query;++key){float product=float(probabilities[row*Length+key])*float(v[vbase+key*256+d]);value=value+product;}
  output[row*256+d]=value;
 )METAL");
 int32_t value=offset;auto position=mx::array(&value,{1},mx::int32);
 auto scores=dot({q,k,position},{{batch,heads,rows,length}},{mx::float32},{32,length,batch*heads*rows},{32,1,1},{},{},false,stream)[0];
 auto probabilities=mx::softmax(scores,-1,true,stream);
 return weighted({probabilities,v,position},{{batch,heads,rows,dim}},{mx::float32},{256,batch*heads*rows,1},{256,1,1},{},{},false,stream)[0];
}
