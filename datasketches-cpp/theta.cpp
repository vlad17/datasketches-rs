#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"

#include "theta/include/theta_sketch.hpp"
#include "theta/include/theta_union.hpp"
#include "theta/include/theta_intersection.hpp"
#include "theta/include/theta_a_not_b.hpp"
#include "theta.hpp"

double OpaqueThetaSketch::estimate() const {
  return this->inner_.get_estimate();
}

void OpaqueThetaSketch::update(rust::Slice<const uint8_t> buf) {
  this->inner_.update(buf.data(), buf.size());
}

void OpaqueThetaSketch::update_u64(uint64_t value) {
  this->inner_.update(value);
}

std::unique_ptr<OpaqueStaticThetaSketch> OpaqueThetaSketch::as_static() const{
  auto compact = this->inner_.compact();
  auto ptr = new OpaqueStaticThetaSketch{std::move(compact)};
  return std::unique_ptr<OpaqueStaticThetaSketch>(ptr);
}

OpaqueThetaSketch::OpaqueThetaSketch():
  inner_{datasketches::update_theta_sketch::builder{}.build()} {
}

OpaqueThetaSketch::OpaqueThetaSketch(datasketches::update_theta_sketch&& theta):
  inner_{std::move(theta)} {
}

std::unique_ptr<OpaqueThetaSketch> new_opaque_theta_sketch() {
  return std::unique_ptr<OpaqueThetaSketch>(new OpaqueThetaSketch{});
}

OpaqueStaticThetaSketch::OpaqueStaticThetaSketch(const datasketches::compact_theta_sketch& theta):
  inner_{theta} {
}

OpaqueStaticThetaSketch::OpaqueStaticThetaSketch(datasketches::compact_theta_sketch&& theta):
  inner_{std::move(theta)} {
}

OpaqueStaticThetaSketch::OpaqueStaticThetaSketch(std::istream& is):
  inner_{datasketches::compact_theta_sketch::deserialize(is)} {
}

double OpaqueStaticThetaSketch::estimate() const {
  return this->inner_.get_estimate();
}

std::unique_ptr<OpaqueStaticThetaSketch> OpaqueStaticThetaSketch::clone() const {
  return std::unique_ptr<OpaqueStaticThetaSketch>(new OpaqueStaticThetaSketch{this->inner_});
}

void OpaqueStaticThetaSketch::set_difference(const OpaqueStaticThetaSketch& other) {
  datasketches::theta_a_not_b a_not_b;
  auto result = a_not_b.compute(std::move(this->inner_), other.inner_);
  this->inner_ = std::move(result);
}

std::unique_ptr<std::vector<uint8_t>> OpaqueStaticThetaSketch::serialize() const {
  auto v = this->inner_.serialize();
  auto ptr = new std::vector<uint8_t>(std::move(v));
  return std::unique_ptr<std::vector<uint8_t>>(ptr);
  /*    // TODO: could use a custom streambuf to avoid the
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
  */
}

std::unique_ptr<OpaqueStaticThetaSketch> deserialize_opaque_static_theta_sketch(rust::Slice<const uint8_t> buf) {
  // TODO: could use a custom streambuf to avoid the slice -> stream copy
  std::stringstream s{};
  s.write(const_cast<char*>(reinterpret_cast<const char*>(buf.data())), std::streamsize(buf.size()));
  s.seekg(0, std::ios::beg);
  return std::unique_ptr<OpaqueStaticThetaSketch>(new OpaqueStaticThetaSketch{s});
}

OpaqueThetaUnion::OpaqueThetaUnion():
  inner_{datasketches::theta_union::builder{}.build()} {
}

std::unique_ptr<OpaqueStaticThetaSketch> OpaqueThetaUnion::sketch() const {
  auto result = this->inner_.get_result();
  auto ptr = new OpaqueStaticThetaSketch{std::move(result)};
  return std::unique_ptr<OpaqueStaticThetaSketch>(ptr);
}

void OpaqueThetaUnion::union_with(std::unique_ptr<OpaqueStaticThetaSketch> to_union) {
  this->inner_.update(std::move(to_union->inner_));
}

std::unique_ptr<OpaqueThetaUnion> new_opaque_theta_union() {
  return std::unique_ptr<OpaqueThetaUnion>(new OpaqueThetaUnion{});
}

OpaqueThetaIntersection::OpaqueThetaIntersection():
  inner_{} {
}

std::unique_ptr<OpaqueStaticThetaSketch> OpaqueThetaIntersection::sketch() const {
  if (!this->inner_.has_result()) {
    return std::unique_ptr<OpaqueStaticThetaSketch>(nullptr);
  }
  auto value = this->inner_.get_result();
  auto ptr = new OpaqueStaticThetaSketch{value};
  return std::unique_ptr<OpaqueStaticThetaSketch>(ptr);
}

void OpaqueThetaIntersection::intersect_with(std::unique_ptr<OpaqueStaticThetaSketch> to_intersect) {
  this->inner_.update(std::move(to_intersect->inner_));
}

std::unique_ptr<OpaqueThetaIntersection> new_opaque_theta_intersection() {
  return std::unique_ptr<OpaqueThetaIntersection>(new OpaqueThetaIntersection{});
}
