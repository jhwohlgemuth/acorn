//! End-to-end controlled-vocabulary benchmarks.
use acorn::io::vocabulary::cached_vocabulary_resolver_in;
use acorn_analysis::CheckBatch;
use acorn_schema::{
    research_activity::ResearchActivity,
    vocabulary::{VocabularyFuzzyMode, VocabularyKind, VocabularyResolver},
    OneOrMany,
};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::tempdir;

const RESEARCH_ACTIVITY: &str = include_str!("../../../tests/fixtures/data/valid_project/index.json");

// Criterion's benchmark callback cannot propagate fixture setup failures.
#[allow(clippy::expect_used)]
fn benchmark_all_fields(c: &mut Criterion) {
    let mut data = serde_json::from_str::<ResearchActivity>(RESEARCH_ACTIVITY).expect("research activity fixture");
    data.contact.affiliation = Some("Oak Ridge National Labortory".to_string());
    data.meta.partners = Some(OneOrMany::Many(vec![
        "National Renewable Energy Labortory".to_string(),
        "Kitware, Inc.".to_string(),
    ]));
    let mut group = c.benchmark_group("vocabulary-all-fields");
    [VocabularyFuzzyMode::Relaxed, VocabularyFuzzyMode::Strict].into_iter().for_each(|mode| {
        group.bench_with_input(BenchmarkId::from_parameter(mode), &mode, |b, mode| {
            b.iter(|| {
                CheckBatch::resolve_research_activity_vocabulary(black_box(&data), black_box(*mode), "benchmark.json", VocabularyResolver::embedded)
            });
        });
    });
    group.finish();
}

// Criterion's benchmark callback cannot propagate cache setup failures.
#[allow(clippy::expect_used)]
fn benchmark_startup(c: &mut Criterion) {
    let directory = tempdir().expect("benchmark cache directory");
    let mut group = c.benchmark_group("vocabulary-startup");
    group.bench_function("cached", |b| {
        cached_vocabulary_resolver_in(VocabularyKind::Organizations, directory.path()).expect("warm cache");
        b.iter(|| cached_vocabulary_resolver_in(black_box(VocabularyKind::Organizations), directory.path()));
    });
    group.bench_function("uncached", |b| {
        b.iter(|| VocabularyResolver::embedded(black_box(VocabularyKind::Organizations)));
    });
    group.finish();
}

criterion_group!(benches, benchmark_all_fields, benchmark_startup);
criterion_main!(benches);
