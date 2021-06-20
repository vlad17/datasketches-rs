#pragma once

#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"
#include "cpc/include/cpc_sketch.hpp"
#include "cpc/include/cpc_union.hpp"

class OpaqueCpcSketch {
public:
  double estimate() const;
  void update(rust::Slice<const uint8_t> buf);
  void update_u64(uint64_t value);
  std::unique_ptr<std::vector<uint8_t>> serialize() const;
private:
  OpaqueCpcSketch();
  OpaqueCpcSketch(datasketches::cpc_sketch&& cpc);
  OpaqueCpcSketch(std::istream& is);
  friend std::unique_ptr<OpaqueCpcSketch> new_opaque_cpc_sketch();
  friend std::unique_ptr<OpaqueCpcSketch> deserialize_opaque_cpc_sketch(rust::Slice<const uint8_t> buf);
  friend class OpaqueCpcUnion;
  datasketches::cpc_sketch inner_;
};

std::unique_ptr<OpaqueCpcSketch> new_opaque_cpc_sketch();
std::unique_ptr<OpaqueCpcSketch> deserialize_opaque_cpc_sketch(rust::Slice<const uint8_t> buf);

class OpaqueCpcUnion {
public:
  std::unique_ptr<OpaqueCpcSketch> sketch() const;
  void merge(std::unique_ptr<OpaqueCpcSketch> to_add);
private:
  OpaqueCpcUnion();
  datasketches::cpc_union inner_;
  friend std::unique_ptr<OpaqueCpcUnion> new_opaque_cpc_union();
};

std::unique_ptr<OpaqueCpcUnion> new_opaque_cpc_union();
