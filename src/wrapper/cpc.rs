//! Wrapper types for the CPC sketch.

use cxx;

use crate::bridge::ffi;
use crate::DataSketchesError;

/// The [Compressed Probability Counting][orig-docs] (CPC) sketch is
/// a dynamically resizing (but still bounded-size) distinct count sketch.
/// Some differences between CPC and the more typical [HLL++][hll-wiki] are:
///
///  * CPC theoretically uses less space than HLL (HLL++ does not
///    asymptotically improve over HLL).
///  * CPC allocates, whereas certain HLL implementations may not.
///
/// To give you a sense of CPC performance, the [linked benchmarks][benches] show
/// CPC requiring only 2.5KB of space for recovering a billion distinct values
/// well within 5% relative error even in the 99-th percentile.
///
/// This sketch supports merging through an intermediate type, [`CpcUnion`].
///
/// [orig-docs]: https://datasketches.apache.org/docs/CPC/CPC.html
/// [hll-wiki]: https://en.wikipedia.org/wiki/HyperLogLog
/// [benches]: https://datasketches.apache.org/docs/CPC/CpcPerformance.html
pub struct CpcSketch {
    inner: cxx::UniquePtr<ffi::OpaqueCpcSketch>,
}

impl CpcSketch {
    /// Create a CPC sketch representing the empty set.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_cpc_sketch(),
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

    pub fn deserialize(buf: &[u8]) -> Result<Self, DataSketchesError> {
        Ok(Self {
            inner: ffi::deserialize_opaque_cpc_sketch(buf)?,
        })
    }
}

pub struct CpcUnion {
    inner: cxx::UniquePtr<ffi::OpaqueCpcUnion>,
}

impl CpcUnion {
    /// Create a CPC union over nothing, which corresponds to the
    /// empty set.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_cpc_union(),
        }
    }

    pub fn merge(&mut self, sketch: CpcSketch) {
        self.inner.pin_mut().merge(sketch.inner)
    }

    /// Retrieve the current unioned sketch as a copy.
    pub fn sketch(&self) -> CpcSketch {
        CpcSketch {
            inner: self.inner.sketch(),
        }
    }
}

#[cfg(test)]
mod tests {
    use byte_slice_cast::AsByteSlice;

    use super::*;

    fn check_cycle(s: &CpcSketch) {
        let est = s.estimate();
        let bytes = s.serialize();
        let cpy = CpcSketch::deserialize(bytes.as_ref()).unwrap();
        let cpy2 = CpcSketch::deserialize(bytes.as_ref()).unwrap();
        let cpy3 = CpcSketch::deserialize(bytes.as_ref()).unwrap();
        assert_eq!(est, cpy.estimate());
        assert_eq!(est, cpy2.estimate());
        assert_eq!(est, cpy3.estimate());
    }

    #[test]
    fn basic_count_distinct() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut cpc = CpcSketch::new();
        for _ in 0..10 {
            for key in 0u64..n {
                slice[0] = key;
                // updates should be equal
                cpc.update(slice.as_byte_slice());
                cpc.update_u64(key);
            }
            check_cycle(&cpc);
            let est = cpc.estimate();
            let lb = n as f64 * 0.95;
            let ub = n as f64 * 1.05;
            assert!((lb..ub).contains(&est));
        }
    }

    #[test]
    fn cpc_empty() {
        let cpc = CpcSketch::new();
        assert_eq!(cpc.estimate(), 0.0);
        check_cycle(&cpc);
    }

    #[test]
    fn union_empty() {
        let cpc = CpcUnion::new().sketch();
        assert_eq!(cpc.estimate(), 0.0);
        let mut union = CpcUnion::new();
        union.merge(cpc);
        union.merge(CpcSketch::new());
        let cpc = union.sketch();
        assert_eq!(cpc.estimate(), 0.0);
    }

    #[test]
    fn basic_union_overlap() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut union = CpcUnion::new();
        for _ in 0..10 {
            let mut cpc = CpcSketch::new();
            for key in 0u64..n {
                slice[0] = key;
                cpc.update(slice.as_byte_slice());
                cpc.update_u64(key);
            }
            union.merge(cpc);
            let merged = union.sketch();
            let est = merged.estimate();
            check_cycle(&merged);
            let lb = n as f64 * 0.95;
            let ub = n as f64 * 1.05;
            assert!((lb..ub).contains(&est));
        }
    }

    #[test]
    fn basic_union_distinct() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut union = CpcUnion::new();
        let nrepeats = 6;
        for i in 0..10 {
            let mut cpc = CpcSketch::new();
            for key in 0u64..n {
                slice[0] = key + (i % nrepeats) * n;
                cpc.update(slice.as_byte_slice());
                cpc.update_u64(key);
            }
            union.merge(cpc);
            let merged = union.sketch();
            let est = merged.estimate();
            check_cycle(&merged);
            let lb = (n * nrepeats.min(i + 1)) as f64 * 0.95;
            let ub = (n * nrepeats.min(i + 1)) as f64 * 1.05;
            assert!((lb..ub).contains(&est));
        }
    }

    #[test]
    fn cpc_deserialization_error() {
        assert!(matches!(
            CpcSketch::deserialize(&[9, 9, 9, 9]),
            Err(DataSketchesError::CXXError(_))
        ));
    }
}
