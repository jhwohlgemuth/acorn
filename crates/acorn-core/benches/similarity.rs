#![allow(clippy::arithmetic_side_effects, clippy::expect_used, missing_docs)]
use acorn_core::similarity::{
    character_ngrams, damerau_levenshtein_distance, jaccard_similarity, jaro_winkler_similarity, levenshtein_distance,
    optimal_string_alignment_distance, word_tokens, NgramOptions, SimilarityLimits, TextPreprocessing,
};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_edit_measures(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let mut group = criterion.benchmark_group("similarity.edit");
    for size in [16, 64, 256, 1_024] {
        let left = "research".repeat(size / 8);
        let right = format!("{}h", "researc".repeat(size / 7));
        group.bench_with_input(BenchmarkId::new("levenshtein", size), &size, |bencher, _| {
            bencher.iter(|| levenshtein_distance(black_box(&left), black_box(&right), limits).expect("bounded comparison"))
        });
        group.bench_with_input(BenchmarkId::new("optimal_string_alignment", size), &size, |bencher, _| {
            bencher.iter(|| optimal_string_alignment_distance(black_box(&left), black_box(&right), limits).expect("bounded comparison"))
        });
        group.bench_with_input(BenchmarkId::new("damerau_levenshtein", size), &size, |bencher, _| {
            bencher.iter(|| damerau_levenshtein_distance(black_box(&left), black_box(&right), limits).expect("bounded comparison"))
        });
        group.bench_with_input(BenchmarkId::new("jaro_winkler", size), &size, |bencher, _| {
            bencher.iter(|| jaro_winkler_similarity(black_box(&left), black_box(&right), limits).expect("bounded comparison"))
        });
    }
    group.finish();
    let left = "é🙂research".repeat(32);
    let right = "é🙃research".repeat(32);
    criterion.bench_function("similarity.edit.unicode_levenshtein", |bencher| {
        bencher.iter(|| levenshtein_distance(black_box(&left), black_box(&right), limits).expect("bounded comparison"))
    });
}
fn bench_preparation(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let text = "Research data software analysis ".repeat(64);
    let ngrams = NgramOptions::new(3).expect("valid width");
    let unicode = "é🙂research-data".repeat(64);
    let mut group = criterion.benchmark_group("similarity.preparation");
    group.bench_function("character_ngrams", |bencher| {
        bencher.iter(|| character_ngrams(black_box(&unicode), TextPreprocessing::Lowercase, ngrams, limits).expect("bounded preparation"))
    });
    group.bench_function("word_tokens", |bencher| {
        bencher.iter(|| word_tokens(black_box(&text), TextPreprocessing::Lowercase, limits).expect("bounded preparation"))
    });
    group.finish();
}
fn bench_set_measure(criterion: &mut Criterion) {
    let left = (0..256).map(|value| format!("term-{value}")).collect::<Vec<_>>();
    let right = (128..384).map(|value| format!("term-{value}")).collect::<Vec<_>>();
    criterion.bench_function("similarity.set.jaccard", |bencher| {
        bencher.iter(|| jaccard_similarity(black_box(&left), black_box(&right), SimilarityLimits::default()).expect("bounded comparison"))
    });
}

criterion_group!(benches, bench_edit_measures, bench_preparation, bench_set_measure);
criterion_main!(benches);
