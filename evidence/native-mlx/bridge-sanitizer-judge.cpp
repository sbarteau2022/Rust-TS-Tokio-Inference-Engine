// Standalone private-ABI adversarial judge; not linked into the server.
#include "mlx_bridge.h"
#include <cassert>
#include <cstdint>
#include <cstring>
int main() {
  char error[1024] = {};
  float value = 1.0f, result = 0.0f;
  int32_t shape[] = {1}, zero[] = {0};
  const uint32_t valid[] = {0, 0}, invalid[] = {UINT32_MAX, 0};
  for (int iteration = 0; iteration < 1000; ++iteration) {
    void* g = rs_mlx_create(0, error, sizeof(error));
    assert(g);
    assert(rs_mlx_input(g, 0, nullptr, shape, 1, error, sizeof(error)) == -1);
    assert(rs_mlx_input(g, 0, &value, zero, 1, error, sizeof(error)) == -1);
    assert(rs_mlx_input(g, 99, &value, shape, 1, error, sizeof(error)) == -1);
    assert(rs_mlx_input(g, 0, &value, shape, 1, error, sizeof(error)) == 0);
    assert(rs_mlx_op(g, 0, invalid, 2, 0, 0, error, sizeof(error)) == -1);
    assert(rs_mlx_op(g, 0, valid, 7, 0, 0, error, sizeof(error)) == -1);
    assert(rs_mlx_op(g, 99, valid, 2, 0, 0, error, sizeof(error)) == -1);
    assert(rs_mlx_op(g, 0, valid, 2, 0, 0, error, sizeof(error)) == 0);
    assert(rs_mlx_finish(g, UINT32_MAX, &result, 1, error, sizeof(error)) == -1);
    assert(rs_mlx_finish(g, 1, &result, 2, error, sizeof(error)) == -1);
    assert(rs_mlx_finish(g, 1, nullptr, 1, error, sizeof(error)) == -1);
    assert(rs_mlx_finish(g, 1, &result, 1, error, sizeof(error)) == 0);
    assert(result == 2.0f);
    rs_mlx_destroy(g);
  }
  {
    void* g=rs_mlx_create(1,error,sizeof(error));assert(g);
    int32_t qs[]={1,2,1,2},vs[]={1,2,2,1},bs[]={1,2,2},ss[]={1,2,1,2};
    float q[]={0.5f,0,0,0.5f},k[]={1,0,0,1},v[]={2,4,6,8},b[]={0.5f,0.5f,0.5f,0.5f},s[]={0,0,0,0};
    assert(rs_mlx_input(g,0,q,qs,4,error,sizeof(error))==0);
    assert(rs_mlx_input(g,0,k,qs,4,error,sizeof(error))==0);
    assert(rs_mlx_input(g,0,v,vs,4,error,sizeof(error))==0);
    assert(rs_mlx_input(g,0,b,bs,3,error,sizeof(error))==0);
    assert(rs_mlx_input(g,0,b,bs,3,error,sizeof(error))==0);
    assert(rs_mlx_input(g,0,s,ss,4,error,sizeof(error))==0);
    uint32_t recurrent[]={0,1,2,3,4,5},joined[]={6,7};
    assert(rs_mlx_kernel(g,3,recurrent,6,nullptr,0,nullptr,0,error,sizeof(error))==0);
    assert(rs_mlx_kernel(g,8,joined,2,nullptr,0,nullptr,0,error,sizeof(error))==0);
    float out[8]={};assert(rs_mlx_finish(g,8,out,8,error,sizeof(error))==0);
    float expected[]={0.5f,1,1.5f,2,0.5f,3,1,4};
    for(int i=0;i<8;++i)assert(out[i]==expected[i]);
    assert(rs_mlx_kernel(g,999,recurrent,6,nullptr,0,nullptr,0,error,sizeof(error))==-1);
    assert(rs_mlx_kernel(g,1,recurrent,6,nullptr,0,nullptr,0,error,sizeof(error))==-1);
    rs_mlx_destroy(g);
    void* weights=rs_mlx_weights_create(error,sizeof(error));assert(weights);
    assert(rs_mlx_weights_load(nullptr,"missing",error,sizeof(error))==-1);
    assert(rs_mlx_weights_load(weights,"/nonexistent-checkpoint-test-file",error,sizeof(error))==-1);
    rs_mlx_weights_destroy(weights);
  }
  rs_mlx_cleanup();
}
