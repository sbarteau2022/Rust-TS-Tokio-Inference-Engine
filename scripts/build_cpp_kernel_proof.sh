#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sdk=${MLX_ROOT:-"$root/vendor/mlx"}
sdk=$(CDPATH= cd -- "$sdk" && pwd)
output=${1:?usage: build_cpp_kernel_proof.sh OUTPUT reference|metal|metal-multi|gate|oracle|selection|phase|width8-m250|width8-m400}
mode=${2:?usage: build_cpp_kernel_proof.sh OUTPUT reference|metal|metal-multi|gate|oracle|selection|phase|width8-m250|width8-m400}
case "$mode" in
 reference) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG ;;
 metal-multi) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG -DRS_CPP_ATTENTION_CANDIDATE -DRS_CPP_MULTI_ROW_ONLY ;;
 metal) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG -DRS_CPP_ATTENTION_CANDIDATE ;;
 width8-m250) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG -DRS_CPP_ATTENTION_CANDIDATE -DRS_CPP_DRAFT_WIDTH=8 -DRS_CPP_DRAFT_MARGIN=2.5f ;;
 width8-m400) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG -DRS_CPP_ATTENTION_CANDIDATE -DRS_CPP_DRAFT_WIDTH=8 -DRS_CPP_DRAFT_MARGIN=4.0f ;;
 phase) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_bench.cpp" -DNDEBUG -DRS_CPP_ATTENTION_CANDIDATE -DRS_CPP_PHASE_TIMING ;;
 selection) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_selection_gate.cpp" ;;
 gate) set -- "$root/native/mlx_bridge.cpp" "$root/judges/native_cpp_attention_gate.cpp" ;;
 oracle) set -- "$root/judges/metal_reference_attention.cpp" ;;
 *) echo 'mode must be reference|metal|metal-multi|gate|oracle|selection|phase|width8-m250|width8-m400' >&2; exit 2 ;;
esac
exec c++ -std=c++20 -O3 -I"$root/native" -I"$sdk/include" "$@" \
 -L"$sdk/lib" -lmlx -Wl,-rpath,"$sdk/lib" -o "$output"
