#![allow(clippy::arithmetic_side_effects, clippy::expect_used, missing_docs)]
use acorn_diff::{unified_diff, DiffValue};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::{json, Map, Value};

const SIZES: [usize; 3] = [16, 256, 4096];

fn object(size: usize, changed: Option<usize>) -> Value {
    Value::Object(
        (0..size)
            .map(|index| (format!("field-{index:05}"), json!(if Some(index) == changed { -1 } else { index as i64 })))
            .collect::<Map<String, Value>>(),
    )
}
fn sequence(values: impl IntoIterator<Item = i64>) -> Value {
    Value::Array(values.into_iter().map(Value::from).collect())
}
fn lines(size: usize, changed: Option<usize>) -> String {
    (0..size)
        .map(|index| if Some(index) == changed { "changed line\n" } else { "repeated line\n" })
        .collect()
}
fn patch_label(size: usize, before: &Value, after: &Value) -> String {
    let patch = before.diff(after);
    format!(
        "{size}/ops={}/bytes={}",
        patch.operations().len(),
        serde_json::to_string(&patch).expect("serializable patch").len()
    )
}
fn bench_json(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("diff.json");
    SIZES.into_iter().for_each(|size| {
        let cases = [
            ("object_one_field", object(size, None), object(size, Some(size / 2))),
            ("array_append", sequence(0..size as i64), sequence(0..=size as i64)),
            (
                "array_front_insert",
                sequence(0..size as i64),
                sequence(core::iter::once(-1).chain(0..size as i64)),
            ),
        ];
        cases.iter().for_each(|(name, before, after)| {
            group.bench_with_input(BenchmarkId::new(*name, patch_label(size, before, after)), &size, |bencher, _| {
                bencher.iter(|| black_box(before).diff(black_box(after)))
            });
        });
    });
    group.finish();
}
fn bench_unified(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("diff.unified");
    SIZES.into_iter().for_each(|size| {
        let before = lines(size, None);
        let after = lines(size, Some(size / 2));
        let bytes = unified_diff(&before, &after, "before", "after").len();
        group.bench_with_input(
            BenchmarkId::new("repetitive_one_line", format!("{size}/bytes={bytes}")),
            &size,
            |bencher, _| bencher.iter(|| unified_diff(black_box(&before), black_box(&after), "before", "after")),
        );
    });
    group.finish();
}
criterion_group!(benches, bench_json, bench_unified);
criterion_main!(benches);
