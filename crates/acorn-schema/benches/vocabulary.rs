//! Controlled-vocabulary resolution benchmarks.
use acorn_schema::vocabulary::{VocabularyFuzzyMode, VocabularyKind, VocabularyOptions, VocabularyResolver};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn benchmark_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("vocabulary-construction");
    [
        VocabularyKind::Keywords,
        VocabularyKind::Organizations,
        VocabularyKind::Partners,
        VocabularyKind::Sponsors,
        VocabularyKind::Technology,
    ]
    .into_iter()
    .for_each(|kind| {
        group.bench_with_input(BenchmarkId::from_parameter(kind), &kind, |b, kind| {
            b.iter(|| VocabularyResolver::embedded(black_box(*kind)));
        });
    });
    group.finish();
}

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
            group.bench_with_input(BenchmarkId::new("typo-relaxed", kind.to_string()), &input, |b, value| {
                b.iter(|| {
                    resolver.resolve(
                        [black_box(*value)],
                        VocabularyOptions {
                            fuzzy: VocabularyFuzzyMode::Relaxed,
                            ..VocabularyOptions::default()
                        },
                    )
                })
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
            let exact = vec!["automation"; 256];
            let unmatched = (0..256).map(|index| format!("protected-hard-negative-{index:03}")).collect::<Vec<_>>();
            group.bench_function("exact-heavy-256", |b| {
                b.iter(|| resolver.resolve(black_box(exact.clone()), VocabularyOptions::default()))
            });
            group.bench_function("maximum-batch-relaxed", |b| {
                b.iter(|| resolver.resolve(black_box(batch.clone()), VocabularyOptions::default()))
            });
            group.bench_function("maximum-batch-strict", |b| {
                b.iter(|| {
                    resolver.resolve(
                        black_box(batch.clone()),
                        VocabularyOptions {
                            fuzzy: VocabularyFuzzyMode::Strict,
                            ..VocabularyOptions::default()
                        },
                    )
                })
            });
            group.bench_function("unmatched-worst-case-256", |b| {
                b.iter(|| resolver.resolve(black_box(unmatched.clone()), VocabularyOptions::default()))
            });
        });
    group.finish();
}

criterion_group!(benches, benchmark_construction, benchmark_resolution);
criterion_main!(benches);
