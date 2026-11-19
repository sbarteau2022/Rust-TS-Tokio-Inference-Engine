#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sdk=${MLX_ROOT:-"$root/vendor/mlx"}
sdk=$(CDPATH= cd -- "$sdk" && pwd)
output=${1:?usage: build_native_dspark.sh OUTPUT gate|integration|confidence|abi}
mode=${2:?usage: build_native_dspark.sh OUTPUT gate|integration|confidence|abi}
case "$mode" in
 gate) set -- "$root/judges/native_dspark_gate.cpp" ;;
 integration) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_dspark_bench.cpp" ;;
 confidence) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_dspark_bench.cpp" -DRS_DSPARK_CONFIDENCE ;;
 abi) set -- "$root/native/mlx_bridge.cpp" "$root/judges/device_abi.cpp" -fsanitize=address,undefined -fno-omit-frame-pointer ;;
 *) echo 'mode must be gate|integration|confidence|abi' >&2; exit 2 ;;
esac
exec c++ -std=c++20 -O3 -I"$root/native" -I"$sdk/include" "$@" \
 -L"$sdk/lib" -lmlx -Wl,-rpath,"$sdk/lib" -o "$output"
