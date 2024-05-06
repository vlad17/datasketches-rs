//! CXX bridge to datasketches-cpp library.
//!
//! See [`crate::wrapper`] for external Rust-friendly types.

use crate::wrapper::hh::remove_from_hashset;

#[cxx::bridge]
pub(crate) mod ffi {

    /// An entry to the heavy hitters sketch with keys that refer to addresses.
    struct ThinHeavyHitterRow {
        addr: usize,
        lb: u64,
        ub: u64,
    }

    extern "Rust" {
        unsafe fn remove_from_hashset(hashset_addr: usize, addr: usize);
    }

    unsafe extern "C++" {
        include!("dsrs/datasketches-cpp/cpc.hpp");

        pub(crate) type OpaqueCpcSketch;

        pub(crate) fn new_opaque_cpc_sketch() -> UniquePtr<OpaqueCpcSketch>;
        pub(crate) fn deserialize_opaque_cpc_sketch(buf: &[u8]) -> Result<UniquePtr<OpaqueCpcSketch>>;
        pub(crate) fn estimate(self: &OpaqueCpcSketch) -> f64;
        pub(crate) fn update(self: Pin<&mut OpaqueCpcSketch>, buf: &[u8]);
        pub(crate) fn update_u64(self: Pin<&mut OpaqueCpcSketch>, value: u64);
        pub(crate) fn serialize(self: &OpaqueCpcSketch) -> UniquePtr<CxxVector<u8>>;

        pub(crate) type OpaqueCpcUnion;

        pub(crate) fn new_opaque_cpc_union() -> UniquePtr<OpaqueCpcUnion>;
        pub(crate) fn sketch(self: &OpaqueCpcUnion) -> UniquePtr<OpaqueCpcSketch>;
        pub(crate) fn merge(self: Pin<&mut OpaqueCpcUnion>, to_add: UniquePtr<OpaqueCpcSketch>);

        include!("dsrs/datasketches-cpp/theta.hpp");

        pub(crate) type OpaqueThetaSketch;

        pub(crate) fn new_opaque_theta_sketch() -> UniquePtr<OpaqueThetaSketch>;
        pub(crate) fn estimate(self: &OpaqueThetaSketch) -> f64;
        pub(crate) fn update(self: Pin<&mut OpaqueThetaSketch>, buf: &[u8]);
        pub(crate) fn update_u64(self: Pin<&mut OpaqueThetaSketch>, value: u64);
        pub(crate) fn as_static(self: &OpaqueThetaSketch) -> UniquePtr<OpaqueStaticThetaSketch>;

        pub(crate) type OpaqueStaticThetaSketch;

        pub(crate) fn estimate(self: &OpaqueStaticThetaSketch) -> f64;
        pub(crate) fn clone(self: &OpaqueStaticThetaSketch) -> UniquePtr<OpaqueStaticThetaSketch>;
        pub(crate) fn set_difference(
            self: Pin<&mut OpaqueStaticThetaSketch>,
            other: &OpaqueStaticThetaSketch,
        );
        pub(crate) fn serialize(self: &OpaqueStaticThetaSketch) -> UniquePtr<CxxVector<u8>>;
        pub(crate) fn deserialize_opaque_static_theta_sketch(
            buf: &[u8],
        ) -> Result<UniquePtr<OpaqueStaticThetaSketch>>;

        pub(crate) type OpaqueThetaUnion;

        pub(crate) fn new_opaque_theta_union() -> UniquePtr<OpaqueThetaUnion>;
        pub(crate) fn sketch(self: &OpaqueThetaUnion) -> UniquePtr<OpaqueStaticThetaSketch>;
        pub(crate) fn union_with(
            self: Pin<&mut OpaqueThetaUnion>,
            to_union: UniquePtr<OpaqueStaticThetaSketch>,
        );

        pub(crate) type OpaqueThetaIntersection;

        pub(crate) fn new_opaque_theta_intersection() -> UniquePtr<OpaqueThetaIntersection>;
        pub(crate) fn sketch(self: &OpaqueThetaIntersection) -> UniquePtr<OpaqueStaticThetaSketch>;
        pub(crate) fn intersect_with(
            self: Pin<&mut OpaqueThetaIntersection>,
            to_intersect: UniquePtr<OpaqueStaticThetaSketch>,
        );

        include!("dsrs/datasketches-cpp/hh.hpp");

        pub(crate) type OpaqueHhSketch;

        pub(crate) fn new_opaque_hh_sketch(lg2_k: u8, hashset_addr: usize) -> UniquePtr<OpaqueHhSketch>;
        pub(crate) fn estimate_no_fp(
            self: &OpaqueHhSketch,
        ) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn estimate_no_fn(
            self: &OpaqueHhSketch,
        ) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn state(
            self: &OpaqueHhSketch,
        ) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn update(self: Pin<&mut OpaqueHhSketch>, value: usize, weight: u64);
        pub(crate) fn set_weights(self: Pin<&mut OpaqueHhSketch>, total_weight: u64, weight: u64);
        pub(crate) fn get_total_weight(self: &OpaqueHhSketch) -> u64;
        pub(crate) fn get_offset(self: &OpaqueHhSketch) -> u64;
    }
}
