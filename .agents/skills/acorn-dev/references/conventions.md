# ACORN conventions

## Logging and CLI output

Use `tracing` for logs and `Label::*()` from `acorn_host::terminal` for status markers, with `owo_colors` for extra styling:

```rust
use acorn_host::terminal::Label;
use owo_colors::OwoColorize;
use tracing::{debug, info, instrument, warn};

#[instrument(skip(data))]
fn process_data(data: Vec<u8>) -> eyre::Result<()> {
    debug!("Processing {} bytes", data.len());
    info!("{} {} items processed", Label::pass(), data.len().cyan().bold());
    Ok(())
}
```

Labels include `pass()`, `fail()`, `run()`, `skip()`, and `using()`. Never use bare `println!` for status output.

Verbosity: `-v` info, `-vv` debug, `-vvv`/`-vvvv` trace. The `ACORN_LOG_LEVEL` environment variable (usually set in `.env`) also controls the level.

## Feature flags

`acorn-lib` defaults to `["std"]` and layers features such as `core`, `alloc`, `wasm`, `std`, `net` (HTTP, lychee, tokio), `archive`, `template`, `progress`, `cli`, `db`, `analysis`, and `agentic`. Check `[features]` in `crates/acorn-lib/Cargo.toml` before gating code, and gate host-only code accordingly:

```rust
#[cfg(feature = "std")]
use std::fs::File;
```

Portable code must keep `make check-wasm` passing.

## Parallelism and builders

- Use `rayon` (`par_iter()`) for file and batch parallelism. The `--threads` argument sizes the global pool in `crates/acorn-cli/src/main.rs`.
- Use the `bon` crate (`#[derive(Builder)]`) for type-safe builders.

## Documentation

- All public modules, functions, and types need `///` docs (`missing_docs = "deny"`); intra-doc links must resolve (`broken_intra_doc_links = "forbid"`).
- Add `# Examples`, `# Errors`, and `# Panics` sections to complex public APIs.
- Update `README.md` for user-facing features and `ARCHITECTURE.md` for structural changes.
- Build docs with `make doc`; build or serve the book with `make book` / `make book-write`.

## Dependencies

Shared versions live in `[workspace.dependencies]` in the root `Cargo.toml`. Run `make audit` after changes and update `deny.toml` for new license concerns.

## Common pitfalls

```rust
// Missing validation
pub name: String,
// Better
#[validate(length(min = 1, max = 100))]
#[serde(deserialize_with = "string_trim")]
pub name: String,

// Missing nested validation
pub contact: ContactPoint,
// Better
#[validate(nested)]
pub contact: ContactPoint,

// Ignoring offline mode
let response = reqwest::get(url)?;
// Better
if offline {
    return Err(eyre!("Network unavailable in offline mode"));
}
let response = network_get_request(url)?;
```

## Performance

- Stream file operations instead of loading whole files where practical.
- Use `fancy_regex` for complex patterns and cache compiled regexes.
