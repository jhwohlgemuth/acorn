//! Controlled-vocabulary resolution benchmarks.
use acorn_schema::vocabulary::{VocabularyKind, VocabularyOptions, VocabularyResolver};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn benchmark_resolution(c: &mut Criterion) {
    let cases = [
        (VocabularyKind::Keywords, "high-performance-computi"),
        (VocabularyKind::Organizations, "Geospatial Science and Human Security Divison"),
        (VocabularyKind::Partners, "National Renewable Energy Labortory"),
        (VocabularyKind::Sponsors, "Department of Energry"),
        (VocabularyKind::Technology, "typescirpt"),
    ];
    let mut group = c.benchmark_group("vocabulary-resolution");
    cases.into_iter().for_each(|(kind, input)| {
        VocabularyResolver::embedded(kind).ok().into_iter().for_each(|resolver| {
            group.bench_with_input(BenchmarkId::new("single", kind.to_string()), &input, |b, value| {
                b.iter(|| resolver.resolve([black_box(*value)], VocabularyOptions::default()))
            });
        });
    });
    VocabularyResolver::embedded(VocabularyKind::Keywords)
        .ok()
        .into_iter()
        .for_each(|resolver| {
            let batch = (0..256)
                .map(|index| match index % 3 {
                    | 0 => "high-performance-computi",
                    | 1 => "automatin",
                    | _ => "not-a-keyword",
                })
                .collect::<Vec<_>>();
            group.bench_function("batch-256", |b| {
                b.iter(|| resolver.resolve(black_box(batch.clone()), VocabularyOptions::default()))
            });
        });
    group.finish();
}

criterion_group!(benches, benchmark_resolution);
criterion_main!(benches);
