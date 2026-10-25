#!/bin/sh
# Standalone native kernel control; no Rust/TS/Tokio runtime in the executable.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sdk=${MLX_ROOT:-"$root/vendor/mlx"}
output=${1:?usage: build_native_cpp_bench.sh OUTPUT_BINARY}
exec c++ -std=c++20 -O3 -DNDEBUG -I"$root/native" -I"$sdk/include" \
  "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" \
  -L"$sdk/lib" -lmlx -Wl,-rpath,"$sdk/lib" -o "$output"
