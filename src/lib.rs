//! `dsrs` contains bindings for a subset of [Apache DataSketches](https://github.com/apache/datasketches-cpp).

mod error;
mod bridge;
pub mod counters;
pub mod stream_reducer;
mod wrapper;

pub use wrapper::CpcSketch;
pub use wrapper::CpcUnion;
pub use wrapper::HhSketch;
pub use wrapper::StaticThetaSketch;
pub use wrapper::ThetaIntersection;
pub use wrapper::ThetaSketch;
pub use wrapper::ThetaUnion;
pub use error::DataSketchesError;
