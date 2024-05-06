//! Wrapper type for the Heavy Hitter sketch.

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;
use std::slice;

use cxx;
use thin_dst::{ThinBox, ThinRef};

use crate::bridge::ffi;

/// A type around a thin box to a byte buffer. Still basically just a pointer,
/// but lets us implement `Borrow<[u8]>` semantics for use as hash structure keys.
struct ThinByteBox(ThinBox<(), u8>);

impl Borrow<[u8]> for ThinByteBox {
    fn borrow(&self) -> &[u8] {
        &self.0.slice
    }
}

impl Hash for ThinByteBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let slice: &[u8] = self.borrow();
        slice.hash(state);
    }
}

impl PartialEq for ThinByteBox {
    fn eq(&self, other: &Self) -> bool {
        let mine: &[u8] = self.borrow();
        let yours: &[u8] = other.borrow();
        mine.eq(yours)
    }
}
impl Eq for ThinByteBox {}

/// The [Heavy Hitter][orig-docs] (HH) sketch computes an approximate set of the
/// heavy hitters, the items in a data stream which appear most often. Along with
/// each proposed approximate heavy hitter, the sketch can provide an estimate of
/// the number of its appearances.
///
/// The sketch is based on the classical [Misra-Gries sketch][mg] (MG), but, crucially, is
/// a randomized version called [Reduce by Sample Median][smed] (SMED) which only requires updates
/// which take time logarithmic in sketch size (unlike typical MG approaches, which are linear).
/// [Space Saving][ss] can be implemented with a linked-list bucketing approach that is constant time,
/// but this does not extend to weighted updates and has significant per-entry constant memory
/// overhead.
///
/// However, this comes at the cost of only a high-probability guarantee rather than absolute.
/// Luckily, this probability tends to 1 as stream size increases. See Theorem 4 from [SMED][smed].
/// This guarantee is similar to an extended form of the MG guarantee, namely that for a sketch
/// of size `k` and stream of size `n` all items occurring greater than `k/n` times will be present
/// in the sketch. The SMED guarantee, due to stochasticity, requires an additional factor of two.
/// Among such items, a subset of only definite true positives (under the
/// event that the sketch "worked" per the high-probability guarantee) or a larger set which
/// might possibly include false positives but misses no false negatives may be returned, too.
///
/// This particular heavy hitter implementation *DOES NOT* give ownership of the actual observed
/// string values to the C++ heavy hitter structure. This:
///   - simplifies memory management across language boundaries
///   - makes hashing with Rust algorithms easier
///   - avoids allocation on updates for strings already in the sketch
/// at the cost of constant additional per-string overhead (at 90% Rust HashMap load
/// factor, about ~11 bytes) and needless hashing overhead on the C++ side. While the C++ side
/// does not own the string values, when it removes a key from the hash, the key is deleted from
/// the Rust side as well.
///
/// [orig-docs]: https://datasketches.apache.org/docs/Frequency/FrequentItemsOverview.html
/// [mg]: https://en.wikipedia.org/wiki/Misra%E2%80%93Gries_summary
/// [smed]: https://arxiv.org/abs/1705.07001
/// [ss]: https://www.cse.ust.hk/~raywong/comp5331/References/EfficientComputationOfFrequentAndTop-kElementsInDataStreams.pdf
pub struct HhSketch {
    /// Note the order of members in this struct is intentional. `inner` must be dropped
    /// first, deleting [`intern`] entries.
    inner: cxx::UniquePtr<ffi::OpaqueHhSketch>,
    /// Bytestring keys are stored here; the C++ implementation refers to the byte slice
    /// _addresses_ as the unique keys in the heavy hitter sketch.
    intern: Box<HashSet<ThinByteBox>>, // boxed for stable address
    lg2_k: u8,
}

/// An entry in the heavy hitters sketch.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HhRow<'a> {
    pub key: &'a [u8],
    pub lb: u64,
    pub ub: u64,
}

