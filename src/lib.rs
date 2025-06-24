//! `dsrs` contains bindings for a subset of [Apache DataSketches](https://github.com/apache/datasketches-cpp).

mod bridge;
pub mod counters;
mod error;
pub mod stream_reducer;
mod wrapper;

pub use error::DataSketchesError;
pub use wrapper::CpcSketch;
pub use wrapper::CpcUnion;
pub use wrapper::HhSketch;
pub use wrapper::KllFloatSketch;
pub use wrapper::KllDoubleSketch;
pub use wrapper::StaticThetaSketch;
pub use wrapper::ThetaIntersection;
pub use wrapper::ThetaSketch;
pub use wrapper::ThetaUnion;
