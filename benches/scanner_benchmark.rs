use criterion::{criterion_group, criterion_main, Criterion};

extern crate libfudgec;
use libfudgec::*;
use scanner::Scanner;

fn scan_lipsum100k(c: &mut Criterion) {
    let source = source::FileSource::from_filepath("testdata/lipsum100k.txt");
    let mut tokens = Vec::new();
    tokens.reserve(100000);

    c.bench_function("lipsum100k", |b| {
        b.iter(|| {
            tokens.clear();
            // Scan all tokens
            let mut scanner = scanner::ScannerImpl::new(&source);
            while let Some(n) = scanner.read_token() {
                tokens.push(n);
            }
        })
    });
}

criterion_group!(benches, scan_lipsum100k);
criterion_main!(benches);
