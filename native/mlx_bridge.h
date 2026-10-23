#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
// All handles and data pointers are private to the Rust-owned executor thread.
void* rs_mlx_create(int gpu, char* error, size_t capacity);
void rs_mlx_destroy(void* graph);
int rs_mlx_input(void* graph, int dtype, const void* data, const int32_t* shape,
                 size_t rank, char* error, size_t capacity);
int rs_mlx_op(void* graph, int op, const uint32_t* inputs, size_t count,
              int group, float epsilon, char* error, size_t capacity);
int rs_mlx_finish(void* graph, uint32_t output, void* data, size_t elements,
                  char* error, size_t capacity);
const char* rs_mlx_version(void);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
void rs_mlx_cleanup(void);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
void* rs_mlx_weights_create(char* error, size_t capacity);
void rs_mlx_weights_destroy(void* store);
int rs_mlx_weights_load(void* store, const char* file, char* error, size_t capacity);
int rs_mlx_parameter(void* graph, void* store, const char* name, const int32_t* shape,
                     size_t rank, int dtype, char* error, size_t capacity);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
int rs_mlx_kernel(void* graph, int op, const uint32_t* inputs, size_t count,
                  const int32_t* ints, size_t ni, const float* floats, size_t nf,
                  char* error, size_t capacity);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
typedef struct {
 int32_t hidden,layers,vocab,q_heads,kv_heads,head_dim,key_heads,value_heads,key_dim,value_dim,conv,interval,experts,top,intermediate,shared,group,bits,context,chunk,rope_dims,activation_precision;
 float epsilon,theta;
} rs_qwen_config;
typedef struct {uint32_t index,mixer;} rs_qwen_layer;
int rs_qwen_quant(void* store,const char* module,int group,int bits,char* error,size_t capacity);
int rs_qwen_forward(void* store,int gpu,const rs_qwen_config* config,const rs_qwen_layer* layers,size_t layer_count,const uint32_t* tokens,size_t count,size_t offset,
 const float* old_state,size_t old_count,float* new_state,size_t new_count,float* logits,size_t logits_count,int projection_checks,const uint32_t* projection_outputs,size_t projection_output_count,float* trace,size_t trace_count,char* error,size_t capacity);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
int rs_mlx_weight_bytes(void* store,const char* name,const int32_t* shape,size_t rank,int dtype,size_t offset,uint8_t* output,size_t count,char* error,size_t capacity);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
extern "C" {
#endif
/* Session handles are worker-confined; state IDs are validated value descriptors. */
#ifndef RS_QWEN_STATE_ID_DEFINED
#define RS_QWEN_STATE_ID_DEFINED
typedef struct StateId {uint64_t owner,slot,generation,position,bytes;} StateId;
#endif
void* rs_qwen_device_create(void*,int,const rs_qwen_config*,const rs_qwen_layer*,size_t,size_t,uint64_t,char*,size_t);
int rs_qwen_device_step(void*,const struct StateId*,size_t,const uint32_t*,size_t,struct StateId*,float*,size_t,float*,size_t,char*,size_t);
/* Position-major logits: row i predicts the token after input token i. */
int rs_qwen_device_block(void*,const struct StateId*,size_t,const uint32_t*,size_t,struct StateId*,float*,size_t,char*,size_t);
int rs_qwen_device_release(void*,const struct StateId*,char*,size_t);
int rs_qwen_device_snapshot(void*,const struct StateId*,float*,size_t,char*,size_t);
int rs_qwen_device_stats(void*,uint64_t*,size_t,char*,size_t);
int rs_qwen_device_drain(void*,uint64_t*,char*,size_t);
void rs_qwen_device_destroy(void*);
#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
extern "C" {
#endif
int rs_qwen_device_hidden_block(void*,const struct StateId*,size_t,const uint32_t*,size_t,struct StateId*,float*,size_t,char*,size_t);
int rs_qwen_device_hidden_snapshot(void*,const struct StateId*,float*,size_t,char*,size_t);
#ifdef __cplusplus
}
#endif