/// Function safety must be justified due to lifetime construction
unsafe fn addr_to_thinref<'a>(addr: usize) -> ThinRef<'a, (), u8> {
    // not actually used as mut, which would be unsafe
    let ptr = addr as *mut _;
    let nonnull = NonNull::<_>::new(ptr).expect("non-null pointer");
    ThinRef::<'a, (), u8>::from_erased(nonnull)
}

/// Function safety must be justified due to lifetime construction
unsafe fn addr_to_hashset<'a>(addr: usize) -> &'a mut HashSet<ThinByteBox> {
    // safe as mut because we only call this in callbacks in &mut self methods
    let ptr = addr as *mut _;
    let mut nonnull = NonNull::<_>::new(ptr).expect("non-null pointer");
    nonnull.as_mut()
}

/// Function is only safe to call so long as the
/// [`HhSketch`] which created the addresses in the arguments here:
///
///   1. Has been borrowed as `Pin<&mut>` for the duration of the C++ call invoking this
///      FFI-intended function.
///   2. The corresponding addresses refer to the hashset and one of its keys from
///      the `HhSketch` in question.
pub(crate) unsafe fn remove_from_hashset(hashset_addr: usize, addr: usize) {
    // eprintln!("remove_from_hashset({},{})", hashset_addr, addr);
    let hs = addr_to_hashset(hashset_addr);
    let thinref = addr_to_thinref(addr);
    // use byte_slice_cast::AsSliceOf;
    // eprintln!("  val {}", thinref.slice.as_slice_of::<u64>().unwrap()[0]);
    let did_remove = hs.remove(&thinref.slice);
    // eprintln!("  hashset contains? {}", did_remove);
    assert!(did_remove, "thinbox {:?}", thinref);
}

impl HhSketch {
    /// Create a HH sketch representing the empty set. The sketch size `k` is set below,
    /// and together with the (runtime-determined) stream size `n` the heavy hitters
    /// which occur at least `n/k` times are to be found with high probability. Richer
    /// guarantees exist; see related work cited in the struct documentation.
    pub fn new(lg2_k: u8) -> Self {
        let intern = Box::new(HashSet::<_>::default());
        Self {
            inner: ffi::new_opaque_hh_sketch(lg2_k, intern.as_ref() as *const _ as usize),
            intern,
            lg2_k,
        }
    }

    fn thin_row_to_owned<'a>(&'a self, row: &ffi::ThinHeavyHitterRow) -> HhRow<'a> {
        let thinref = unsafe { addr_to_thinref::<'a>(row.addr) };
        let ptr = thinref.slice.as_ptr();
        HhRow {
            key: unsafe { slice::from_raw_parts(ptr, thinref.slice.len()) },
            lb: row.lb,
            ub: row.ub,
        }
    }

    /// Return the heavy hitters with no false positives, their
    /// frequency lower bound, and their frequency upper bound.
    pub fn estimate_no_fp(&self) -> Vec<HhRow> {
        self.inner
            .estimate_no_fp()
            .into_iter()
            .map(|x| self.thin_row_to_owned(x))
            .collect()
    }

    /// Return the heavy hitters with no false negatives; this is less
    /// conservative than [`Self::estimate_no_fp`].
    pub fn estimate_no_fn(&self) -> Vec<HhRow> {
        self.inner
            .estimate_no_fn()
            .into_iter()
            .map(|x| self.thin_row_to_owned(x))
            .collect()
    }

    /// Observe a new value.
    pub fn update(&mut self, value: &[u8], weight: u64) {
        // TODO: once this hash_set_entry API merges, this approach can save
        // on two (!) needless hash re-computations.
        // #![feature(hash_set_entry)]
        // let key = self.intern.get_or_insert_with::<[u8], _>(value, |buf| {
        // ThinByteBox(ThinBox::new((), buf.iter().cloned()))
        // });
        let key = if let Some(key) = self.intern.get(value) {
            &*key.0
        } else {
            let key = ThinByteBox(ThinBox::new((), value.iter().cloned()));
            self.intern.insert(key);
            &*self.intern.get(value).expect("present key").0
        };
        let thinref = ThinRef::<(), u8>::from(key);
        let key = ThinRef::<(), u8>::erase(thinref).as_ptr() as *const _ as usize;
        self.inner.pin_mut().update(key, weight)
    }

    pub fn merge(&mut self, other: &Self) {
        let state = other.inner.state();
        let total_weight = self.inner.get_total_weight() + other.inner.get_total_weight();
        for row in state.iter() {
            let row = other.thin_row_to_owned(row);
            self.update(row.key, row.lb);
        }
        let offset = self.inner.get_offset() + other.inner.get_offset();
        self.inner.pin_mut().set_weights(total_weight, offset);
    }
}

