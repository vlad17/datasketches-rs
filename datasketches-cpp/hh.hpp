#pragma once

#include <cstdint>
#include <vector>
#include <memory>

#include "rust/cxx.h"

#include "fi/include/frequent_items_sketch.hpp"

struct ThinHeavyHitterRow;

class OpaqueHhSketch {
public:
  typedef datasketches::frequent_items_sketch<size_t> hhsketch;
  std::unique_ptr<std::vector<ThinHeavyHitterRow>> estimate_no_fp() const;
  std::unique_ptr<std::vector<ThinHeavyHitterRow>> estimate_no_fn() const;
  void update(size_t value, uint64_t weight);
  std::unique_ptr<std::vector<ThinHeavyHitterRow>> state() const;
  void set_weights(uint64_t total_weight, uint64_t offset);
  uint64_t get_total_weight() const;
  uint64_t get_offset() const;
private:
  OpaqueHhSketch(hhsketch&& theta);
  friend std::unique_ptr<OpaqueHhSketch> new_opaque_hh_sketch(uint8_t lg2_k, size_t hashset_addr);
  hhsketch inner_;
};

std::unique_ptr<OpaqueHhSketch> new_opaque_hh_sketch(uint8_t lg2_k, size_t hashset_addr);
