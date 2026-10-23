// Diagnostic microprobe: plain vs bounded row-wise FP32 matmul, same operands.
#include <mlx/mlx.h>
#include <cmath>
#include <iostream>
#include <vector>
namespace mx=mlx::core;
int main(){
 constexpr int rows=4,width=4096,outputs=2048;
 std::vector<float> input(rows*width),weight(outputs*width);
 for(size_t i=0;i<input.size();++i)input[i]=std::sin(float(i)*0.013f)*0.3f;
 // Exactly representable BF16 values, as in the dense MTP weights.
 for(size_t i=0;i<weight.size();++i)weight[i]=float(int(i*17%257)-128)/512.f;
 auto s=mx::default_stream(mx::Device::gpu);auto x=mx::array(input.data(),{rows,width},mx::float32),w=mx::array(weight.data(),{outputs,width},mx::float32);
 auto matrix=mx::transpose(w,{1,0},s),plain=mx::matmul(x,matrix,s);
 std::vector<mx::array> pieces;for(int row=0;row<rows;++row)pieces.push_back(mx::matmul(mx::take(x,row,0,s),matrix,s));auto bounded=mx::stack(pieces,0,s);
 mx::eval({plain,bounded});mx::synchronize(s);
 auto error=[&](const mx::array& value){double d=0,n=0;auto p=value.data<float>();for(int row=0;row<rows;++row)for(int sample=0;sample<8;++sample){int out=sample*(outputs-1)/7;double sum=0;for(int k=0;k<width;++k)sum+=double(input[row*width+k])*weight[out*width+k];d+=std::pow(double(p[row*outputs+out])-sum,2);n+=sum*sum;}return std::sqrt(d/std::max(n,1e-30));};
 const auto plain_error=error(plain),bounded_error=error(bounded);
 std::cout<<"plain_relative_l2="<<plain_error<<" rowwise_relative_l2="<<bounded_error<<"\n";
 return (!std::isfinite(plain_error)||!std::isfinite(bounded_error)||bounded_error>1e-5)?1:0;
}
