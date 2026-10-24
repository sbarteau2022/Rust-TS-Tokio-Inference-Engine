#pragma once
#include <mlx/mlx.h>
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
