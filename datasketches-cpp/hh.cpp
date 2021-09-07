#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"

#include "dsrs/src/bridge.rs.h"
#include "fi/include/frequent_items_sketch.hpp"
#include "hh.hpp"

std::unique_ptr<std::vector<ThinHeavyHitterRow>> convert_to_thin(OpaqueHhSketch::hhsketch::vector_row v) {
  std::vector<ThinHeavyHitterRow> result(v.size());
  for (std::size_t i = 0; i < v.size(); ++i) {
    auto& row = v[i];
    auto& target = result[i];
    target.addr = row.get_item();
    target.lb = row.get_lower_bound();
    target.ub = row.get_upper_bound();
  }
  auto ptr = new std::vector<ThinHeavyHitterRow>(std::move(result));
  return std::unique_ptr<std::vector<ThinHeavyHitterRow>>(ptr);
}

std::unique_ptr<std::vector<ThinHeavyHitterRow>> OpaqueHhSketch::estimate_no_fp() const {
  return convert_to_thin(this->inner_.get_frequent_items(datasketches::NO_FALSE_POSITIVES));
}

std::unique_ptr<std::vector<ThinHeavyHitterRow>> OpaqueHhSketch::estimate_no_fn() const {
  return convert_to_thin(this->inner_.get_frequent_items(datasketches::NO_FALSE_NEGATIVES));
}

void OpaqueHhSketch::update(size_t value, uint64_t weight) {
  this->inner_.update(value, weight);
}

OpaqueHhSketch::OpaqueHhSketch(hhsketch&& sketch):
  inner_{sketch} {
}

std::unique_ptr<std::vector<ThinHeavyHitterRow>> OpaqueHhSketch::state() const {
  return convert_to_thin(this->inner_.get_frequent_items(datasketches::NO_FALSE_NEGATIVES, 0));
}

void OpaqueHhSketch::set_weights(uint64_t total_weight, uint64_t offset) {
  this->inner_.set_weights(total_weight, offset);
}

uint64_t OpaqueHhSketch::get_total_weight() const {
  return this->inner_.get_total_weight();
}

uint64_t OpaqueHhSketch::get_offset() const {
  return this->inner_.get_offset();
}

std::unique_ptr<OpaqueHhSketch> new_opaque_hh_sketch(uint8_t lg2_k, size_t hashset_addr) {
  OpaqueHhSketch::hhsketch sketch(lg2_k, hashset_addr);
  auto ptr = new OpaqueHhSketch(std::move(sketch));
  return std::unique_ptr<OpaqueHhSketch>(ptr);
}