impl Clone for HhSketch {
    fn clone(&self) -> Self {
        let mut hh = Self::new(self.lg2_k);
        hh.merge(self);
        hh
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter;

    use byte_slice_cast::{AsByteSlice, AsSliceOf};

    use super::*;

    fn check_cycle(s: &HhSketch) {
        let mut est_fn = s.estimate_no_fn();
        let mut est_fp = s.estimate_no_fp();

        assert!(est_fp
            .clone()
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&est_fn.clone().into_iter().collect::<HashSet<_>>()));

        let cpy2 = s.clone();
        let cpy3 = cpy2.clone();
        let cpys = [cpy2, cpy3];

        est_fn.sort_unstable();
        est_fp.sort_unstable();

        for cpy in cpys.iter() {
            let mut cpy_fn = cpy.estimate_no_fn();
            cpy_fn.sort_unstable();
            let mut cpy_fp = cpy.estimate_no_fp();
            cpy_fp.sort_unstable();
            assert_eq!(est_fn, cpy_fn);
            assert_eq!(est_fp, cpy_fp);
        }
    }

    /// Extracts HhRows from a HhSketch filled with u64 data in tuples (key, lb, ub),
    /// pre-sorted.
    fn row2keys(hh: &HhSketch) -> Vec<(u64, u64, u64)> {
        let results = hh.estimate_no_fn();
        let mut v: Vec<_> = results
            .into_iter()
            .map(|row| {
                let key = row.key.as_slice_of::<u64>().unwrap();
                assert!(row.lb <= row.ub);
                (key[0], row.lb, row.ub)
            })
            .collect();
        v.sort_unstable();
        v
    }

    /// Makes sure that all keys in `expected` are present with the expected frequency.
    fn matches(hh: &HhSketch, expected: &[(u64, u64)]) {
        let present = row2keys(&hh)
            .into_iter()
            .map(|(key, lb, ub)| (key, (lb, ub)))
            .collect::<HashMap<_, _>>();
        for &(k, v) in expected {
            assert!(present.contains_key(&k), "key missing {}", k);
            let (lb, ub) = present[&k];
            assert!(lb <= v, "key {} lb {} incorrect (true value {})", k, lb, v);
            assert!(ub >= v, "key {} ub {} incorrect (true value {})", k, ub, v);
        }
    }

    fn matches_violations(hh: &HhSketch, expected: &[(u64, u64)]) -> usize {
        let present = row2keys(&hh)
            .into_iter()
            .map(|(key, lb, ub)| (key, (lb, ub)))
            .collect::<HashMap<_, _>>();
        let mut violations = 0;
        for &(k, v) in expected {
            if !present.contains_key(&k) {
                violations += 1;
                continue;
            }
            let (lb, ub) = present[&k];
            if lb > v || ub < v {
                violations += 1;
            }
        }
        return violations;
    }

    #[test]
    fn basic_heavy() {
        // for various sizes, ensure retains all if available, with full info
        // because of capacity constraints, give an extra factor of 2 for error
        for &lg2_k in &[3, 4, 5] {
            let mut hh = HhSketch::new(lg2_k);
            let max = 1u64 << lg2_k;
            let heavies = &[max, max + 1, max + 2];
            let iters = 3;
            for _ in 0..iters {
                for i in 0u64..max {
                    let slice = [i];
                    hh.update(slice.as_byte_slice(), 1)
                }
                for &i in heavies {
                    let slice = [i];
                    hh.update(slice.as_byte_slice(), max * 2 + 1);
                }
                for i in 0u64..max {
                    let slice = [i];
                    hh.update(slice.as_byte_slice(), 1)
                }
            }
            matches(
                &hh,
                &heavies
                    .iter()
                    .cloned()
                    .map(|k| (k, (max * 2 + 1) * iters))
                    .collect::<Vec<_>>(),
            );
            check_cycle(&hh);
        }
    }

