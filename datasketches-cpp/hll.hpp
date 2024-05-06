#pragma once

#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"
#include "hll/include/hll.hpp"

// alias
typedef datasketches::target_hll_type target_hll_type;

class OpaqueHLLSketch {
public:
  double estimate() const;
  void update(rust::Slice<const uint8_t> buf);
  void update_u64(uint64_t value);
  std::unique_ptr<std::vector<uint8_t>> serialize() const;
  friend std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf);
  OpaqueHLLSketch(unsigned lg_k, datasketches::target_hll_type tgt_type);
private:
  OpaqueHLLSketch(datasketches::hll_sketch&& hll);
  OpaqueHLLSketch(std::istream& is);
  friend class OpaqueHLLUnion;
  datasketches::hll_sketch inner_;
};

std::unique_ptr<OpaqueHLLSketch> new_opaque_hll_sketch(unsigned lg_k, datasketches::target_hll_type tgt_type);
std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf);

class OpaqueHLLUnion {
public:
  std::unique_ptr<OpaqueHLLSketch> sketch(datasketches::target_hll_type tgt_type) const;
  void merge(std::unique_ptr<OpaqueHLLSketch> to_add);
  OpaqueHLLUnion(uint8_t lg_max_k);
private:
  datasketches::hll_union inner_;
};

std::unique_ptr<OpaqueHLLUnion> new_opaque_hll_union(uint8_t lg_max_k);
