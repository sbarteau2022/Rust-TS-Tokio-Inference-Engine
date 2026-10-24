#pragma once
#include <mlx/mlx.h>
#include <utility>
// IEEE total-order keys match Rust f32::total_cmp for finite values, including
// -0.0 < +0.0. MLX argmax retains the lowest index for equal integer keys.
inline mlx::core::array device_total_argmax(const mlx::core::array& logits, mlx::core::Stream stream) {
 namespace mx=mlx::core;
 if(logits.dtype()!=mx::float32||logits.ndim()<1)throw std::invalid_argument("selection requires float32 logits");
 auto bits=mx::view(logits,mx::uint32,stream);
 auto sign=mx::array(uint32_t(0x80000000));
 auto negative=mx::not_equal(mx::bitwise_and(bits,sign,stream),mx::array(uint32_t(0)),stream);
 auto keys=mx::where(negative,mx::bitwise_invert(bits,stream),mx::bitwise_xor(bits,sign,stream),stream);
 return mx::argmax(keys,-1,false,stream);
}
// On-GPU top-1 token ID (with IEEE total-order tie-breaking) and top-1 vs top-2
// float32 logit margin, eliminating 248,320-float (993 KB) host logit copies.
inline std::pair<mlx::core::array,mlx::core::array> device_top2_margin(const mlx::core::array& logits, mlx::core::Stream stream) {
 namespace mx=mlx::core;
 auto flat=mx::reshape(logits,{-1},stream);
 auto winner=device_total_argmax(flat,stream);
 int vocab=flat.shape(0);
 if(vocab<2)return {winner,mx::array(std::numeric_limits<float>::infinity())};
 auto top2_idx=mx::slice(mx::argpartition(flat,-2,-1,stream),{vocab-2},{vocab},stream);
 auto top2_vals=mx::take(flat,top2_idx,0,stream);
 auto v0=mx::take(top2_vals,0,0,stream),v1=mx::take(top2_vals,1,0,stream);
 auto margin=mx::abs(mx::subtract(v1,v0,stream),stream);
 return {winner,margin};
}
