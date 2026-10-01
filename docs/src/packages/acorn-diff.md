# Portable diffs

`acorn-diff` provides deterministic, in-memory diff primitives without depending on other ACORN crates or host services.

Its default `std` feature supports native applications. Disable default features and enable `alloc` for portable and WebAssembly-compatible consumers:

```toml
[dependencies]
acorn-diff = { version = "0.3.2", default-features = false, features = ["alloc"] }
```

Use `unified_diff` for labeled line-oriented output. Import `DiffValue` and call `before.diff(&after)` to create an RFC 6902 patch over two `serde_json::Value` instances. Generated patches use deterministic object-key ordering and the `add`, `remove`, and `replace` operation subset; the `Rfc6902` trait exposes validated constructors for all six operation shapes.

ACORN-specific format detection and canonicalization live in `acorn::analyzer::diff`. The `acorn diff BEFORE AFTER` command uses that adapter to compare files in the same metadata standard and can emit unified output or JSON Patch.
