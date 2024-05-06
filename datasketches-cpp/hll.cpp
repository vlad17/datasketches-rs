#include <cstdint>
#include <ios>
#include <sstream>
#include <iostream>

#include "rust/cxx.h"
#include "hll/include/hll.hpp"

#include "hll.hpp"

OpaqueHLLSketch::OpaqueHLLSketch(unsigned lg_k):
  inner_{ datasketches::hll_sketch(lg_k) } {
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

//std::unique_ptr<std::vector<uint8_t>> OpaqueHLLSketch::serialize() const {
//  // TODO: could use a custom streambuf to avoid the
//  // stream -> vec copy https://stackoverflow.com/a/13059195/1779853
//  std::stringstream s{};
//  auto start = s.tellg();
//  this->inner_.serialize(s);
//  s.seekg(0, std::ios::end);
//  auto stop = s.tellg();
//
//  std::vector<uint8_t> v(std::size_t(stop-start));
//  s.seekg(0, std::ios::beg);
//  s.read(reinterpret_cast<char*>(v.data()), std::streamsize(v.size()));
//
//  return std::unique_ptr<std::vector<uint8_t>>(new std::vector<uint8_t>(std::move(v)));
//}

std::unique_ptr<OpaqueHLLSketch> new_opaque_hll_sketch(unsigned lg_k) {
  return std::unique_ptr<OpaqueHLLSketch>(new OpaqueHLLSketch { lg_k });
}

std::unique_ptr<OpaqueHLLSketch> deserialize_opaque_hll_sketch(rust::Slice<const uint8_t> buf) {
    // TODO: could use a custom streambuf to avoid the slice -> stream copy
    std::stringstream s{};
    s.write(const_cast<char*>(reinterpret_cast<const char*>(buf.data())), std::streamsize(buf.size()));
    s.seekg(0, std::ios::beg);
    return std::unique_ptr<OpaqueHLLSketch>(new OpaqueHLLSketch{s});
}
