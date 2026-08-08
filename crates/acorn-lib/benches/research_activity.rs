#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    missing_docs
)]
use acorn::schema::research_activity::ResearchActivity;
use acorn::util::ToMarkdown;
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use validator::Validate;

const VALID_PROJECT: &str = include_str!("../../../tests/fixtures/data/valid_project/index.json");

fn bench_deserialize_json(c: &mut Criterion) {
    c.bench_function("research_activity.deserialize_json", |b| {
        b.iter(|| {
            let value: ResearchActivity =
                serde_json::from_str(core::hint::black_box(VALID_PROJECT)).expect("valid_project fixture should deserialize");
            black_box(value)
        });
    });
}
fn bench_validate(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    c.bench_function("research_activity.validate", |b| {
        b.iter(|| {
            let result = core::hint::black_box(&value).validate();
            black_box(result)
        });
    });
}
fn bench_serialize_json(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    let mut group = c.benchmark_group("research_activity.serialize");
    group.bench_function("json", |b| {
        b.iter(|| {
            let result = serde_json::to_string(core::hint::black_box(&value));
            black_box(result)
        });
    });
    group.finish();
}
fn bench_to_markdown(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    let mut group = c.benchmark_group("research_activity.render");
    group.bench_function("markdown", |b| {
        b.iter(|| {
            let result = core::hint::black_box(&value).to_markdown();
            black_box(result)
        });
    });
    group.finish();
}
fn bench_format(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    let mut group = c.benchmark_group("research_activity.transform");
    group.bench_function("format", |b| {
        b.iter(|| {
            let result = core::hint::black_box(value.clone()).format();
            black_box(result)
        });
    });
    group.finish();
}
criterion_group!(
    benches,
    bench_deserialize_json,
    bench_validate,
    bench_serialize_json,
    bench_to_markdown,
    bench_format
);
criterion_main!(benches);
