#pragma once
#include <mlx/mlx.h>
#include <stdexcept>
#include <string>
#include <vector>

namespace rs {
// Outside compile tracing: share immutable parameter payloads across shape caches.
// The owner remains responsible for destroying callables before these arrays.
template <class Arrays, class Prefixes>
bool prepare_compiled_parameters(Arrays& arrays, Prefixes& prepared,
                                 const std::string& prefix, mlx::core::Stream stream) {
  if (prefix.empty() || prefix.back() == '.')
    throw std::invalid_argument("compiled parameter prefix");
  if (prepared.contains(prefix)) return false;
  if (prepared.size() >= 64) throw std::runtime_error("compiled parameter prefix cap");
  std::vector<mlx::core::array> parameters;
  const auto start = prefix + ".";
  for (const auto& [name, value] : arrays)
    if (name.starts_with(start)) parameters.push_back(value);
  if (parameters.empty()) throw std::invalid_argument("compiled parameter prefix absent");
  mlx::core::eval(parameters);
  mlx::core::synchronize(stream);
  prepared.insert(prefix); // An evaluation failure must not mark the prefix ready.
  return true;
}
}
