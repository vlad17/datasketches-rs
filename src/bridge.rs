//! CXX bridge to datasketches-cpp library.
//!
//! See [`crate::wrapper`] for external Rust-friendly types.

#[cxx::bridge]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("dsrs/datasketches-cpp/bridge.hpp");

        pub(crate) type OpaqueCpcSketch;

        pub(crate) fn new_opaque_cpc_sketch() -> UniquePtr<OpaqueCpcSketch>;
        pub(crate) fn deserialize_opaque_cpc_sketch(buf: &[u8]) -> UniquePtr<OpaqueCpcSketch>;
        pub(crate) fn estimate(self: &OpaqueCpcSketch) -> f64;
        pub(crate) fn update(self: Pin<&mut OpaqueCpcSketch>, buf: &[u8]);
        pub(crate) fn update_u64(self: Pin<&mut OpaqueCpcSketch>, value: u64);
        pub(crate) fn serialize(self: &OpaqueCpcSketch) -> UniquePtr<CxxVector<u8>>;

        pub(crate) type OpaqueCpcUnion;

        pub(crate) fn new_opaque_cpc_union() -> UniquePtr<OpaqueCpcUnion>;
        pub(crate) fn sketch(self: &OpaqueCpcUnion) -> UniquePtr<OpaqueCpcSketch>;
        pub(crate) fn merge(self: Pin<&mut OpaqueCpcUnion>, to_add: UniquePtr<OpaqueCpcSketch>);
    }
}
