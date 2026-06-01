#![allow(missing_docs)]
use acorn::analyzer::readability::{
    automated_readability_index, coleman_liau_index, expand_acronyms, flesch_kincaid_grade_level, flesch_reading_ease_score, gunning_fog_index, lix,
    smog, syllable_count, word_count,
};
use acorn::prelude::HashMap;
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

const SAMPLE_TEXT: &str = "ACORN standardizes and automates the research content creation process to better communicate projects, capabilities, and technology. Research project communication mediums should be straightforward and standardized, but the creation process is often cumbersome and involves multiple editing rounds among different teams.";

fn bench_readability_metrics(c: &mut Criterion) {
    let text = SAMPLE_TEXT.repeat(8);
    let mut group = c.benchmark_group("readability.metrics");
    group.bench_function("word_count", |b| b.iter(|| black_box(word_count(black_box(text.as_str())))));
    group.bench_function("syllable_count", |b| b.iter(|| black_box(syllable_count(black_box(text.as_str())))));
    group.bench_function("ari", |b| b.iter(|| black_box(automated_readability_index(black_box(text.as_str())))));
    group.bench_function("cli", |b| b.iter(|| black_box(coleman_liau_index(black_box(text.as_str())))));
    group.bench_function("fkgl", |b| b.iter(|| black_box(flesch_kincaid_grade_level(black_box(text.as_str())))));
    group.bench_function("fres", |b| b.iter(|| black_box(flesch_reading_ease_score(black_box(text.as_str())))));
    group.bench_function("gfi", |b| b.iter(|| black_box(gunning_fog_index(black_box(text.as_str())))));
    group.bench_function("lix", |b| b.iter(|| black_box(lix(black_box(text.as_str())))));
    group.bench_function("smog", |b| b.iter(|| black_box(smog(black_box(text.as_str())))));
    group.finish();
}
fn bench_readability_acronym_expansion(c: &mut Criterion) {
    let text = "Use ARI, CLI, and FKGL in ACORN quality checks. ACORN also reports FRES and GFI.".repeat(32);
    let acronyms = HashMap::from([
        ("ACORN".to_string(), "Accessible Content Optimization for Research Needs".to_string()),
        ("ARI".to_string(), "Automated Readability Index".to_string()),
        ("CLI".to_string(), "Coleman Liau Index".to_string()),
        ("FKGL".to_string(), "Flesch Kincaid Grade Level".to_string()),
        ("FRES".to_string(), "Flesch Reading Ease Score".to_string()),
        ("GFI".to_string(), "Gunning Fog Index".to_string()),
    ]);
    let mut group = c.benchmark_group("readability.preprocessing");
    group.bench_function("expand_acronyms", |b| {
        b.iter(|| {
            let result = expand_acronyms(black_box(text.as_str()), black_box(&acronyms));
            black_box(result)
        })
    });
    group.finish();
}
criterion_group!(benches, bench_readability_metrics, bench_readability_acronym_expansion);
criterion_main!(benches);
