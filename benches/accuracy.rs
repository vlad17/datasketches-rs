/*use criterion::{
    criterion_group, criterion_main, measurement::Measurement, measurement::ValueFormatter,
    BenchmarkId, Criterion, SamplingMode, Throughput,
};
use dsrs::CpcSketch;

use std::time::Duration;

struct RelativeError;
struct RelativeErrorFormatter;

impl Measurement for RelativeError {
    type Intermediate = ();
    type Value = f64;

    fn start(&self) -> Self::Intermediate {
        ()
    }
    fn end(&self, _: Self::Intermediate) -> Self::Value {
        0.0
    }
    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }
    fn zero(&self) -> Self::Value {
        0.0
    }
    fn to_f64(&self, val: &Self::Value) -> f64 {
        *val
    }
    fn formatter(&self) -> &dyn ValueFormatter {
        &RelativeErrorFormatter
    }
}

impl ValueFormatter for RelativeErrorFormatter {
    fn scale_throughputs(&self, _: f64, _: &Throughput, _: &mut [f64]) -> &'static str {
        ""
    }

    fn scale_values(&self, re: f64, values: &mut [f64]) -> &'static str {
        for v in values {
            *v *= 100.0
        }
        "%"
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        // no scaling is needed
        ""
    }
}

fn bench_acc(c: &mut Criterion<RelativeError>) {
    let mut group = c.benchmark_group("accuracy");
    let million = 1000 * 1000;
    let billion = 1000 * million;
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.nresamples(2);
    group.warm_up_time(Duration::from_nanos(1));
    for i in [million, billion].iter() {
        group.bench_with_input(BenchmarkId::new("dsrs::CpcSketch", i), i, |b, i| {
            b.iter_custom(|x| {
                let mut cpc = CpcSketch::new();
                for _ in 0..2 {
                    for key in 0..(*i) {
                        cpc.update_u64(key)
                    }
                }
                let actual = *i as f64;
                dbg!(i, x);
                (cpc.estimate() - actual).abs() / actual
            })
        });
        /* group.bench_with_input(BenchmarkId::new("Iterative", i), i, |b, i| {
            b.iter(|| fibonacci_fast(*i))
        });*/
    }
    group.finish();
}

fn relative_error_measurement() -> Criterion<RelativeError> {
    Criterion::default().with_measurement(RelativeError)
}

criterion_group! {
    name = benches;
    config = relative_error_measurement();
    targets = bench_acc
}
criterion_main!(benches);
*/