    #[test]
    fn retains_all() {
        // for various sizes, ensure retains all if available, with full info
        // because of capacity constraints, give an extra factor of 2 for error
        for &lg2_k in &[3, 4, 5] {
            let mut hh = HhSketch::new(lg2_k + 1);
            for i in 0u64..(1 << lg2_k) {
                let slice = [i];
                hh.update(slice.as_byte_slice(), 1)
            }
            assert_eq!(
                row2keys(&hh),
                (0u64..(1 << lg2_k)).map(|v| (v, 1, 1)).collect::<Vec<_>>()
            );
            for i in 0u64..(1 << lg2_k) {
                let slice = [i];
                hh.update(slice.as_byte_slice(), 4)
            }
            assert_eq!(
                row2keys(&hh),
                (0u64..(1 << lg2_k)).map(|v| (v, 5, 5)).collect::<Vec<_>>()
            );
            check_cycle(&hh);
        }
    }

    #[test]
    fn retains_all_clone() {
        // for various sizes, ensure retains all if available, with full info
        // because of capacity constraints, give an extra factor of 2 for error
        for &lg2_k in &[3, 4, 5] {
            let mut hh = HhSketch::new(lg2_k + 1);
            for i in 0u64..(1 << lg2_k) {
                let slice = [i];
                hh.update(slice.as_byte_slice(), 1)
            }
            hh.merge(&hh.clone());
            assert_eq!(
                row2keys(&hh),
                (0u64..(1 << lg2_k)).map(|v| (v, 2, 2)).collect::<Vec<_>>()
            );
            check_cycle(&hh);
        }
    }

    #[test]
    fn basic_merge() {
        // for various sizes, ensure retains all if available, with full info
        // because of capacity constraints, give an extra factor of 2 for error
        for &lg2_k in &[3, 4, 5] {
            let mut hhs = vec![HhSketch::new(lg2_k); 3];
            let max = 1u64 << lg2_k;
            let heavies = &[max, max + 1, max + 2];
            let heavy_weight = max * 2 + 1;
            for (&heavy_key, hh) in heavies.iter().zip(hhs.iter_mut()) {
                for i in 0u64..max {
                    let slice = [i];
                    hh.update(slice.as_byte_slice(), 1)
                }
                let slice = [heavy_key];
                hh.update(slice.as_byte_slice(), heavy_weight);
                for i in 0u64..max {
                    let slice = [i];
                    hh.update(slice.as_byte_slice(), 1)
                }
                check_cycle(&hh);
            }
            let mut hh = hhs.pop().expect("some last");
            hhs.into_iter().for_each(|other| hh.merge(&other));
            matches(
                &hh,
                &heavies
                    .iter()
                    .cloned()
                    .map(|k| (k, heavy_weight))
                    .collect::<Vec<_>>(),
            );
            check_cycle(&hh);
        }
    }

