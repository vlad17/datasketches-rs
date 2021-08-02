#pragma once

#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"

#include "theta/include/theta_sketch.hpp"
#include "theta/include/theta_union.hpp"
#include "theta/include/theta_intersection.hpp"

class OpaqueStaticThetaSketch;

class OpaqueThetaSketch {
public:
  double estimate() const;
  void update(rust::Slice<const uint8_t> buf);
  void update_u64(uint64_t value);
  std::unique_ptr<OpaqueStaticThetaSketch> as_static() const;
private:
  OpaqueThetaSketch();
  OpaqueThetaSketch(datasketches::update_theta_sketch&& theta);
  friend std::unique_ptr<OpaqueThetaSketch> new_opaque_theta_sketch();
  datasketches::update_theta_sketch inner_;
};

std::unique_ptr<OpaqueThetaSketch> new_opaque_theta_sketch();

class OpaqueStaticThetaSketch {
public:
  double estimate() const;
  std::unique_ptr<OpaqueStaticThetaSketch> clone() const;
  void set_difference(const OpaqueStaticThetaSketch& other);
  std::unique_ptr<std::vector<uint8_t>> serialize() const;
private:
  OpaqueStaticThetaSketch(const datasketches::compact_theta_sketch& theta);
  OpaqueStaticThetaSketch(datasketches::compact_theta_sketch&& theta);
  OpaqueStaticThetaSketch(std::istream& is);
  friend std::unique_ptr<OpaqueStaticThetaSketch> deserialize_opaque_static_theta_sketch(rust::Slice<const uint8_t> buf);
  friend class OpaqueThetaSketch;
  friend class OpaqueThetaUnion;
  friend class OpaqueThetaIntersection;
  datasketches::compact_theta_sketch inner_;
};

std::unique_ptr<OpaqueStaticThetaSketch> deserialize_opaque_static_theta_sketch(rust::Slice<const uint8_t> buf);

class OpaqueThetaUnion {
public:
  std::unique_ptr<OpaqueStaticThetaSketch> sketch() const;
  void union_with(std::unique_ptr<OpaqueStaticThetaSketch> to_union);
private:
  OpaqueThetaUnion();
  datasketches::theta_union inner_;
  friend std::unique_ptr<OpaqueThetaUnion> new_opaque_theta_union();
};

std::unique_ptr<OpaqueThetaUnion> new_opaque_theta_union();

class OpaqueThetaIntersection {
public:
  // Null if the intersection is over an empty collection, i.e., the sketch
  // implicitly represents the full universe of items.
  std::unique_ptr<OpaqueStaticThetaSketch> sketch() const;
  void intersect_with(std::unique_ptr<OpaqueStaticThetaSketch> to_intersect);
private:
  OpaqueThetaIntersection();
  datasketches::theta_intersection inner_;
  friend std::unique_ptr<OpaqueThetaIntersection> new_opaque_theta_intersection();
};

std::unique_ptr<OpaqueThetaIntersection> new_opaque_theta_intersection();
