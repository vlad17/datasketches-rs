//! Wrapper type for the Heavy Hitter sketch.

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;
use std::slice;

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

    pub fn serialize(&self) -> impl AsRef<[u8]> {
        struct UPtrVec(cxx::UniquePtr<cxx::CxxVector<u8>>);
        impl AsRef<[u8]> for UPtrVec {
            fn as_ref(&self) -> &[u8] {
                self.0.as_slice()
            }
        }
        UPtrVec(self.inner.serialize())
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
    use bstr::ByteSlice;
    use std::collections::HashMap;
    use std::iter;

    use crate::CpcSketch;
    use byte_slice_cast::{AsByteSlice, AsSliceOf};

    use super::*;

    fn check_cycle(s: &HLLSketch) {
        let est = s.estimate();
        let bytes = s.serialize();
        let cpy = HLLSketch::deserialize(bytes.as_ref());
        let cpy2 = HLLSketch::deserialize(bytes.as_ref());
        let cpy3 = HLLSketch::deserialize(bytes.as_ref());
        assert_eq!(est, cpy.estimate());
        assert_eq!(est, cpy2.estimate());
        assert_eq!(est, cpy3.estimate());
    }

    #[test]
    fn hll_empty() {
        let cpc = HLLSketch::new(12);
        assert_eq!(cpc.estimate(), 0.0);
        check_cycle(&cpc);
    }

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
        let bytes = base64::decode_config(
            "AgEHDAMABAgr8vsGdYFmB4Yv+Q2BvF0GAAAAAAAAAAAAAAAAAAAAAA==",
            base64::STANDARD_NO_PAD,
        )
        .unwrap();
        let hh = HLLSketch::deserialize(&bytes);

        assert_eq!(hh.estimate(), 4.000000029802323);
    }

    #[test]
    fn hll_merge_sketches() {
        let bytes = base64::decode_config(
            "AgEHDAMABAgr8vsGdYFmB4Yv+Q2BvF0GAAAAAAAAAAAAAAAAAAAAAA==",
            base64::STANDARD_NO_PAD,
        )
        .unwrap();
        let hh1 = HLLSketch::deserialize(&bytes);

        let bytes = base64::decode_config(
            "AgEHDAMABAgGc2UEe2XmCNsXmgrDsDgEAAAAAAAAAAAAAAAAAAAAAA==",
            base64::STANDARD_NO_PAD,
        )
        .unwrap();
        let hh2 = HLLSketch::deserialize(&bytes);

        assert_eq!(hh1.estimate(), 4.000000029802323);
        assert_eq!(hh2.estimate(), 4.000000029802323);
    }
}