    // lg2_k in 4,5
    // stream_multiplier in 2, 5, 20
    // n = stream_multiplier * k
    // need to appear at least ceil(EPSILON_FACTOR * stream_multiplier) times
    // to be present, where EPSILON_FACTOR is 3.5 from the source code of the frequent items
    // implementation (any constant >2 works).
    // with probability at least 1 - (1/n)
    //
    // nunique is the number of unique probability mass values to make, should be 1-3
    fn check_hh_property(lg2_k: u8, stream_multiplier: u8, nunique: u8) {
        use rand::prelude::*;
        let k: u64 = 1u64 << lg2_k;
        let n = k * (stream_multiplier as u64);
        let thresh = (7 * (stream_multiplier as u64) + 1) / 2;

        let mut histogram = match nunique {
            1 => {
                assert!(n / thresh > 1);
                vec![thresh; (n / thresh) as usize]
            }
            2 => {
                assert!(n / thresh / 2 > 1);
                let mut v = vec![thresh; (n / thresh / 2) as usize];
                let remain = n - (n / thresh / 2) * thresh;
                assert!(remain > 0);
                let low = thresh - 1;
                assert!(low > 0);
                v.extend(vec![low; (remain / low) as usize]);
                assert!(remain / low > 0);
                v
            }
            3 => {
                let hi = thresh + 1;
                let nhi = n / thresh / 3;
                assert!(nhi > 0);
                let mut v = vec![hi; nhi as usize];
                let remain = n - nhi * hi;
                let med = thresh;
                let nmed = remain / med / 2;
                assert!(nmed > 0);
                v.extend(vec![med; nmed as usize]);
                let remain = remain - nmed * med;
                let low = thresh - 1;
                let nlow = remain / low;
                assert!(nlow > 0);
                v.extend(vec![low; nlow as usize]);
                v
            }
            _ => panic!("invalid nunique {}", nunique),
        };

        let sum = histogram.iter().cloned().sum::<u64>();
        for _ in sum..n {
            histogram.push(1);
        }

        let mut data = histogram
            .iter()
            .cloned()
            .enumerate()
            .flat_map(|(i, repeats)| iter::repeat(i as u64).take(repeats as usize))
            .collect::<Vec<_>>();
        assert!(data.len() == n as usize);

        let expected = histogram
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, repeats)| *repeats >= thresh)
            .map(|(k, repeats)| (k as u64, repeats))
            .collect::<Vec<_>>();

        let ntrials = 25;
        let mut rng = StdRng::seed_from_u64(1234);
        let mut failures = 0;
        for _ in 0..ntrials {
            data.shuffle(&mut rng);
            let mut hh = HhSketch::new(lg2_k);
            for &i in &data {
                let slice = [i];
                hh.update(slice.as_byte_slice(), 1)
            }
            check_cycle(&hh);
            let any_invalid = row2keys(&hh)
                .into_iter()
                .any(|(k, lb, ub)| lb > histogram[k as usize] || ub < histogram[k as usize]);
            if any_invalid || matches_violations(&hh, &expected) > 0 {
                failures += 1;
            }
        }

        // Could derive a proper p-value here but I don't trust the numerics of the current
        // statrs crate (especially at this wonky setting for low 1/n and low ntrials).
        assert!(
            failures <= 1,
            "failures {} ntrials {} n {}",
            failures,
            ntrials,
            n
        );
    }

    #[test]
    fn check_hh_lgk4_multiplier2_nunique1() {
        check_hh_property(4, 2, 1);
    }

    #[test]
    fn check_hh_lgk4_multiplier2_nunique2() {
        check_hh_property(4, 2, 2);
    }

    #[test]
    fn check_hh_lgk4_multiplier2_nunique3() {
        check_hh_property(4, 2, 3);
    }

    #[test]
    fn check_hh_lgk4_multiplier5_nunique1() {
        check_hh_property(4, 5, 1);
    }

    #[test]
    fn check_hh_lgk4_multiplier5_nunique2() {
        check_hh_property(4, 5, 2);
    }

    #[test]
    fn check_hh_lgk4_multiplier5_nunique3() {
        check_hh_property(4, 5, 3);
    }

    #[test]
    fn check_hh_lgk4_multiplier20_nunique1() {
        check_hh_property(4, 20, 1);
    }

    #[test]
    fn check_hh_lgk4_multiplier20_nunique2() {
        check_hh_property(4, 20, 2);
    }

    #[test]
    fn check_hh_lgk4_multiplier20_nunique3() {
        check_hh_property(4, 20, 3);
    }

    #[test]
    fn hh_empty() {
        let hh = HhSketch::new(12);
        assert!(hh.estimate_no_fp().is_empty());
        assert!(hh.estimate_no_fn().is_empty());
        check_cycle(&hh);
    }
}
