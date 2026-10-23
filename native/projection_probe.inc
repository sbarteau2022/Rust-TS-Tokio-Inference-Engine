// Included inside Qwen: opt-in bounded CPU snapshots of sampled affine outputs.
 void check_projection(const mx::array& input,const mx::array& output,const mx::array& packed,const mx::array& scales,const mx::array& biases,int group,int bits,const std::string& label){
  int width=input.shape(-1),outputs=packed.shape(0),samples=std::min(8,outputs);
  if(input.size()/width>8||width>32768||packed.ndim()!=2)throw std::invalid_argument("projection probe bounds");
  for(int start=0;start<(full_reference?outputs:1);start+=256){
  std::vector<int32_t> selected;for(int i=0;i<samples;++i)selected.push_back(samples==1?0:i*(outputs-1)/(samples-1));
  if(full_reference){samples=std::min(256,outputs-start);selected.clear();for(int i=0;i<samples;++i)selected.push_back(start+i);}
  if(!full_reference&&label=="language_model.lm_head"&&!head_channels.empty()){selected=head_channels;samples=int(selected.size());}
  auto ids=mx::array(selected.data(),{samples},mx::int32);
  std::vector<mx::array> arrays={mx::reshape(input,{-1,width},s),mx::take(mx::reshape(output,{-1,outputs},s),ids,1,s),mx::take(packed,ids,0,s),mx::take(scales,ids,0,s),mx::take(biases,ids,0,s)};
  for(size_t i=0;i<arrays.size();++i)arrays[i]=mx::contiguous(i==2?arrays[i]:mx::astype(arrays[i],mx::float32,s),false,s);
  mx::eval(arrays);mx::synchronize(s);
  auto floats=[&](size_t i){const float* data=arrays[i].data<float>();return std::vector<float>(data,data+arrays[i].size());};
  const uint32_t* data=arrays[2].data<uint32_t>();
  auto expected=projection_reference(floats(0),{data,data+arrays[2].size()},floats(3),floats(4),arrays[0].shape(0),samples,width,group,bits);
  const float* actual=arrays[1].data<float>();double error=0,norm=0,maximum=0;
  for(size_t i=0;i<expected.size();++i){double delta=actual[i]-expected[i];error+=delta*delta;norm+=expected[i]*expected[i];maximum=std::max(maximum,std::abs(delta));}
  double nrmse=std::sqrt(error/std::max(norm,1e-30));
  std::fprintf(stderr,"PROJECTION-ORACLE module=%s bits=%d group=%d rows=%d channels=%d width=%d channel_start=%d nrmse=%.9g max_abs=%.9g\n",label.c_str(),bits,group,arrays[0].shape(0),samples,width,full_reference?start:-1,nrmse,maximum);
  if(!std::isfinite(nrmse)||nrmse>1e-4)throw std::runtime_error("projection differs from sampled scalar FP64 oracle: "+label);
  }
 }

 void check_dense(const mx::array& x,const mx::array& y,const mx::array& weight,const std::string& label){
  size_t width=x.shape(-1),rows=x.size()/width,outputs=weight.shape(0);if(!rows||rows>8||weight.ndim()!=2||size_t(weight.shape(1))!=width)throw std::invalid_argument("dense probe bounds");
  auto input=cpu_values(x),matrix=cpu_values(weight);std::vector<int32_t> ids;for(int i=0;i<8;++i)ids.push_back(i*(outputs-1)/7);
  std::vector<double> expected;for(size_t row=0;row<rows;++row)for(auto out:ids){double sum=0;for(size_t col=0;col<width;++col)sum+=double(input[row*width+col])*matrix[size_t(out)*width+col];expected.push_back(sum);}
  check_operator(label+"/dense-projection",expected,mx::take(y,mx::array(ids.data(),{8},mx::int32),-1,s));
 }
