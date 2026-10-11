#![allow(clippy::arithmetic_side_effects, clippy::expect_used, missing_docs)]
use acorn_core::similarity::{
    character_ngrams, cologne_phonetic, damerau_levenshtein_distance, jaccard_similarity, jaro_winkler_similarity, levenshtein_distance, metaphone,
    monge_elkan_similarity, needleman_wunsch_score, optimal_string_alignment_distance, soft_tfidf_similarity, soundex, word_tokens, AlignmentScoring,
    NgramOptions, SimilarityLimits, SimilarityPreprocessing, SimilarityScore, TfidfCorpus,
};
use acorn_core::{OneOrMany, ThisOrThat};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_alignment(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let scoring = AlignmentScoring {
        gap: -1,
        matched: 2,
        mismatched: -1,
    };
    let mut group = criterion.benchmark_group("similarity.alignment");
    for size in [16, 64, 256] {
        let left = "ACGT".repeat(size / 4);
        let right = "AGGT".repeat(size / 4);
        group.bench_with_input(BenchmarkId::new("needleman_wunsch", size), &size, |bencher, _| {
            bencher.iter(|| {
                needleman_wunsch_score(black_box(ThisOrThat::new(left.as_str(), right.as_str())), scoring, limits).expect("bounded alignment")
            })
        });
    }
    group.finish();
}
fn bench_corpus(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let mut group = criterion.benchmark_group("similarity.corpus");
    for documents in [16, 64, 256] {
        let corpus = (0..documents)
            .map(|document| (0..32).map(|token| format!("term-{}-{token}", document % 8)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        group.bench_with_input(BenchmarkId::new("construct", documents), &documents, |bencher, _| {
            bencher.iter(|| TfidfCorpus::new(black_box(&corpus), limits).expect("bounded corpus"))
        });
    }
    group.finish();
}
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
fn bench_hybrid(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let left = OneOrMany::from((0..64).map(|index| format!("research-{index}")).collect::<Vec<_>>());
    let right = OneOrMany::from((0..64).map(|index| format!("researhc-{index}")).collect::<Vec<_>>());
    let scorer = |left: &str, right: &str| jaro_winkler_similarity(left, right, limits);
    let corpus = TfidfCorpus::new(&[left.clone(), right.clone()], limits).expect("explicit corpus");
    let threshold = SimilarityScore::new(0.8).expect("valid threshold");
    let mut group = criterion.benchmark_group("similarity.hybrid");
    group.bench_function("monge_elkan_nested", |bencher| {
        bencher.iter(|| monge_elkan_similarity(black_box(ThisOrThat::new(&left, &right)), scorer, limits).expect("bounded comparison"))
    });
    group.bench_function("soft_tfidf_nested", |bencher| {
        bencher.iter(|| soft_tfidf_similarity(&corpus, black_box(ThisOrThat::new(&left, &right)), threshold, scorer).expect("bounded comparison"))
    });
    group.finish();
}
fn bench_phonetic(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let mut group = criterion.benchmark_group("similarity.phonetic");
    group.bench_function("cologne", |bencher| {
        bencher.iter(|| cologne_phonetic(black_box("Müller-Lüdenscheidt"), limits).expect("bounded encoding"))
    });
    group.bench_function("metaphone", |bencher| {
        bencher.iter(|| metaphone(black_box("Schneider"), limits).expect("bounded encoding"))
    });
    group.bench_function("soundex", |bencher| {
        bencher.iter(|| soundex(black_box("Ashcraft"), limits).expect("bounded encoding"))
    });
    group.finish();
}
fn bench_preparation(criterion: &mut Criterion) {
    let limits = SimilarityLimits::default();
    let text = "Research data software analysis ".repeat(64);
    let ngrams = NgramOptions::new(3).expect("valid width");
    let unicode = "é🙂research-data".repeat(64);
    let mut group = criterion.benchmark_group("similarity.preparation");
    group.bench_function("character_ngrams", |bencher| {
        bencher.iter(|| character_ngrams(black_box(&unicode), SimilarityPreprocessing::Lowercase, ngrams, limits).expect("bounded preparation"))
    });
    group.bench_function("word_tokens", |bencher| {
        bencher.iter(|| word_tokens(black_box(&text), SimilarityPreprocessing::Lowercase, limits).expect("bounded preparation"))
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

criterion_group!(
    benches,
    bench_alignment,
    bench_corpus,
    bench_edit_measures,
    bench_hybrid,
    bench_phonetic,
    bench_preparation,
    bench_set_measure
);
criterion_main!(benches);
