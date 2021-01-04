use criterion::{criterion_group, criterion_main, Criterion};

extern crate libfudgec;
use libfudgec::*;

fn scan_lipsum100k(c: &mut Criterion) {
    let source = source::FileSource::new("testdata/lipsum100k.txt");
    let mut tokens = Vec::new();
    tokens.reserve(100000);

    c.bench_function("lipsum100k", |b| b.iter(|| {
        tokens.clear();
        // Scan all tokens
        let mut scanner = scanner::Scanner::new(&source);
        while let Some(n) = scanner.read_token() {
            tokens.push(n);
        };
    }));
}

criterion_group!(benches, scan_lipsum100k);
criterion_main!(benches);