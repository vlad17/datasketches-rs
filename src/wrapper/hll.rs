//! Wrapper type for the Heavy Hitter sketch.

use std::ptr::NonNull;
use std::slice;
use std::borrow::Borrow;
use std::collections::HashSet;
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

    use byte_slice_cast::{AsByteSlice, AsSliceOf};

    use super::*;

    #[test]
    fn hll_empty() {
        let hh = HLLSketch::new(12);
        println!("{:?}", hh.estimate());
    }
}
