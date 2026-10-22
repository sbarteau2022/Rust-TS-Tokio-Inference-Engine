 std::vector<float> cpu_values(const mx::array& x){auto a=mx::contiguous(mx::astype(x,mx::float32,s),false,s);mx::eval(a);mx::synchronize(s);const float* p=a.data<float>();return {p,p+a.size()};}
 void check_operator(const std::string& label,const std::vector<double>& expected,const mx::array& actual){
  auto observed=cpu_values(actual);if(expected.size()!=observed.size()||expected.empty())throw std::invalid_argument("operator probe shape");
  double error=0,norm=0,maximum=0;for(size_t i=0;i<expected.size();++i){double delta=observed[i]-expected[i];error+=delta*delta;norm+=expected[i]*expected[i];maximum=std::max(maximum,std::abs(delta));}
  double nrmse=std::sqrt(error/std::max(norm,1e-30));std::fprintf(stderr,"OPERATOR-ORACLE operation=%s elements=%zu nrmse=%.9g max_abs=%.9g\n",label.c_str(),expected.size(),nrmse,maximum);
  if(!std::isfinite(nrmse)||nrmse>1e-5)throw std::runtime_error("operator differs from scalar FP64 oracle: "+label);
 }
 mx::array rope_checked(const mx::array& x,int offset,const std::string& label){auto y=mx::fast::rope(x,c.rope_dims,false,c.theta,1.f,offset,{},s);if(operators)check_operator(label,scalar_reference::rope(cpu_values(x),x.shape(1),x.shape(2),c.head_dim,c.rope_dims,offset,c.theta),y);return y;}
 void check_delta(const mx::array& qkv,const mx::array& conv,const mx::array& cw,const mx::array& a,const mx::array& b,const mx::array& old,const mx::array& activated,const mx::array& output,const mx::array& state,const mx::array& history,const std::string& label){
  auto expected=scalar_reference::delta(cpu_values(qkv),cpu_values(conv),cpu_values(cw),cpu_values(a),cpu_values(b),cpu_values(w(label+".A_log")),cpu_values(w(label+".dt_bias")),cpu_values(old),qkv.shape(1),c.key_heads,c.value_heads,c.key_dim,c.value_dim,c.conv);
  check_operator(label+"/convolution-silu",expected.activated,activated);check_operator(label+"/history",expected.history,history);check_operator(label+"/readout",expected.output,output);check_operator(label+"/state",expected.state,state);
 }
 void check_embedding(const mx::array& ids,const mx::array& value,const std::string& label,int group,int bits){
  auto packed=mx::contiguous(mx::take(w(label+".weight"),ids,0,s),false,s);mx::eval(packed);mx::synchronize(s);const uint32_t* raw=packed.data<uint32_t>();
  auto expected=embedding_reference({raw,raw+packed.size()},cpu_values(mx::take(w(label+".scales"),ids,0,s)),cpu_values(mx::take(w(label+".biases"),ids,0,s)),ids.size(),c.hidden,group,bits);check_operator(label+"/embedding",expected,value);
 }
 void check_add(const mx::array& a,const mx::array& b,const mx::array& value,const std::string& label){auto av=cpu_values(a),bv=cpu_values(b);if(av.size()!=bv.size())throw std::invalid_argument("addition oracle shape");std::vector<double> expected;for(size_t i=0;i<av.size();++i)expected.push_back(double(av[i])+bv[i]);check_operator(label,expected,value);}
 void check_gate(const mx::array& a,const mx::array& gate,const mx::array& value,const std::string& label,bool use_silu=false){auto av=cpu_values(a),gv=cpu_values(gate);if(av.size()!=gv.size())throw std::invalid_argument("gate oracle shape");std::vector<double> expected;for(size_t i=0;i<av.size();++i)expected.push_back(double(av[i])*scalar_reference::sigmoid(gv[i])*(use_silu?gv[i]:1.));check_operator(label,expected,value);}
 void check_routing(const mx::array& logits,const mx::array& gates,const mx::array& indices,const mx::array& scores,const std::string& label){
  auto chosen=mx::contiguous(mx::astype(indices,mx::int32,s),false,s);mx::eval(chosen);mx::synchronize(s);const int32_t* selected=chosen.data<int32_t>();
  auto expected=scalar_reference::routing(cpu_values(logits),{selected,selected+chosen.size()},logits.shape(0),c.experts,c.top);
  check_operator(label+"/softmax",expected.probabilities,gates);check_operator(label+"/routing-weights",expected.weights,scores);
 }
 void check_mixture(const mx::array& selected,const mx::array& scores,const mx::array& mixed,const mx::array& shared,const mx::array& shared_gate,const mx::array& result,const std::string& label){
  auto sv=cpu_values(selected),weights=cpu_values(scores),mv=cpu_values(mixed),sh=cpu_values(shared),gate=cpu_values(shared_gate);std::vector<double> expected(mv.size(),0.),merged(mv.size());
  for(int row=0;row<mixed.shape(0);++row)for(int ch=0;ch<c.hidden;++ch){size_t pos=size_t(row)*c.hidden+ch;for(int j=0;j<c.top;++j)expected[pos]+=double(sv[(size_t(row)*c.top+j)*c.hidden+ch])*weights[row*c.top+j];merged[pos]=double(mv[pos])+double(sh[pos])*scalar_reference::sigmoid(gate[row]);}
  check_operator(label+"/mixture",expected,mixed);check_operator(label+"/shared-gating-add",merged,result);
 }
