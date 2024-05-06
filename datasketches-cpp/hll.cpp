#include <cstdint>
#include <ios>
#include <sstream>
#include <iostream>

#include "rust/cxx.h"
#include "hll/include/hll.hpp"

#include "hll.hpp"

OpaqueHLLSketch::OpaqueHLLSketch(unsigned lg_k, datasketches::target_hll_type tgt_type):
  inner_{ datasketches::hll_sketch(lg_k, tgt_type) } {
}

OpaqueHLLSketch::OpaqueHLLSketch(datasketches::hll_sketch&& hll):
  inner_{std::move(hll)} {
}

OpaqueHLLSketch::OpaqueHLLSketch(std::istream& is):
  inner_{datasketches::hll_sketch::deserialize(is)} {
}

double OpaqueHLLSketch::estimate() const {
  return this->inner_.get_estimate();
}

void OpaqueHLLSketch::update(rust::Slice<const uint8_t> buf) {
  this->inner_.update(buf.data(), buf.size());
}

void OpaqueHLLSketch::update_u64(uint64_t value) {
  this->inner_.update(value);
}

std::unique_ptr<std::vector<uint8_t>> OpaqueHLLSketch::serialize() const {
  // TODO: could use a custom streambuf to avoid the
  // stream -> vec copy https://stackoverflow.com/a/13059195/1779853
  std::stringstream s{};
  auto start = s.tellg();
  this->inner_.serialize_compact(s);
  s.seekg(0, std::ios::end);
  auto stop = s.tellg();

  std::vector<uint8_t> v(std::size_t(stop-start));
  s.seekg(0, std::ios::beg);
  s.read(reinterpret_cast<char*>(v.data()), std::streamsize(v.size()));

  return std::unique_ptr<std::vector<uint8_t>>(new std::vector<uint8_t>(std::move(v)));
}

std::unique_ptr<OpaqueHLLSketch> new_opaque_hll_sketch(unsigned lg_k, datasketches::target_hll_type tgt_type) {
  return std::unique_ptr<OpaqueHLLSketch>(new OpaqueHLLSketch { lg_k, tgt_type });
}

std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf) {
    // TODO: could use a custom streambuf to avoid the slice -> stream copy
    std::stringstream s{};
    s.write(const_cast<char*>(reinterpret_cast<const char*>(buf.data())), std::streamsize(buf.size()));
    s.seekg(0, std::ios::beg);
    return std::unique_ptr<OpaqueHLLSketch>(new OpaqueHLLSketch{s});
}

OpaqueHLLUnion::OpaqueHLLUnion(uint8_t lg_max_k):
  inner_{ datasketches::hll_union(lg_max_k) } {
}

std::unique_ptr<OpaqueHLLSketch> OpaqueHLLUnion::sketch(datasketches::target_hll_type tgt_type) const {
  return std::unique_ptr<OpaqueHLLSketch>(new OpaqueHLLSketch{this->inner_.get_result(tgt_type)});
}

void OpaqueHLLUnion::merge(std::unique_ptr<OpaqueHLLSketch> to_add) {
  this->inner_.update(std::move(to_add->inner_));
}


std::unique_ptr<OpaqueHLLUnion> new_opaque_hll_union(uint8_t lg_max_k) {
  return std::unique_ptr<OpaqueHLLUnion>(new OpaqueHLLUnion{ lg_max_k });
}
