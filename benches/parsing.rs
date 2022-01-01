#[cfg(feature = "mock")]
use criterion::{criterion_group, criterion_main, Criterion};
use powercap::PowerCap;
use std::convert::TryFrom;

pub fn criterion_benchmark(c: &mut Criterion) {
    let root = temp_dir::TempDir::new().unwrap();
    powercap::mock::MockBuilder::default()
        .build(root.path())
        .unwrap();
    c.bench_function("success", |b| {
        b.iter(|| PowerCap::try_from(root.path()).is_ok())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
