use criterion::{criterion_group, criterion_main, Criterion};

extern crate libfudgec;
use libfudgec::*;

fn scan_lipsum100k(c: &mut Criterion) {
    c.bench_function("lipsum100k", |b| {
        b.iter(|| {
            let source = source::Source::from_file("testdata/lipsum100k.txt");
            scanner::tokenize(&source);
        })
    });
}

criterion_group!(benches, scan_lipsum100k);
criterion_main!(benches);
