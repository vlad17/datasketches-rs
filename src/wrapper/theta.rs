//! Wrapper types for the Theta sketch.

use cxx;

use crate::bridge::ffi;
use crate::DataSketchesError;

/// The [Theta][orig-docs] sketch is, essentially, an adaptive random sample
/// of a stream. As a result, it can be used to estimate distinct counts and
/// the sketches can be combined to estimate distinct counts of unions and
/// and intersections and differences of streams.
///
/// While the types of operations which theta sketches support are richer
/// than, say, the [`crate::wrapper::CpcSketch`], they have an important
/// drawback. For a sketch using `k` space, HLL or CPC sketches have variance
/// `O(1/k)` whereas Theta sketches have `O(1/sqrt(k))` for size estimates.
///
/// To recover estimates of set operations, the [`ThetaSketch`] must first
/// be converted into an immutable form, [`StaticThetaSketch`]
///
/// [orig-docs]: https://datasketches.apache.org/docs/Theta/ThetaSketchFramework.html
pub struct ThetaSketch {
    inner: cxx::UniquePtr<ffi::OpaqueThetaSketch>,
}

impl ThetaSketch {
    /// Create a Theta sketch representing the empty set.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_theta_sketch(),
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

    pub fn as_static(&self) -> StaticThetaSketch {
        StaticThetaSketch {
            inner: self.inner.as_static(),
        }
    }
}

pub struct StaticThetaSketch {
    inner: cxx::UniquePtr<ffi::OpaqueStaticThetaSketch>,
}

impl StaticThetaSketch {
    /// Return the current estimate of distinct values seen.
    pub fn estimate(&self) -> f64 {
        self.inner.estimate()
    }

    /// Return the sketch representing the set of elements present
    /// in `self` without any of the elements also present in `other`.
    pub fn set_difference(&mut self, other: &StaticThetaSketch) {
        self.inner
            .pin_mut()
            .set_difference(other.inner.as_ref().expect("non-null"));
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
            inner: ffi::deserialize_opaque_static_theta_sketch(buf)?,
        })
    }
}

impl Clone for StaticThetaSketch {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct ThetaUnion {
    inner: cxx::UniquePtr<ffi::OpaqueThetaUnion>,
}

impl ThetaUnion {
    /// Create a theta union over nothing, which corresponds to the
    /// empty set.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_theta_union(),
        }
    }

    pub fn merge(&mut self, sketch: StaticThetaSketch) {
        self.inner.pin_mut().union_with(sketch.inner)
    }

    /// Retrieve the current unioned sketch as a copy.
    pub fn sketch(&self) -> StaticThetaSketch {
        StaticThetaSketch {
            inner: self.inner.sketch(),
        }
    }
}

pub struct ThetaIntersection {
    inner: cxx::UniquePtr<ffi::OpaqueThetaIntersection>,
}

impl ThetaIntersection {
    /// Create a theta intersection.
    pub fn new() -> Self {
        Self {
            inner: ffi::new_opaque_theta_intersection(),
        }
    }

    pub fn merge(&mut self, sketch: StaticThetaSketch) {
        self.inner.pin_mut().intersect_with(sketch.inner);
    }

    /// Retrieve the current intersected sketch as a copy. Returns `None`
    /// if the sketch represents the universal set (which it does before
    /// at least one call to `merge()`.)
    pub fn sketch(&self) -> Option<StaticThetaSketch> {
        let inner = self.inner.sketch();
        let valid = !inner.is_null();
        valid.then(|| StaticThetaSketch { inner })
    }
}

#[cfg(test)]
mod tests {
    use byte_slice_cast::AsByteSlice;

    use super::*;

    fn check_cycle(s: &ThetaSketch) {
        let est = s.estimate();
        let s = s.as_static();
        let est2 = s.estimate();
        assert_eq!(est, est2);
        check_cycle_static(&s);
    }

    fn check_cycle_static(s: &StaticThetaSketch) {
        let est = s.estimate();
        let lb = est * 0.95;
        let ub = est * 1.05;

        let bytes = s.serialize();
        let cpy = StaticThetaSketch::deserialize(bytes.as_ref()).unwrap();
        let cpy2 = StaticThetaSketch::deserialize(bytes.as_ref()).unwrap();
        let cpy3 = StaticThetaSketch::deserialize(bytes.as_ref()).unwrap();
        assert_eq!(est, cpy.estimate());
        assert_eq!(est, cpy2.estimate());
        assert_eq!(est, cpy3.estimate());

        let mut union = ThetaUnion::new();
        union.merge(cpy.clone());
        union.merge(cpy2.clone());
        union.merge(cpy3.clone());
        let est2 = union.sketch().estimate();
        assert!((lb..ub).contains(&est2));

        let mut intersection = ThetaIntersection::new();
        intersection.merge(cpy);
        intersection.merge(cpy2);
        intersection.merge(cpy3);
        let est2 = intersection.sketch().expect("non-infinite").estimate();
        assert!((lb..ub).contains(&est2));
    }

    #[test]
    fn basic_count_distinct() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut theta = ThetaSketch::new();
        for _ in 0..10 {
            for key in 0u64..n {
                slice[0] = key;
                // updates should be equal
                theta.update(slice.as_byte_slice());
                theta.update_u64(key);
            }
            check_cycle(&theta);
            let est = theta.estimate();
            let lb = n as f64 * 0.95;
            let ub = n as f64 * 1.05;
            assert!((lb..ub).contains(&est));
        }
    }

    #[test]
    fn basic_intersect_overlap() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut intersection = ThetaIntersection::new();
        for _ in 0..10 {
            let mut theta = ThetaSketch::new();
            for key in 0u64..n {
                slice[0] = key;
                theta.update(slice.as_byte_slice());
                theta.update_u64(key);
            }
            intersection.merge(theta.as_static());
            let merged = intersection.sketch().expect("non-inf");
            let est = merged.estimate();
            check_cycle_static(&merged);
            let lb = n as f64 * 0.95;
            let ub = n as f64 * 1.05;
            assert!((lb..ub).contains(&est));
        }
    }

    #[test]
    fn basic_intersect() {
        let mut slice = [0u64];
        let n = 100 * 1000;
        let mut intersection = ThetaIntersection::new();
        let nrepeats = 10;
        for i in 0..nrepeats {
            let mut theta = ThetaSketch::new();
            for key in 0u64..n {
                let key = key + i * n / nrepeats;
                slice[0] = key;
                theta.update(slice.as_byte_slice());
                theta.update_u64(key);
            }
            intersection.merge(theta.as_static());
            let merged = intersection.sketch().expect("non-inf");
            let est = merged.estimate();
            check_cycle_static(&merged);
            let value = (nrepeats - i) * n / nrepeats;
            let value = value as f64;
            let lb = value * 0.95;
            let ub = value * 1.05;
            assert!(
                (lb..ub).contains(&est),
                "iteration {} value {} est {} relerr {}",
                i,
                value,
                est,
                (value - est).abs() / value.abs()
            );
        }
    }

    #[test]
    fn theta_static_deserialization_error() {
        assert!(matches!(
            StaticThetaSketch::deserialize(&[9, 9, 9, 9]),
            Err(DataSketchesError::CXXError(_))
        ));
    }
}
