//! Wrapper types for the KLL sketch.

use cxx;
use serde::{Serialize, Deserialize};

use crate::bridge::ffi;
use crate::DataSketchesError;

/// The [KLL quantile sketch][orig-docs] is a very compact quantiles sketch 
/// with lazy compaction scheme and nearly optimal accuracy per retained item.
/// 
/// This sketch enables near-real time analysis of the approximate distribution 
/// of values from a very large stream in a single pass, requiring only that 
/// the values are comparable.
///
/// The analysis is obtained using `get_quantile()` or `get_quantiles()` functions 
/// or the inverse functions `get_rank()`.
///
/// [orig-docs]: https://datasketches.apache.org/docs/KLL/KLLSketch.html
pub struct KllFloatSketch {
    inner: cxx::UniquePtr<ffi::OpaqueKllFloatSketch>,
}

impl KllFloatSketch {
    /// Create a KLL sketch with default parameter k=200.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_kll_float_sketch(),
        }
    }

    /// Create a KLL sketch with specified parameter k.
    /// Parameter k controls the size and accuracy of the sketch.
    pub fn with_k(k: u16) -> Self {
        Self {
            inner: ffi::new_opaque_kll_float_sketch_with_k(k),
        }
    }

    /// Updates this sketch with the given value.
    pub fn update(&mut self, value: f32) {
        self.inner.pin_mut().kll_float_update(value);
    }

    /// Merges another sketch into this one.
    pub fn merge(&mut self, other: &KllFloatSketch) {
        self.inner.pin_mut().kll_float_merge(&other.inner);
    }

    /// Returns true if this sketch is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns configured parameter k.
    pub fn get_k(&self) -> u16 {
        self.inner.get_k()
    }

    /// Returns the length of the input stream.
    pub fn get_n(&self) -> u64 {
        self.inner.get_n()
    }

    /// Returns the number of retained items (samples) in the sketch.
    pub fn get_num_retained(&self) -> u32 {
        self.inner.get_num_retained()
    }

    /// Returns true if this sketch is in estimation mode.
    pub fn is_estimation_mode(&self) -> bool {
        self.inner.is_estimation_mode()
    }

    /// Returns the min value of the stream.
    /// If the sketch is empty this returns NaN.
    pub fn get_min_value(&self) -> f32 {
        self.inner.get_min_value()
    }

    /// Returns the max value of the stream.
    /// If the sketch is empty this returns NaN.
    pub fn get_max_value(&self) -> f32 {
        self.inner.get_max_value()
    }

    /// Returns an approximation to the value at the given fractional position.
    /// 
    /// # Arguments
    /// * `fraction` - the fractional position in the hypothetical sorted stream (0.0 to 1.0)
    pub fn get_quantile(&self, fraction: f64) -> f32 {
        self.inner.get_quantile(fraction)
    }

    /// Returns approximations to the given fractional positions.
    /// This is more efficient than multiple calls to `get_quantile()`.
    /// 
    /// # Arguments
    /// * `fractions` - slice of fractional positions in the hypothetical sorted stream (0.0 to 1.0)
    pub fn get_quantiles(&self, fractions: &[f64]) -> Vec<f32> {
        let result = self.inner.get_quantiles(fractions);
        result.as_slice().to_vec()
    }

    /// Returns approximations to evenly-spaced fractional positions.
    /// 
    /// # Arguments
    /// * `num` - number of evenly-spaced fractional ranks to return
    pub fn get_quantiles_evenly_spaced(&self, num: u32) -> Vec<f32> {
        let result = self.inner.get_quantiles_evenly_spaced(num);
        result.as_slice().to_vec()
    }

    /// Returns the rank (fractional position) of the given value.
    /// 
    /// # Arguments
    /// * `value` - the value to find the rank for
    pub fn get_rank(&self, value: f32) -> f64 {
        self.inner.get_rank(value)
    }

    /// Serialize this sketch to bytes.
    pub fn serialize(&self) -> impl AsRef<[u8]> {
        struct UPtrVec(cxx::UniquePtr<cxx::CxxVector<u8>>);
        impl AsRef<[u8]> for UPtrVec {
            fn as_ref(&self) -> &[u8] {
                self.0.as_slice()
            }
        }
        UPtrVec(self.inner.serialize())
    }

    /// Deserialize a sketch from bytes.
    pub fn deserialize(buf: &[u8]) -> Result<Self, DataSketchesError> {
        Ok(Self {
            inner: ffi::deserialize_opaque_kll_float_sketch(buf)?,
        })
    }

    /// Serialize this sketch to MessagePack format.
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let sketch_data = SketchData::from_sketch(self);
        rmp_serde::to_vec(&sketch_data)
    }

    /// Deserialize a sketch from MessagePack format.
    pub fn from_msgpack(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let sketch_data: SketchData = rmp_serde::from_slice(buf)?;
        let sketch = Self::deserialize(&sketch_data.serialized_sketch)?;
        Ok(sketch)
    }
}

impl Default for KllFloatSketch {
    fn default() -> Self {
        Self::new()
    }
}

/// The [KLL quantile sketch][orig-docs] for double precision values.
/// 
/// This sketch enables near-real time analysis of the approximate distribution 
/// of values from a very large stream in a single pass, requiring only that 
/// the values are comparable.
///
/// [orig-docs]: https://datasketches.apache.org/docs/KLL/KLLSketch.html
pub struct KllDoubleSketch {
    inner: cxx::UniquePtr<ffi::OpaqueKllDoubleSketch>,
}

