#pragma once
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <stdexcept>
#include <vector>
namespace scalar_reference {
inline void finite(const std::vector<float>& x){for(float v:x)if(!std::isfinite(v))throw std::invalid_argument("operator oracle nonfinite input");}
inline double sigmoid(double x){if(x>=0)return 1./(1.+std::exp(-x));double e=std::exp(x);return e/(1.+e);}
inline std::vector<double> silu(const std::vector<float>& x){finite(x);std::vector<double> y;for(float v:x)y.push_back(double(v)*sigmoid(v));return y;}
inline std::vector<double> rms(const std::vector<float>& x,const std::vector<float>& weight,size_t width,double epsilon){
 if(width==0||width>8192||x.empty()||x.size()>1024*1024||x.size()%width||(!weight.empty()&&weight.size()!=width)||!std::isfinite(epsilon)||epsilon<=0)throw std::invalid_argument("RMS oracle geometry");finite(x);finite(weight);
 std::vector<double> y(x.size());for(size_t row=0;row<x.size()/width;++row){double square=0;for(size_t col=0;col<width;++col){double v=x[row*width+col];square+=v*v;}double inverse=1./std::sqrt(square/width+epsilon);for(size_t col=0;col<width;++col)y[row*width+col]=double(x[row*width+col])*inverse*(weight.empty()?1.:weight[col]);}return y;
}
inline std::vector<double> rope(const std::vector<float>& x,int heads,int tokens,int dim,int rotary,int offset,double theta){
 if(heads<=0||heads>64||tokens<=0||tokens>8||dim<=0||dim>256||rotary<=0||rotary>dim||rotary%2||offset<0||offset>512-tokens||!std::isfinite(theta)||theta<=1||x.size()!=size_t(heads)*tokens*dim)throw std::invalid_argument("RoPE oracle geometry");finite(x);
 std::vector<double> y(x.begin(),x.end());for(int h=0;h<heads;++h)for(int t=0;t<tokens;++t)for(int j=0;j<rotary/2;++j){size_t first=(size_t(h)*tokens+t)*dim+j,second=first+rotary/2;double angle=(offset+t)*std::pow(theta,-2.*j/rotary);double c=std::cos(angle),s=std::sin(angle);y[first]=double(x[first])*c-double(x[second])*s;y[second]=double(x[first])*s+double(x[second])*c;}return y;
}
struct Delta {std::vector<double> activated,history,output,state;};
inline Delta delta(const std::vector<float>& qkv,const std::vector<float>& history,const std::vector<float>& convolution,const std::vector<float>& a,const std::vector<float>& b,const std::vector<float>& alog,const std::vector<float>& dt,const std::vector<float>& initial,int tokens,int hk,int hv,int dk,int dv,int kernel){
 if(tokens<=0||tokens>8||hk<=0||hk>64||hv<=0||hv>64||hv%hk||dk<=0||dk>256||dv<=0||dv>256||kernel<2||kernel>16)throw std::invalid_argument("delta oracle geometry");
 size_t kd=size_t(hk)*dk,vd=size_t(hv)*dv,channels=2*kd+vd;
 if(qkv.size()!=size_t(tokens)*channels||history.size()!=size_t(kernel-1)*channels||convolution.size()!=channels*kernel||a.size()!=size_t(tokens)*hv||b.size()!=a.size()||alog.size()!=size_t(hv)||dt.size()!=alog.size()||initial.size()!=size_t(hv)*dv*dk)throw std::invalid_argument("delta oracle shape");
 for(const auto* v:{&qkv,&history,&convolution,&a,&b,&alog,&dt,&initial})finite(*v);
 std::vector<double> stream(history.begin(),history.end());stream.insert(stream.end(),qkv.begin(),qkv.end());
 Delta result;result.state.assign(initial.begin(),initial.end());result.activated.resize(qkv.size());result.output.resize(size_t(tokens)*vd);
 result.history.assign(stream.begin()+size_t(tokens)*channels,stream.end());
 for(int t=0;t<tokens;++t){
  for(size_t ch=0;ch<channels;++ch){double value=0;for(int tap=0;tap<kernel;++tap)value+=stream[(size_t(t)+tap)*channels+ch]*convolution[ch*kernel+tap];result.activated[size_t(t)*channels+ch]=value*sigmoid(value);}
  for(int head=0;head<hv;++head){int keyhead=head/(hv/hk);std::vector<double> q(dk),k(dk);double qs=0,ks=0;
   for(int j=0;j<dk;++j){q[j]=result.activated[size_t(t)*channels+size_t(keyhead)*dk+j];k[j]=result.activated[size_t(t)*channels+kd+size_t(keyhead)*dk+j];qs+=q[j]*q[j];ks+=k[j]*k[j];}
   double qi=1./(std::sqrt(qs/dk+1e-6)*dk),ki=1./(std::sqrt(ks/dk+1e-6)*std::sqrt(double(dk)));
   for(int j=0;j<dk;++j){q[j]*=qi;k[j]*=ki;}
   double logit=double(a[size_t(t)*hv+head])+dt[head];double softplus=std::max(logit,0.)+std::log1p(std::exp(-std::abs(logit)));double decay=std::exp(-std::exp(double(alog[head]))*softplus),beta=sigmoid(b[size_t(t)*hv+head]);
   for(int value=0;value<dv;++value){size_t start=(size_t(head)*dv+value)*dk;double memory=0;
    for(int j=0;j<dk;++j){result.state[start+j]*=decay;memory+=result.state[start+j]*k[j];}
    double change=(result.activated[size_t(t)*channels+2*kd+size_t(head)*dv+value]-memory)*beta,read=0;
    for(int j=0;j<dk;++j){result.state[start+j]+=change*k[j];read+=result.state[start+j]*q[j];}
    result.output[size_t(t)*vd+size_t(head)*dv+value]=read;
   }
  }
 }
 return result;
}
}
namespace scalar_reference {
struct Routing {std::vector<double> probabilities,weights;};
inline Routing routing(const std::vector<float>& logits,const std::vector<int32_t>& selected,int rows,int experts,int top){
 if(rows<=0||rows>8||experts<=0||experts>256||top<=0||top>experts||top>16||logits.size()!=size_t(rows)*experts||selected.size()!=size_t(rows)*top)throw std::invalid_argument("routing oracle shape");finite(logits);Routing result;
 for(int row=0;row<rows;++row){const float* values=logits.data()+row*experts;double peak=*std::max_element(values,values+experts),total=0;for(int e=0;e<experts;++e)total+=std::exp(double(values[e])-peak);for(int e=0;e<experts;++e)result.probabilities.push_back(std::exp(double(values[e])-peak)/total);
  std::vector<bool> seen(experts,false);double minimum=INFINITY,weight=0;
  for(int j=0;j<top;++j){int id=selected[row*top+j];if(id<0||id>=experts||seen[id])throw std::invalid_argument("invalid scalar routing membership");seen[id]=true;minimum=std::min(minimum,double(values[id]));weight+=std::exp(double(values[id])-peak);}
  for(int e=0;e<experts;++e)if(!seen[e]&&values[e]>minimum)throw std::invalid_argument("non-top-k scalar routing membership");
  for(int j=0;j<top;++j)result.weights.push_back(std::exp(double(values[selected[row*top+j]])-peak)/weight);
 }
 return result;
}
}
