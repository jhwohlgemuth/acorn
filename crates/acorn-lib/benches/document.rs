#![allow(clippy::expect_used, missing_docs)]
use acorn::io::document::{DocumentIndex, DocumentPath, DocumentQuery, SourceDocument};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

const DOCUMENT: &str = include_str!("../../../tests/fixtures/data/invalid_project_b/index.yaml");

fn bench_document_resolver(c: &mut Criterion) {
    let document = SourceDocument::init().content(DOCUMENT).format("yaml").source("index.yaml").build();
    let index = DocumentIndex::new(document);
    let exact = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.graphics[0].contentUrl"))
        .with_value("00.invalid");
    let fallback = DocumentQuery::new()
        .with_path(DocumentPath::parse("meta.media[0].content_url"))
        .with_value("00.invalid");
    let mut group = c.benchmark_group("document.resolve");
    group.bench_function("exact", |b| b.iter(|| black_box(&index).resolve(black_box(&exact))));
    group.bench_function("source_aware_fallback", |b| b.iter(|| black_box(&index).resolve(black_box(&fallback))));
    group.finish();
}

criterion_group!(benches, bench_document_resolver);
criterion_main!(benches);
