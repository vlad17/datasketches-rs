use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::Mutex;

use criterion::{
    criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup, BenchmarkId,
    Criterion, SamplingMode,
};
use hyperloglogplus::HyperLogLog;
use ordered_float::NotNan;

use dsrs::CpcSketch;

struct TrialTracker {
    tracker: Mutex<HashMap<u64, HashMap<String, f64>>>,
}

impl TrialTracker {
    fn observe(&self, name: &str, expected: u64, actual: f64) {
        let mut tracker = self.tracker.lock().unwrap();
        let tracker = tracker.entry(expected).or_insert_with(HashMap::new);
        let entry = tracker.entry(name.to_owned()).or_insert(0.0);
        let relerr = (expected as f64 - actual) / (expected as f64);
        *entry = entry.max(relerr.abs());
    }

    fn to_map(self) -> HashMap<u64, HashMap<String, f64>> {
        self.tracker.into_inner().unwrap()
    }
}

fn bench_input<T, I, F, E, M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    sz: u64,
    name: &str,
    init: I,
    update: F,
    estimate: E,
    tracker: &TrialTracker,
) where
    I: Fn() -> T,
    F: Fn(&mut T, u64),
    E: Fn(&T) -> f64,
{
    group.bench_with_input(BenchmarkId::new(name, sz), &sz, |b, i| {
        b.iter(|| {
            let mut sketch = init();
            for _ in 0..10 {
                for key in 0..(*i) {
                    update(&mut sketch, key);
                }
            }
            let e = estimate(&sketch);
            tracker.observe(name, sz, e);
        })
    });
}

fn bench_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeat-ten");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    let million = 1000 * 1000;
    let tracker = TrialTracker {
        tracker: Mutex::new(HashMap::new()),
    };
    for i in [million].iter().copied() {
        bench_input(
            &mut group,
            i,
            "dsrs::CpcSketch",
            CpcSketch::new,
            |sketch, key| sketch.update_u64(key),
            CpcSketch::estimate,
            &tracker,
        );
        // the default lg2_k param of CPC corresponds to the accuracy
        // of the corresponding bin count param in HLL to be set to lg2_k+1
        // (see https://arxiv.org/abs/1708.06839). This parameter was
        // found by inspecting the source of the given package as that which
        // gives rise to 12 bins, correspnoding to CPC's lg2_k default of 11 bins
        bench_input(
            &mut group,
            i,
            "amadeus_streaming::HyperLogLog",
            || amadeus_streaming::HyperLogLog::new(0.174),
            |sketch, key| sketch.push(&key),
            amadeus_streaming::HyperLogLog::<u64>::len,
            &tracker,
        );
        bench_input(
            &mut group,
            i,
            "probabilistic_collections::HyperLogLog",
            || probabilistic_collections::hyperloglog::HyperLogLog::new(0.174),
            |sketch, key| sketch.insert(&key),
            probabilistic_collections::hyperloglog::HyperLogLog::<u64>::len,
            &tracker,
        );
        bench_input(
            &mut group,
            i,
            "probably::HyperLogLog",
            || probably::frequency::hll::HyperLogLog::new(0.174),
            |sketch, key| sketch.insert(&key),
            probably::frequency::hll::HyperLogLog::len,
            &tracker,
        );
        bench_input(
            &mut group,
            i,
            "hyperloglogplus::HyperLogLogPlus",
            || hyperloglogplus::HyperLogLogPlus::<u64, _>::new(12, RandomState::new()).unwrap(),
            |sketch, key| sketch.insert(&key),
            |sketch| sketch.clone().count(),
            &tracker,
        );
    }
    group.finish();

    eprintln!("relative errors");
    for (sz, map) in tracker.to_map().into_iter() {
        eprintln!("size: {}", sz);
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by_key(|(name, relerr)| (NotNan::new(*relerr).ok(), name.clone()));
        for (name, relerr) in v.into_iter() {
            eprintln!("  relerr: {:5.1}% name: {}", relerr * 100.0, name);
        }
    }
}

criterion_group!(benches, bench_speed);
criterion_main!(benches);
