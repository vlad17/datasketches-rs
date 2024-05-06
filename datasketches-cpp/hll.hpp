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
  OpaqueHLLSketch(datasketches::hll_sketch&& hll);
  OpaqueHLLSketch(std::istream& is);
  friend class OpaqueHLLUnion;
  datasketches::hll_sketch inner_;
};

std::unique_ptr<OpaqueHLLSketch> new_opaque_hll_sketch(unsigned lg_k);
std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf);

class OpaqueHLLUnion {
public:
  std::unique_ptr<OpaqueHLLSketch> sketch() const;
  void merge(std::unique_ptr<OpaqueHLLSketch> to_add);
  OpaqueHLLUnion(uint8_t lg_max_k);
private:
  datasketches::hll_union inner_;
};

std::unique_ptr<OpaqueHLLUnion> new_opaque_hll_union(uint8_t lg_max_k);
