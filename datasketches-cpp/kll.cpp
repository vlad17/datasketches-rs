#include "kll.hpp"
#include <sstream>
#include <stdexcept>

// KLL Float Sketch Implementation

OpaqueKllFloatSketch::OpaqueKllFloatSketch() : inner_() {}

OpaqueKllFloatSketch::OpaqueKllFloatSketch(uint16_t k) : inner_(k) {}

OpaqueKllFloatSketch::OpaqueKllFloatSketch(std::istream& is) : inner_(datasketches::kll_sketch<float>::deserialize(is)) {}

void OpaqueKllFloatSketch::kll_float_update(float value) {
    inner_.update(value);
}

void OpaqueKllFloatSketch::kll_float_merge(const OpaqueKllFloatSketch& other) {
    inner_.merge(other.inner_);
}

bool OpaqueKllFloatSketch::is_empty() const {
    return inner_.is_empty();
}

uint16_t OpaqueKllFloatSketch::get_k() const {
    return inner_.get_k();
}

uint64_t OpaqueKllFloatSketch::get_n() const {
    return inner_.get_n();
}

uint32_t OpaqueKllFloatSketch::get_num_retained() const {
    return inner_.get_num_retained();
}

bool OpaqueKllFloatSketch::is_estimation_mode() const {
    return inner_.is_estimation_mode();
}

float OpaqueKllFloatSketch::get_min_value() const {
    return inner_.get_min_value();
}

float OpaqueKllFloatSketch::get_max_value() const {
    return inner_.get_max_value();
}

float OpaqueKllFloatSketch::get_quantile(double fraction) const {
    return inner_.get_quantile(fraction);
}

std::unique_ptr<std::vector<float>> OpaqueKllFloatSketch::get_quantiles(rust::Slice<const double> fractions) const {
    auto result = inner_.get_quantiles(fractions.data(), fractions.size());
    return std::make_unique<std::vector<float>>(std::move(result));
}

std::unique_ptr<std::vector<float>> OpaqueKllFloatSketch::get_quantiles_evenly_spaced(uint32_t num) const {
    auto result = inner_.get_quantiles(num);
    return std::make_unique<std::vector<float>>(std::move(result));
}

double OpaqueKllFloatSketch::get_rank(float value) const {
    return inner_.get_rank(value);
}

std::unique_ptr<std::vector<uint8_t>> OpaqueKllFloatSketch::serialize() const {
    std::ostringstream os;
    inner_.serialize(os);
    std::string serialized = os.str();
    return std::make_unique<std::vector<uint8_t>>(serialized.begin(), serialized.end());
}

std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch() {
    return std::unique_ptr<OpaqueKllFloatSketch>(new OpaqueKllFloatSketch());
}

std::unique_ptr<OpaqueKllFloatSketch> new_opaque_kll_float_sketch_with_k(uint16_t k) {
    return std::unique_ptr<OpaqueKllFloatSketch>(new OpaqueKllFloatSketch(k));
}

std::unique_ptr<OpaqueKllFloatSketch> deserialize_opaque_kll_float_sketch(rust::Slice<const uint8_t> buf) {
    std::string str(buf.begin(), buf.end());
    std::istringstream is(str);
    return std::unique_ptr<OpaqueKllFloatSketch>(new OpaqueKllFloatSketch(is));
}

// KLL Double Sketch Implementation

OpaqueKllDoubleSketch::OpaqueKllDoubleSketch() : inner_() {}

OpaqueKllDoubleSketch::OpaqueKllDoubleSketch(uint16_t k) : inner_(k) {}

OpaqueKllDoubleSketch::OpaqueKllDoubleSketch(std::istream& is) : inner_(datasketches::kll_sketch<double>::deserialize(is)) {}

void OpaqueKllDoubleSketch::kll_double_update(double value) {
    inner_.update(value);
}

void OpaqueKllDoubleSketch::kll_double_merge(const OpaqueKllDoubleSketch& other) {
    inner_.merge(other.inner_);
}

bool OpaqueKllDoubleSketch::is_empty() const {
    return inner_.is_empty();
}

uint16_t OpaqueKllDoubleSketch::get_k() const {
    return inner_.get_k();
}

uint64_t OpaqueKllDoubleSketch::get_n() const {
    return inner_.get_n();
}

uint32_t OpaqueKllDoubleSketch::get_num_retained() const {
    return inner_.get_num_retained();
}

bool OpaqueKllDoubleSketch::is_estimation_mode() const {
    return inner_.is_estimation_mode();
}

double OpaqueKllDoubleSketch::get_min_value() const {
    return inner_.get_min_value();
}

double OpaqueKllDoubleSketch::get_max_value() const {
    return inner_.get_max_value();
}

double OpaqueKllDoubleSketch::get_quantile(double fraction) const {
    return inner_.get_quantile(fraction);
}

std::unique_ptr<std::vector<double>> OpaqueKllDoubleSketch::get_quantiles(rust::Slice<const double> fractions) const {
    auto result = inner_.get_quantiles(fractions.data(), fractions.size());
    return std::make_unique<std::vector<double>>(std::move(result));
}

std::unique_ptr<std::vector<double>> OpaqueKllDoubleSketch::get_quantiles_evenly_spaced(uint32_t num) const {
    auto result = inner_.get_quantiles(num);
    return std::make_unique<std::vector<double>>(std::move(result));
}

double OpaqueKllDoubleSketch::get_rank(double value) const {
    return inner_.get_rank(value);
}

std::unique_ptr<std::vector<uint8_t>> OpaqueKllDoubleSketch::serialize() const {
    std::ostringstream os;
    inner_.serialize(os);
    std::string serialized = os.str();
    return std::make_unique<std::vector<uint8_t>>(serialized.begin(), serialized.end());
}

std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch() {
    return std::unique_ptr<OpaqueKllDoubleSketch>(new OpaqueKllDoubleSketch());
}

std::unique_ptr<OpaqueKllDoubleSketch> new_opaque_kll_double_sketch_with_k(uint16_t k) {
    return std::unique_ptr<OpaqueKllDoubleSketch>(new OpaqueKllDoubleSketch(k));
}

std::unique_ptr<OpaqueKllDoubleSketch> deserialize_opaque_kll_double_sketch(rust::Slice<const uint8_t> buf) {
    std::string str(buf.begin(), buf.end());
    std::istringstream is(str);
    return std::unique_ptr<OpaqueKllDoubleSketch>(new OpaqueKllDoubleSketch(is));
}