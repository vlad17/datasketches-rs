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
        pub(crate) fn deserialize_opaque_cpc_sketch(
            buf: &[u8],
        ) -> Result<UniquePtr<OpaqueCpcSketch>>;
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

        pub(crate) fn new_opaque_hh_sketch(
            lg2_k: u8,
            hashset_addr: usize,
        ) -> UniquePtr<OpaqueHhSketch>;
        pub(crate) fn estimate_no_fp(
            self: &OpaqueHhSketch,
        ) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn estimate_no_fn(
            self: &OpaqueHhSketch,
        ) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn state(self: &OpaqueHhSketch) -> UniquePtr<CxxVector<ThinHeavyHitterRow>>;
        pub(crate) fn update(self: Pin<&mut OpaqueHhSketch>, value: usize, weight: u64);
        pub(crate) fn set_weights(self: Pin<&mut OpaqueHhSketch>, total_weight: u64, weight: u64);
        pub(crate) fn get_total_weight(self: &OpaqueHhSketch) -> u64;
        pub(crate) fn get_offset(self: &OpaqueHhSketch) -> u64;

        include!("dsrs/datasketches-cpp/kll.hpp");

        // KLL Float Sketch
        pub(crate) type OpaqueKllFloatSketch;

        pub(crate) fn new_opaque_kll_float_sketch() -> UniquePtr<OpaqueKllFloatSketch>;
        pub(crate) fn new_opaque_kll_float_sketch_with_k(k: u16) -> UniquePtr<OpaqueKllFloatSketch>;
        pub(crate) fn deserialize_opaque_kll_float_sketch(
            buf: &[u8],
        ) -> Result<UniquePtr<OpaqueKllFloatSketch>>;
        pub(crate) fn kll_float_update(self: Pin<&mut OpaqueKllFloatSketch>, value: f32);
        pub(crate) fn kll_float_merge(self: Pin<&mut OpaqueKllFloatSketch>, other: &OpaqueKllFloatSketch);
        pub(crate) fn is_empty(self: &OpaqueKllFloatSketch) -> bool;
        pub(crate) fn get_k(self: &OpaqueKllFloatSketch) -> u16;
        pub(crate) fn get_n(self: &OpaqueKllFloatSketch) -> u64;
        pub(crate) fn get_num_retained(self: &OpaqueKllFloatSketch) -> u32;
        pub(crate) fn is_estimation_mode(self: &OpaqueKllFloatSketch) -> bool;
        pub(crate) fn get_min_value(self: &OpaqueKllFloatSketch) -> f32;
        pub(crate) fn get_max_value(self: &OpaqueKllFloatSketch) -> f32;
        pub(crate) fn get_quantile(self: &OpaqueKllFloatSketch, fraction: f64) -> f32;
        pub(crate) fn get_quantiles(
            self: &OpaqueKllFloatSketch,
            fractions: &[f64],
        ) -> UniquePtr<CxxVector<f32>>;
        pub(crate) fn get_quantiles_evenly_spaced(
            self: &OpaqueKllFloatSketch,
            num: u32,
        ) -> UniquePtr<CxxVector<f32>>;
        pub(crate) fn get_rank(self: &OpaqueKllFloatSketch, value: f32) -> f64;
        pub(crate) fn serialize(self: &OpaqueKllFloatSketch) -> UniquePtr<CxxVector<u8>>;

        // KLL Double Sketch
        pub(crate) type OpaqueKllDoubleSketch;

        pub(crate) fn new_opaque_kll_double_sketch() -> UniquePtr<OpaqueKllDoubleSketch>;
        pub(crate) fn new_opaque_kll_double_sketch_with_k(k: u16) -> UniquePtr<OpaqueKllDoubleSketch>;
        pub(crate) fn deserialize_opaque_kll_double_sketch(
            buf: &[u8],
        ) -> Result<UniquePtr<OpaqueKllDoubleSketch>>;
        pub(crate) fn kll_double_update(self: Pin<&mut OpaqueKllDoubleSketch>, value: f64);
        pub(crate) fn kll_double_merge(self: Pin<&mut OpaqueKllDoubleSketch>, other: &OpaqueKllDoubleSketch);
        pub(crate) fn is_empty(self: &OpaqueKllDoubleSketch) -> bool;
        pub(crate) fn get_k(self: &OpaqueKllDoubleSketch) -> u16;
        pub(crate) fn get_n(self: &OpaqueKllDoubleSketch) -> u64;
        pub(crate) fn get_num_retained(self: &OpaqueKllDoubleSketch) -> u32;
        pub(crate) fn is_estimation_mode(self: &OpaqueKllDoubleSketch) -> bool;
        pub(crate) fn get_min_value(self: &OpaqueKllDoubleSketch) -> f64;
        pub(crate) fn get_max_value(self: &OpaqueKllDoubleSketch) -> f64;
        pub(crate) fn get_quantile(self: &OpaqueKllDoubleSketch, fraction: f64) -> f64;
        pub(crate) fn get_quantiles(
            self: &OpaqueKllDoubleSketch,
            fractions: &[f64],
        ) -> UniquePtr<CxxVector<f64>>;
        pub(crate) fn get_quantiles_evenly_spaced(
            self: &OpaqueKllDoubleSketch,
            num: u32,
        ) -> UniquePtr<CxxVector<f64>>;
        pub(crate) fn get_rank(self: &OpaqueKllDoubleSketch, value: f64) -> f64;
        pub(crate) fn serialize(self: &OpaqueKllDoubleSketch) -> UniquePtr<CxxVector<u8>>;
    }
}
