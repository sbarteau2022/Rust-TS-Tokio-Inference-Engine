#pragma once
#include <cmath>
#include <cstdint>
#include <stdexcept>
#include <vector>
// Diagnostic only. Independent scalar affine dequantization and FP64 dot product.
inline std::vector<double> projection_reference(const std::vector<float>& x,const std::vector<uint32_t>& packed,const std::vector<float>& scales,const std::vector<float>& biases,size_t rows,size_t outputs,size_t width,int group,int bits){
 if(rows==0||rows>8||outputs==0||outputs>8||width==0||width>32768||(group!=32&&group!=64&&group!=128)||(bits!=2&&bits!=3&&bits!=4&&bits!=5&&bits!=6&&bits!=8)||width%size_t(group))throw std::invalid_argument("projection oracle geometry");
 const size_t words=width*bits/32,groups=width/group;
 if(x.size()!=rows*width||packed.size()!=outputs*words||scales.size()!=outputs*groups||biases.size()!=scales.size())throw std::invalid_argument("projection oracle shape");
 for(const auto* a:{&x,&scales,&biases})for(float value:*a)if(!std::isfinite(value))throw std::invalid_argument("projection oracle nonfinite input");
 std::vector<double> result(rows*outputs,0.);
 for(size_t row=0;row<rows;++row)for(size_t out=0;out<outputs;++out){
  double sum=0;
  for(size_t col=0;col<width;++col){
   size_t bit=col*bits,word=bit/32;unsigned shift=bit%32;
   uint64_t value=packed[out*words+word];
   if(shift+bits>32)value|=uint64_t(packed[out*words+word+1])<<32;
   unsigned q=(value>>shift)&((1u<<bits)-1);
   size_t g=out*groups+col/group;
   double weight=double(scales[g])*q+double(biases[g]);
   sum+=double(x[row*width+col])*weight;
  }
  result[row*outputs+out]=sum;
 }
 return result;
}
