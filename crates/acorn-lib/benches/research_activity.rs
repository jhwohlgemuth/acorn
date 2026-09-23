#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    missing_docs
)]
use acorn_core::util::MarkdownSupport;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::zon;
use acorn_schema::validation::Validate;
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

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
fn bench_deserialize_zon(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    let encoded = zon::encode(&value).expect("valid_project fixture should serialize as ZON");
    let mut group = c.benchmark_group("research_activity.deserialize_zon");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.bench_function("zon", |b| {
        b.iter(|| {
            let value = zon::decode::<ResearchActivity>(black_box(&encoded));
            black_box(value)
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
fn bench_serialize_zon(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    let bytes = zon::encode(&value).expect("valid_project fixture should serialize as ZON").len();
    let mut group = c.benchmark_group("research_activity.serialize_zon");
    group.throughput(Throughput::Bytes(bytes as u64));
    group.bench_function("zon", |b| {
        b.iter(|| {
            let result = zon::encode(black_box(&value));
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
fn bench_validate(c: &mut Criterion) {
    let value: ResearchActivity = serde_json::from_str(VALID_PROJECT).expect("valid_project fixture should deserialize");
    c.bench_function("research_activity.validate", |b| {
        b.iter(|| {
            let result = core::hint::black_box(&value).validate();
            black_box(result)
        });
    });
}
criterion_group!(
    benches,
    bench_deserialize_json,
    bench_deserialize_zon,
    bench_validate,
    bench_serialize_json,
    bench_serialize_zon,
    bench_to_markdown,
    bench_format
);
criterion_main!(benches);
