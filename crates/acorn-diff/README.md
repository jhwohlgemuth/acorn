# acorn-diff

Portable, deterministic unified-text and RFC 6902 JSON Patch primitives for ACORN and other Rust applications.

The default `std` feature supports native applications. Portable consumers can disable default features and enable the minimum allocation-backed API:

```toml
[dependencies]
acorn-diff = { version = "0.3.2", default-features = false, features = ["alloc"] }
```

Create a labeled unified line diff:

```rust
use acorn_diff::unified_diff;

let output = unified_diff("old\n", "new\n", "a/file", "b/file");
assert!(output.starts_with("--- a/file\n+++ b/file\n@@"));
assert!(output.contains("-old"));
assert!(output.contains("+new"));
```

Create a deterministic JSON Patch from two `serde_json::Value` instances:

```rust
use acorn_diff::DiffValue;
use serde_json::json;

let patch = json!({"name": "before"}).diff(&json!({"name": "after"}));
let encoded = serde_json::to_value(patch).expect("patch serializes");
assert_eq!(encoded, json!([{"op": "replace", "path": "/name", "value": "after"}]));
```

Generated patches use deterministic object-key ordering and the `add`, `remove`, and `replace` operation subset. The `Rfc6902` trait exposes validated constructors for all six RFC 6902 operation shapes. ACORN-specific format detection and canonicalization remain in `acorn::analyzer::diff`.
