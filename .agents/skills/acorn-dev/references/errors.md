# Errors, unwrap, and eserde

## Error reporting

Use `color-eyre`:

```rust
use color_eyre::eyre::{self, eyre, Result, WrapErr};

Err(eyre!("Configuration file cannot be empty"))
```

Command functions return `Void` (`eyre::Result<(), eyre::Report>`).

## Fixing `clippy::unwrap_used`

The workspace sets `unwrap_used = "warn"`. `clippy.toml` allows `unwrap()`/`expect()` in `#[test]` functions and `#[cfg(test)]` modules, so test code needs no attribute. Production code must not use bare `.unwrap()`. Prefer these strategies in order:

1. **Arithmetic rounding:** `format!("{val:.2}").parse().unwrap()` → `(val * 100.0).round() / 100.0`.
2. **Propagate:** when the function already returns `Result` or `Option`, propagate the error instead of panicking (follow the `guidelines` skill's error-handling rules).
3. **Fallbacks:** `unwrap_or_default()` or `unwrap_or(fallback)` when a sensible default exists.
4. **Iterator-safe alternatives:**
   ```rust
   // Before
   iter.filter(|x| !x.as_ref().unwrap().is_empty())
   // After
   iter.filter_map(Result::ok).filter(|x| !x.is_empty())
   ```
5. **Pattern matching:** replace guarded `.unwrap()` with `if let` or `let else`.
6. **`#[allow(clippy::unwrap_used)]` with a rationale comment:** only for genuinely infallible cases (hardcoded patterns, known-good templates, guarded invariants) where nothing above applies.

## eserde multi-error deserialization

Use `eserde` at user-facing JSON entry points to report every deserialization error instead of the first.

1. Change `#[derive(Deserialize)]` to `#[derive(eserde::Deserialize)]` on top-level structs only; it generates both `serde::Deserialize` and `eserde::EDeserialize`.
2. Every field type must implement `EDeserialize`. Add the field-level `#[eserde(compat)]` to fields whose types still use vanilla `serde::Deserialize`. Primitives, `String`, `Vec<String>`, and `Option<String>` need no compat.
3. Replace `serde_json::from_str` with `eserde::json::from_str`, which returns `DeserializationErrors` (`.iter()`, `.len()`; each error has `.message()` and `.path()`).

```rust
#[derive(eserde::Deserialize, Serialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct MySchema {
    pub name: String,
    pub count: Option<u32>,
    #[eserde(compat)]
    pub nested: MyNestedType,
}

#[derive(Deserialize, Serialize)]
pub struct MyNestedType { /* ... */ }
```

```rust
fn read_json(path: PathBuf) -> ApiResult<MySchema> {
    read_file(path).and_then(|content| {
        eserde::json::from_str::<MySchema>(&content).map_err(|errors| {
            for error in errors.iter() {
                let path = error.path().map_or("root", |p| p);
                let message = error.message();
                fail!("{path}: {message}");
            }
            eyre!("JSON has {} deserialization error(s)", errors.len().to_string().bold())
        })
    })
}
```

Only user-facing `read_json` entry points use `eserde::json::from_str`. YAML (`serde_norway::from_str`) and internal API calls stay unchanged. Existing examples: `crates/acorn-schema/src/codec.rs` and `crates/acorn-schema/src/standard/ckan.rs`.
