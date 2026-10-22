#include "projection_reference.h"
#include <cassert>
#include <iostream>
#include <limits>
#include <functional>
// Packing uses one bit at a time, independently of the oracle's word extraction.
static void set_value(std::vector<uint32_t>& packed,size_t row,size_t width,size_t col,int bits,unsigned value){
 size_t row_words=width*bits/32;
 for(int bit=0;bit<bits;++bit)if(value&(1u<<bit)){
  size_t position=col*bits+bit;
  packed[row*row_words+position/32]|=1u<<(position%32);
 }
}
int main(){
 int checks=0;
 for(int bits:{2,3,4,5,6,8})for(int group:{32,64,128})for(size_t rows:{size_t(1),size_t(8)}){
  size_t width=group*2,outputs=8;std::vector<float> x(rows*width),scale(outputs*2),bias(outputs*2);std::vector<uint32_t> packed(outputs*width*bits/32,0);std::vector<double> expected(rows*outputs,0.);
  for(size_t i=0;i<x.size();++i)x[i]=(int(i%19)-9)*.125f;
  for(size_t i=0;i<scale.size();++i){scale[i]=(i%2?-.25f:.125f);bias[i]=float(int(i%7)-3)*.5f;}
  for(size_t out=0;out<outputs;++out)for(size_t col=0;col<width;++col){
   unsigned q=(col*17+out*3)&((1u<<bits)-1);set_value(packed,out,width,col,bits,q);
   double weight=double(scale[out*2+col/group])*q+bias[out*2+col/group];
   for(size_t row=0;row<rows;++row)expected[row*outputs+out]+=double(x[row*width+col])*weight;
  }
  auto actual=projection_reference(x,packed,scale,bias,rows,outputs,width,group,bits);
  assert(actual==expected);++checks;
 }
 // Literal 3-bit byte stream: values 0,1,2,3,4,5,6,7 -> 88 c6 fa.
 std::vector<uint32_t> literal={0x00fac688,0,0};
 for(size_t col=0;col<8;++col){std::vector<float> x(32,0);x[col]=1;assert(projection_reference(x,literal,{1},{0},1,1,32,32,3)[0]==double(col));++checks;}
 auto reject=[&](const std::function<void()>& f){bool failed=false;try{f();}catch(const std::invalid_argument&){failed=true;}assert(failed);++checks;};
 const std::vector<float> x(32,1);const std::vector<uint32_t> packed(4,0);
 reject([&]{projection_reference(x,packed,{1},{0},0,1,32,32,4);});
 reject([&]{projection_reference(x,packed,{1},{0},1,9,32,32,4);});
 reject([&]{projection_reference(x,packed,{1},{0},1,1,32769,32,4);});
 reject([&]{projection_reference(x,packed,{1},{0},1,1,32,0,4);});
 reject([&]{projection_reference(x,packed,{1},{0},1,1,32,32,7);});
 reject([&]{projection_reference(x,packed,{1},{0},1,1,31,32,4);});
 reject([&]{projection_reference({},packed,{1},{0},1,1,32,32,4);});
 reject([&]{projection_reference(x,{}, {1},{0},1,1,32,32,4);});
 reject([&]{projection_reference(x,packed,{NAN},{0},1,1,32,32,4);});
 reject([&]{auto invalid=x;invalid[0]=INFINITY;projection_reference(invalid,packed,{1},{0},1,1,32,32,4);});
 reject([&]{projection_reference(x,packed,{1},{INFINITY},1,1,32,32,4);});
 std::cout<<"PASS scalar projection oracle checks="<<checks<<" widths=2,3,4,5,6,8 groups=32,64,128 sanitizer=address,undefined\n";
}
