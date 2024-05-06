#pragma once

#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"
#include "hll/include/hll.hpp"

class OpaqueHLLSketch {
public:
  double estimate() const;
  void update(rust::Slice<const uint8_t> buf);
  void update_u64(uint64_t value);
  std::unique_ptr<std::vector<uint8_t>> serialize() const;
  friend std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf);
  OpaqueHLLSketch(unsigned lg_k);
private:
  OpaqueHLLSketch(std::istream& is);
  datasketches::hll_sketch inner_;
};

std::unique_ptr<OpaqueHLLSketch> new_opaque_hll_sketch(unsigned lg_k);
std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf);