impl KllDoubleSketch {
    /// Create a KLL sketch with default parameter k=200.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_kll_double_sketch(),
        }
    }

    /// Create a KLL sketch with specified parameter k.
    /// Parameter k controls the size and accuracy of the sketch.
    pub fn with_k(k: u16) -> Self {
        Self {
            inner: ffi::new_opaque_kll_double_sketch_with_k(k),
        }
    }

    /// Updates this sketch with the given value.
    pub fn update(&mut self, value: f64) {
        self.inner.pin_mut().kll_double_update(value);
    }

    /// Merges another sketch into this one.
    pub fn merge(&mut self, other: &KllDoubleSketch) {
        self.inner.pin_mut().kll_double_merge(&other.inner);
    }

    /// Returns true if this sketch is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns configured parameter k.
    pub fn get_k(&self) -> u16 {
        self.inner.get_k()
    }

    /// Returns the length of the input stream.
    pub fn get_n(&self) -> u64 {
        self.inner.get_n()
    }

    /// Returns the number of retained items (samples) in the sketch.
    pub fn get_num_retained(&self) -> u32 {
        self.inner.get_num_retained()
    }

    /// Returns true if this sketch is in estimation mode.
    pub fn is_estimation_mode(&self) -> bool {
        self.inner.is_estimation_mode()
    }

    /// Returns the min value of the stream.
    /// If the sketch is empty this returns NaN.
    pub fn get_min_value(&self) -> f64 {
        self.inner.get_min_value()
    }

    /// Returns the max value of the stream.
    /// If the sketch is empty this returns NaN.
    pub fn get_max_value(&self) -> f64 {
        self.inner.get_max_value()
    }

    /// Returns an approximation to the value at the given fractional position.
    /// 
    /// # Arguments
    /// * `fraction` - the fractional position in the hypothetical sorted stream (0.0 to 1.0)
    pub fn get_quantile(&self, fraction: f64) -> f64 {
        self.inner.get_quantile(fraction)
    }

    /// Returns approximations to the given fractional positions.
    /// This is more efficient than multiple calls to `get_quantile()`.
    /// 
    /// # Arguments
    /// * `fractions` - slice of fractional positions in the hypothetical sorted stream (0.0 to 1.0)
    pub fn get_quantiles(&self, fractions: &[f64]) -> Vec<f64> {
        let result = self.inner.get_quantiles(fractions);
        result.as_slice().to_vec()
    }

    /// Returns approximations to evenly-spaced fractional positions.
    /// 
    /// # Arguments
    /// * `num` - number of evenly-spaced fractional ranks to return
    pub fn get_quantiles_evenly_spaced(&self, num: u32) -> Vec<f64> {
        let result = self.inner.get_quantiles_evenly_spaced(num);
        result.as_slice().to_vec()
    }

    /// Returns the rank (fractional position) of the given value.
    /// 
    /// # Arguments
    /// * `value` - the value to find the rank for
    pub fn get_rank(&self, value: f64) -> f64 {
        self.inner.get_rank(value)
    }

    /// Serialize this sketch to bytes.
    pub fn serialize(&self) -> impl AsRef<[u8]> {
        struct UPtrVec(cxx::UniquePtr<cxx::CxxVector<u8>>);
        impl AsRef<[u8]> for UPtrVec {
            fn as_ref(&self) -> &[u8] {
                self.0.as_slice()
            }
        }
        UPtrVec(self.inner.serialize())
    }

    /// Deserialize a sketch from bytes.
    pub fn deserialize(buf: &[u8]) -> Result<Self, DataSketchesError> {
        Ok(Self {
            inner: ffi::deserialize_opaque_kll_double_sketch(buf)?,
        })
    }

    /// Serialize this sketch to MessagePack format.
    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let sketch_data = SketchData::from_sketch_double(self);
        rmp_serde::to_vec(&sketch_data)
    }

    /// Deserialize a sketch from MessagePack format.
    pub fn from_msgpack(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let sketch_data: SketchData = rmp_serde::from_slice(buf)?;
        let sketch = Self::deserialize(&sketch_data.serialized_sketch)?;
        Ok(sketch)
    }
}

impl Default for KllDoubleSketch {
    fn default() -> Self {
        Self::new()
    }
}

/// MessagePack-serializable structure for cross-language compatibility.
/// This includes both the serialized sketch data and metadata for validation.
#[derive(Serialize, Deserialize, Debug)]
pub struct SketchData {
    /// The type of sketch (for validation)
    pub sketch_type: String,
    /// Version info for compatibility checking
    pub version: String,
    /// Configuration parameter k
    pub k: u16,
    /// Number of items processed
    pub n: u64,
    /// The actual serialized sketch bytes from DataSketches
    pub serialized_sketch: Vec<u8>,
    /// Optional metadata for additional compatibility info
    pub metadata: std::collections::HashMap<String, String>,
}

impl SketchData {
    /// Create SketchData from a KLL float sketch
    pub fn from_sketch(sketch: &KllFloatSketch) -> Self {
        let serialized = sketch.serialize();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("sketch_class".to_string(), "KllFloatSketch".to_string());
        metadata.insert("value_type".to_string(), "f32".to_string());
        
        Self {
            sketch_type: "kll".to_string(),
            version: "1.0".to_string(),
            k: sketch.get_k(),
            n: sketch.get_n(),
            serialized_sketch: serialized.as_ref().to_vec(),
            metadata,
        }
    }

    /// Create SketchData from a KLL double sketch
    pub fn from_sketch_double(sketch: &KllDoubleSketch) -> Self {
        let serialized = sketch.serialize();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("sketch_class".to_string(), "KllDoubleSketch".to_string());
        metadata.insert("value_type".to_string(), "f64".to_string());
        
        Self {
            sketch_type: "kll".to_string(),
            version: "1.0".to_string(),
            k: sketch.get_k(),
            n: sketch.get_n(),
            serialized_sketch: serialized.as_ref().to_vec(),
            metadata,
        }
    }
}

