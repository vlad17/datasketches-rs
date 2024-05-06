//! Wrapper type for the Heavy Hitter sketch.

use std::ptr::NonNull;
use std::slice;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

use cxx;

use crate::bridge::ffi;

pub struct HLLSketch {
    inner: cxx::UniquePtr<ffi::OpaqueHLLSketch>,
}

impl HLLSketch {
    /// Create a HH sketch representing the empty set.
    pub fn new(lg2_k: u32) -> Self {
        Self {
            inner: ffi::new_opaque_hll_sketch(lg2_k),
        }
    }

    /// Return the current estimate of distinct values seen.
    pub fn estimate(&self) -> f64 {
        self.inner.estimate()
    }

    /// Observe a new value. Two values must have the exact same
    /// bytes and lengths to be considered equal.
    pub fn update(&mut self, value: &[u8]) {
        self.inner.pin_mut().update(value)
    }

    /// Observe a new `u64`. If the native-endian byte ordered bytes
    /// are equal to any other value seen by `update()`, this will be considered
    /// equal. If you are intending to use serialized sketches across
    /// platforms with different endianness, make sure to convert this
    /// `value` to network order first.
    pub fn update_u64(&mut self, value: u64) {
        self.inner.pin_mut().update_u64(value)
    }

    pub fn deserialize(buf: &[u8]) -> Self {
        // TODO: this could be friendlier, it currently terminates
        // the program no bad deserialization, and instead can be a
        // Result.
        Self {
            inner: ffi::deserialize_opaque_hll_sketch(buf),
        }
    }
}

// impl Clone for HLLSketch {
//     fn clone(&self) -> Self {
//         let mut hh = Self::new(self.lg2_k);
//         hh.merge(self);
//         hh
//     }
// }

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter;
    use bstr::ByteSlice;

    use byte_slice_cast::{AsByteSlice, AsSliceOf};

    use super::*;

    #[test]
    fn hll_simple_test() {
        let mut hh = HLLSketch::new(12);
        assert_eq!(hh.estimate(), 0.0);

        hh.update_u64(1);
        hh.update_u64(2);
        hh.update_u64(3);
        hh.update_u64(4);
        hh.update_u64(5);

        assert_eq!(hh.estimate(), 5.000000049670538);

        println!("{:?}", hh.estimate());
    }

    #[test]
    fn hll_deserialize_databricks() {
        let bytes = base64::decode_config("AgEHDAMABAgr8vsGdYFmB4Yv+Q2BvF0GAAAAAAAAAAAAAAAAAAAAAA==", base64::STANDARD_NO_PAD).unwrap();
        let hh = HLLSketch::deserialize(&bytes);

        assert_eq!(hh.estimate(), 4.000000029802323);
    }
}
