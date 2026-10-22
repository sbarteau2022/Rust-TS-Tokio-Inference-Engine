#include "attention_reference.h"
#include <cassert>
int main(){
 // Zero queries: exact uniform weights, cached rows expose 3 then 4 keys.
 auto y=attention_reference({0,0,0,0},{9,8,7,6},{2,4,6,100},2,1,2,2,1);
 assert(y.size()==4);assert(y[0]==4 && y[1]==28 && y[2]==4 && y[3]==28);
 // Future value cannot influence first causal row, even with extreme logits.
 auto z=attention_reference({1000,1000},{1000,-1000},{3,99},1,1,2,0,1);
 assert(z[0]==3 && z[1]==3);
 auto g=attention_reference({0,0},{0,0},{5,11},2,2,1,0,1);
 assert(g[0]==5 && g[1]==11);
 auto soft=attention_reference({1,1},{0,1},{2,6},1,1,2,0,1);
 assert(soft[0]==2);assert(std::abs(soft[1]-(2+6*std::exp(1.))/(1+std::exp(1.)))<1e-12);
 bool finite=false;try{attention_reference({NAN},{0},{0},1,1,1,0,1);}catch(const std::invalid_argument&){finite=true;}assert(finite);
 for(int mode=0;mode<4;++mode){bool rejected=false;try{attention_reference({0},{0},{0},mode==0?0:2,mode==1?3:1,1,mode==2?-1:0,1);}catch(const std::invalid_argument&){rejected=true;}assert(rejected);}
}
