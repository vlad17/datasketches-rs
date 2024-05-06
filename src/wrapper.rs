//! Wrapper types for bound CXX opaque types.
//!
//! The only required overhead from the C++ code is a pointer indirection,
//! to an opaque C++ type (and the corresponding heap allocation) and
//! lack of inlining, though this may be improved with cross-language
//! LTO, see dtolnay/cxx#371.

mod cpc;
pub(crate) mod hh;
mod hll;
mod theta;

pub use cpc::{CpcSketch, CpcUnion};
pub use hh::HhSketch;
pub use hll::{HLLSketch, HLLType, HLLUnion};
pub use theta::{StaticThetaSketch, ThetaIntersection, ThetaSketch, ThetaUnion};
