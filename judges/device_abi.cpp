#include "mlx_bridge.h"
#include "state_table.h"
#include <cassert>
#include <iostream>
#include <vector>
int main(){
 rs_qwen_config c={2048,40,248320,16,2,256,16,32,128,128,4,4,256,8,512,512,64,4,512,8,64,1,1e-6f,10000000.f};
 rs_qwen_layer layers[40];for(unsigned i=0;i<40;++i)layers[i]={i,uint32_t((i+1)%4==0)};
 char error[1024]={};int sentinel=0;void* fake=&sentinel;
 assert(!rs_qwen_device_create(nullptr,0,&c,layers,40,2,1<<28,error,sizeof(error)));
 auto bad=c;bad.interval=0;assert(!rs_qwen_device_create(fake,0,&bad,layers,40,2,1<<28,error,sizeof(error)));
 assert(!rs_qwen_device_create(fake,0,&c,layers,40,65,1<<28,error,sizeof(error)));
 void* session=rs_qwen_device_create(fake,0,&c,layers,40,2,1<<28,error,sizeof(error));assert(session);
 std::vector<float> logits(c.vocab);uint32_t invalid=uint32_t(c.vocab);StateId output={9,9,9,9,9};uint64_t stats[7];size_t checks=3;
 for(size_t i=0;i<14000;++i){
  StateId foreign={i+1,i%64,i+1,1,4};
  assert(rs_qwen_device_release(session,&foreign,error,sizeof(error))!=0);
  assert(rs_qwen_device_step(session,&foreign,1,&invalid,1,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);
  // Valid empty descriptor reaches a reservation then fails token validation before weights access.
  assert(rs_qwen_device_step(session,nullptr,0,&invalid,1,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);
  assert(output.owner==9&&output.slot==9&&output.generation==9);
  assert(rs_qwen_device_stats(session,stats,7,error,sizeof(error))==0);assert(stats[0]==0&&stats[1]==0&&stats[4]==0);checks+=5;
 }
 assert(rs_qwen_device_step(session,nullptr,512,&invalid,1,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);
 assert(rs_qwen_device_stats(session,stats,6,error,sizeof(error))!=0);
 uint32_t valid=0;float scalar=0;StateId forged={999,0,1,1,4};
 assert(rs_qwen_device_step(session,nullptr,0,&valid,1,&output,logits.data(),logits.size(),&scalar,1,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_step(session,nullptr,0,&valid,1,nullptr,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_step(session,nullptr,0,nullptr,1,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_step(session,nullptr,0,&valid,0,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_step(session,nullptr,0,&valid,9,&output,logits.data(),logits.size(),nullptr,0,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_step(session,nullptr,0,&valid,1,&output,logits.data(),1,nullptr,0,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_snapshot(session,&forged,&scalar,1,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_snapshot(session,&forged,nullptr,1,error,sizeof(error))!=0);++checks;
 uint64_t invalidated=99;assert(rs_qwen_device_drain(session,&invalidated,error,sizeof(error))==0&&invalidated==0);++checks;
 assert(rs_qwen_device_stats(session,stats,7,error,sizeof(error))==0&&stats[0]==0&&stats[1]==0&&stats[4]==0);++checks;
 // Block output count and input validation reject before sentinel weight dereference.
 for(size_t n=0;n<=10;++n){
  std::vector<uint32_t> ids(n,invalid);std::vector<float> rows(size_t(c.vocab)*std::min(n,size_t(8)));
  assert(rs_qwen_device_block(session,nullptr,0,ids.data(),n,&output,rows.data(),rows.size(),error,sizeof(error))!=0);++checks;
  assert(output.owner==9&&output.slot==9&&output.generation==9);
 }
 assert(rs_qwen_device_block(session,nullptr,0,&valid,1,&output,&scalar,1,error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_block(session,&forged,1,&valid,1,&output,logits.data(),logits.size(),error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_block(session,nullptr,0,&valid,1,nullptr,logits.data(),logits.size(),error,sizeof(error))!=0);++checks;
 assert(rs_qwen_device_stats(session,stats,7,error,sizeof(error))==0&&stats[0]==0&&stats[1]==0&&stats[4]==0);++checks;
 rs_qwen_device_destroy(session);rs_qwen_device_destroy(nullptr);rs_mlx_cleanup();
 std::cout<<"PASS device ABI checks="<<checks+2<<" ASan/UBSan (upstream MLX uninstrumented)\n";
}
