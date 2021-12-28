use criterion::{criterion_group, criterion_main, Criterion};
use powercap::PowerCap;
use std::convert::TryFrom;
use std::path::PathBuf;

pub fn criterion_benchmark(c: &mut Criterion) {
    let root = PathBuf::from(".").join("assets").join("success");
    c.bench_function("success", |b| {
        b.iter(|| PowerCap::try_from(root.clone()).is_ok())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
