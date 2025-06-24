#pragma once

#include <cstdint>
#include <iostream>
#include <vector>
#include <memory>

#include "rust/cxx.h"
#include "kll/include/kll_sketch.hpp"

// KLL sketch for float values
class OpaqueKllFloatSketch {
public:
  void kll_float_update(float value);
  void kll_float_merge(const OpaqueKllFloatSketch& other);
  bool is_empty() const;
  uint16_t get_k() const;
  uint64_t get_n() const;
  uint32_t get_num_retained() const;
  bool is_estimation_mode() const;
  float get_min_value() const;
  float get_max_value() const;
  float get_quantile(double fraction) const;
  std::unique_ptr<std::vector<float>> get_quantiles(rust::Slice<const double> fractions) const;
  std::unique_ptr<std::vector<float>> get_quantiles_evenly_spaced(uint32_t num) const;
  double get_rank(float value) const;
  std::unique_ptr<std::vector<uint8_t>> serialize() const;

private:
  OpaqueKllFloatSketch();
  OpaqueKllFloatSketch(uint16_t k);
  OpaqueKllFloatSketch(std::istream& is);
  friend std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch();
  friend std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch_with_k(uint16_t k);
  friend std::unique_ptr<OpaqueKllFloatSketch> deserialize_opaque_kll_float_sketch(rust::Slice<const uint8_t> buf);
  datasketches::kll_sketch<float> inner_;
};

std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch();
std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch_with_k(uint16_t k);
std::unique_ptr<OpaqueKllFloatSketch> deserialize_opaque_kll_float_sketch(rust::Slice<const uint8_t> buf);

// KLL sketch for double values
class OpaqueKllDoubleSketch {
public:
  void kll_double_update(double value);
  void kll_double_merge(const OpaqueKllDoubleSketch& other);
  bool is_empty() const;
  uint16_t get_k() const;
  uint64_t get_n() const;
  uint32_t get_num_retained() const;
  bool is_estimation_mode() const;
  double get_min_value() const;
  double get_max_value() const;
  double get_quantile(double fraction) const;
  std::unique_ptr<std::vector<double>> get_quantiles(rust::Slice<const double> fractions) const;
  std::unique_ptr<std::vector<double>> get_quantiles_evenly_spaced(uint32_t num) const;
  double get_rank(double value) const;
  std::unique_ptr<std::vector<uint8_t>> serialize() const;

private:
  OpaqueKllDoubleSketch();
  OpaqueKllDoubleSketch(uint16_t k);
  OpaqueKllDoubleSketch(std::istream& is);
  friend std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch();
  friend std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch_with_k(uint16_t k);
  friend std::unique_ptr<OpaqueKllDoubleSketch> deserialize_opaque_kll_double_sketch(rust::Slice<const uint8_t> buf);
  datasketches::kll_sketch<double> inner_;
};

std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch();
std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch_with_k(uint16_t k);
std::unique_ptr<OpaqueKllDoubleSketch> deserialize_opaque_kll_double_sketch(rust::Slice<const uint8_t> buf);