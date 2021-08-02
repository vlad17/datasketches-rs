#include <cstdint>
#include <ios>
#include <sstream>
#include <iostream>

#include "rust/cxx.h"
#include "cpc/include/cpc_sketch.hpp"

#include "cpc.hpp"

OpaqueCpcSketch::OpaqueCpcSketch():
  inner_{} {
}

OpaqueCpcSketch::OpaqueCpcSketch(datasketches::cpc_sketch&& cpc):
  inner_{std::move(cpc)} {
}

OpaqueCpcSketch::OpaqueCpcSketch(std::istream& is):
  inner_{datasketches::cpc_sketch::deserialize(is)} {
}


double OpaqueCpcSketch::estimate() const {
  return this->inner_.get_estimate();
}

void OpaqueCpcSketch::update(rust::Slice<const uint8_t> buf) {
  this->inner_.update(buf.data(), buf.size());
}

void OpaqueCpcSketch::update_u64(uint64_t value) {
  this->inner_.update(value);
}

std::unique_ptr<std::vector<uint8_t>> OpaqueCpcSketch::serialize() const {
  // TODO: could use a custom streambuf to avoid the
  // stream -> vec copy https://stackoverflow.com/a/13059195/1779853
  std::stringstream s{};
  auto start = s.tellg();
  this->inner_.serialize(s);
  s.seekg(0, std::ios::end);
  auto stop = s.tellg();

  std::vector<uint8_t> v(std::size_t(stop-start));
  s.seekg(0, std::ios::beg);
  s.read(reinterpret_cast<char*>(v.data()), std::streamsize(v.size()));

  return std::unique_ptr<std::vector<uint8_t>>(new std::vector<uint8_t>(std::move(v)));
}

std::unique_ptr<OpaqueCpcSketch> new_opaque_cpc_sketch() {
  return std::unique_ptr<OpaqueCpcSketch>(new OpaqueCpcSketch{});
}

std::unique_ptr<OpaqueCpcSketch> deserialize_opaque_cpc_sketch(rust::Slice<const uint8_t> buf) {
  // TODO: could use a custom streambuf to avoid the slice -> stream copy
  std::stringstream s{};
  s.write(const_cast<char*>(reinterpret_cast<const char*>(buf.data())), std::streamsize(buf.size()));
  s.seekg(0, std::ios::beg);
  return std::unique_ptr<OpaqueCpcSketch>(new OpaqueCpcSketch{s});
}

OpaqueCpcUnion::OpaqueCpcUnion():
  inner_{} {
}

std::unique_ptr<OpaqueCpcSketch> OpaqueCpcUnion::sketch() const {
  return std::unique_ptr<OpaqueCpcSketch>(new OpaqueCpcSketch{this->inner_.get_result()});
}

void OpaqueCpcUnion::merge(std::unique_ptr<OpaqueCpcSketch> to_add) {
  this->inner_.update(std::move(to_add->inner_));
}


std::unique_ptr<OpaqueCpcUnion> new_opaque_cpc_union() {
  return std::unique_ptr<OpaqueCpcUnion>(new OpaqueCpcUnion{});
}
