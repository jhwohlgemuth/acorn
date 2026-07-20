# AGENTS.md

**USE [guidelines SKILL](./.github/skills/guidelines/SKILL.md) WHENEVER CODE IS TOUCHED OR CONSIDERED**

This file provides guidance to agents (including WARP) when working with code in this repository.

ACORN (Accessible Content Optimization for Research Needs) is a Rust-based CLI tool and library for maintaining and validating research activity data (RAD).

## Quick Start Decision Tree

When assigned a task:

1. **Is it a bug fix or feature addition?**
   - Bug: Review existing tests in `#[cfg(test)]` modules, run `make test`, ensure no regression
   - Feature: Start by reading relevant schema/command module, check if new field needs validation

2. **Does it involve schema changes?**
   - YES → Update `ResearchActivity` or relevant struct in `acorn-lib/src/schema/`
   - Add validators (`#[validate(...)]`)
   - Add/update tests in adjacent `#[cfg(test)]` module
   - Update ARCHITECTURE.md if major structural change

3. **Does it involve CLI output?**
   - YES → Use `Label::*()` methods from `acorn-lib/src/util/mod.rs` for consistent coloring
   - Use `owo_colors` for additional styling
   - Test with `-vvv` or `-vvvv` verbose flags

4. **Does it require external data or network?**
   - YES → Respect `offline` flag (passed through most command functions)
   - Use `acorn-lib/src/io/` utilities for file/network operations
   - Check `tests/fixtures/` for test data

## Development Commands

### Essential Commands
```bash
# Format and lint code
make lint

# Run all tests (uses cargo-nextest)
make test

# Build release binaries
make build

# Clean build artifacts
make clean

# Generate documentation
make doc

# Regenerate command-line help markdown
make help

# Run tests with coverage
make cover

# Security audit
make audit
```

### Running From Source
ACORN's full API is available without installation:
```bash
# Instead of: acorn check /path/to/folder
cargo run --bin acorn -- check /path/to/folder

# Instead of: acorn export --format pdf path/to/file.json
cargo run --bin acorn -- export --format pdf path/to/file.json
```

## Common Tasks

### Adding a Schema Field
1. Edit struct in `acorn-lib/src/schema/research_activity/mod.rs` (or relevant schema file)
2. Add validation attributes using `validator` crate:
   ```rust
   #[validate(length(min = 1, max = 100))]
   #[serde(deserialize_with = "string_trim")]
   pub new_field: String,
   ```
3. If field is nested, add `#[validate(nested)]`
4. Add test case in `#[cfg(test)]` module
5. Update any related command that processes the field
6. Update JSON schema if needed (schemas are auto-generated via `schemars`)

### Adding a CLI Command
1. Create `acorn-cli/src/commands/{command_name}/mod.rs`
2. Add `pub fn run(...)` that takes parsed arguments and returns `color_eyre::eyre::Result<(), eyre::Report>`
3. Export module in `acorn-cli/src/commands/mod.rs`
4. Add command variant to `Commands` enum in `acorn-cli/src/cli/mod.rs`
5. Add match arm in `acorn-cli/src/main.rs` to call your run function
6. Use `#[derive(Parser)]` from `clap` for argument parsing

### Adding Analysis/Checking Logic
1. Add module in `acorn-lib/src/analyzer/`
2. Integrate result into `Check` struct (see `acorn-lib/src/analyzer/mod.rs`)
3. Call from `acorn-cli/src/commands/check/mod.rs`
4. Surface results using `Label::*()` for output

### Fixing Export/PowerPoint Issues
1. Check `acorn-lib/src/powerpoint/mod.rs` for OOXML manipulation
2. Check `acorn-cli/src/template/mod.rs` for HTML template generation
3. Test both PDF and PPTX targets from `acorn-cli/src/cli/arguments.rs`
4. Verify embedded assets are included (fonts, images, skills in `acorn-cli/assets/`)

## Testing Strategy

### Unit Tests
- Located inline with `#[cfg(test)]` modules
- Use simple `#[test]` functions, no external frameworks
- Test happy path AND edge cases

Example pattern:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_with_valid_input() {
        let result = my_function("valid");
        assert!(result.is_ok());
    }
    #[test]
    fn test_function_with_edge_case() {
        let result = my_function("");
        assert!(result.is_err());
    }
}
```

### Running Tests
```bash
# All tests (preferred)
make test
# OR
cargo nextest run --run-ignored all

# Specific test module
cargo nextest run --package acorn-lib analyzer

# Specific test function
cargo nextest run --package acorn-cli commands::check::tests::test_check_command

# Run tests with cargo (for CI)
make test-ci

# With coverage report
make cover
# OR
cargo tarpaulin --out lcov --line --skip-clean --no-dead-code --engine llvm
```

### Test Fixtures
Located in `tests/fixtures/`:
- `buckets.json`: Repository configuration examples
- `raid/`: RAiD metadata test data
- `glob/`: Pattern matching test files

## Fixing `clippy::unwrap_used` Warnings

The workspace enables `unwrap_used = "warn"` in Cargo.toml. Test modules suppress it with `#[allow(clippy::unwrap_used)]`, but production code must not use bare `.unwrap()`. Use these strategies (in order of preference):

### Strategy 1: Arithmetic rounding
`format!("{val:.2}").parse().unwrap()` → `(val * 100.0).round() / 100.0`

### Strategy 2: Avoid `?` operator
When the function already returns `Result<T, E>` or `Option<T>`, propagate errors instead of panicking.

### Strategy 3: `unwrap_or_default()` / `unwrap_or(fallback)`
When a sensible fallback exists (e.g., `.unwrap_or(0)`, `.unwrap_or_default()`, `.unwrap_or("unknown")`).

### Strategy 4: Iterator-safe alternatives
```rust
// Before
iter.filter(|x| !x.as_ref().unwrap().is_empty())
// After
iter.filter_map(Result::ok).filter(|x| !x.is_empty())
```

### Strategy 5: `if let` / pattern matching
Replace guarded `.unwrap()` with `if let` destructuring or early returns with `let else`.

### Strategy 6: `#[allow(clippy::unwrap_used)]` with rationale
Use only when the unwrap is genuinely infallible (hardcoded patterns, known-good templates, guarded invariants) and none of the above strategies apply. Document *why* it's safe with a comment.

## Code Review Checklist

Before submitting changes, verify:

- [ ] No `unsafe` keyword appears (workspace forbids it with `unsafe_code = "forbid"`)
- [ ] All public items have doc comments (`///`) - workspace denies `missing-docs`
- [ ] No new warnings from `cargo clippy --no-deps`
- [ ] Code formatted: `cargo fmt` run successfully
- [ ] All tests pass: `make test` or `cargo nextest run --run-ignored all`
- [ ] No intra-doc link errors (workspace forbids `broken-intra-doc-links`)
- [ ] Serde: Used `#[serde(deny_unknown_fields)]` for new public structs where appropriate
- [ ] Validation: Added appropriate `#[validate(...)]` attributes from `validator` crate
- [ ] CLI output: Uses `Label::*()` from `acorn::util` for consistent styling
- [ ] No new clippy warnings: `cargo clippy --workspace --no-deps --all-targets`
- [ ] Coverage not decreased: `make cover` (if applicable)

## Eserde Integration for Multi-Error Deserialization

Use `eserde` to collect **all** serde deserialization errors from user-facing JSON entry points instead of failing on the first error.

### How It Works
1. Change `#[derive(Deserialize)]` to `#[derive(eserde::Deserialize)]` on top-level structs only
2. The derive generates both `serde::Deserialize` + `eserde::EDeserialize`
3. ALL field types must implement `EDeserialize` — use `#[eserde(compat)]` on fields whose types still use vanilla `serde::Deserialize`
4. Replace `serde_json::from_str` with `eserde::json::from_str` — it returns `DeserializationErrors` with `.iter()`, `.len()`, and each error has `.message()` and `.path()`

### Migration Pattern
```rust
// Top-level struct (the entry point)
#[derive(eserde::Deserialize, Serialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct MySchema {
    // Primitive/String fields — no compat needed
    pub name: String,
    pub count: Option<u32>,
    // Nested types still using serde::Deserialize — add compat
    #[eserde(compat)]
    pub nested: MyNestedType,
}

// Nested types keep vanilla serde
#[derive(Deserialize, Serialize)]
pub struct MyNestedType { ... }
```

### Error Handling Pattern
```rust
fn read_json(path: PathBuf) -> ApiResult<MySchema> {
    read_file(path).and_then(|content| {
        eserde::json::from_str::<MySchema>(&content).map_err(|errors| {
            for error in errors.iter() {
                fail!("{}: {}", error.path().map_or("root", |p| p), error.message());
            }
            eyre!("JSON has {} deserialization error(s)", errors.len().to_string().bold())
        })
    })
}
```

### Important Notes
- Only use `eserde::json::from_str` at **user-facing entry points** (`read_json` methods)
- YAML paths (`serde_norway::from_str`) and internal API calls remain unchanged
- `#[eserde(compat)]` is a field-level attribute — it tells eserde to fall back to vanilla serde `Deserialize` for that specific field
- Primitive types (`String`, `bool`, `u32`, `Vec<String>`, `Option<String>`, etc.) implement `EDeserialize` automatically — no compat needed

## Project Structure

### Workspace Crates
The project uses Cargo workspaces with three main crates:
- **acorn-cli**: CLI application with commands (check, export, format, link, download, doctor)
- **acorn-lib**: Core library with schemas, validation, analysis, and utilities
- **acorn-py**: Python bindings (PyO3)

### Current Version
The current version is defined in `[workspace.package].version` in `Cargo.toml`.

### Feature Flags
The library uses feature flags for optional functionality:
- `default = ["analysis", "std"]`
- `std`: Standard library support (includes reqwest, lychee-lib, tokio, which, duct)
- `analysis`: Text analysis and readability metrics (includes bat)
- `doctor`: System diagnostics
- `powerpoint`: PowerPoint import/export functionality

When adding code that uses `std`:
```rust
#[cfg(feature = "std")]
use std::fs::File;
```

When adding code that uses analysis features:
```rust
#[cfg(feature = "analysis")]
use crate::analyzer::readability;
```

### Updating Dependencies
- Check `Cargo.toml` workspace.dependencies for shared versions
- Run `make audit` to check for security vulnerabilities
- Update `deny.toml` if adding new dependencies with license concerns

## Key Technologies

### Parallel Processing
ACORN uses `rayon` for parallel iteration over files/batches:

```rust
use rayon::prelude::*;

let results: Vec<_> = items
    .par_iter()
    .map(|item| process(item))
    .collect();
```

The `--threads` CLI argument controls the thread pool size (initialized in main.rs):
```rust
rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
```

### Builder Pattern
ACORN uses the `bon` crate for type-safe builder patterns:
```rust
use bon::Builder;

#[derive(Builder)]
pub struct MyStruct {
    field: String,
}
```

## Error Handling Pattern

Always use `color-eyre` for rich error reporting:

```rust
use color_eyre::eyre::{self, eyre, Result, WrapErr};

Err(eyre!("Configuration file cannot be empty"))
```

Note: Command functions return `Void` (aliased as `eyre::Result<(), eyre::Report>`) in main.rs.

## Logging and Output

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error, instrument};
use acorn::util::Label;
use owo_colors::OwoColorize;

#[instrument(skip(large_data))]
fn process_data(data: Vec<u8>, verbose: &Verbosity) -> eyre::Result<()> {
    debug!("Processing {} bytes", data.len());
    if data.is_empty() {
        warn!("Empty data provided");
    }
    info!("{} {} items processed", Label::pass(), count.cyan().bold());
    Ok(())
}
```

### Verbosity Configuration
- `-v`: info level
- `-vv`: debug level  
- `-vvv` or `-vvvv`: trace level
- Environment variable: `ACORN_LOG_LEVEL` (set in .env file)

### CLI Output Styling
Always use `Label::*()` methods for consistent output:
- `Label::pass()` - ✓ green checkmark
- `Label::fail()` - ✗ red X
- `Label::run()` - ⚙ indicator
- `Label::using()` - arrow indicator

Combine with `owo_colors` for additional styling:
```rust
info!("{} Processing {}", Label::run(), "file.json".cyan());
```

## When to Ask for Clarification

Stop and ask the user if:

1. **Ambiguous requirements**: "Should this field be validated as email or just non-empty?"
2. **Architecture conflicts**: "Adding this feature would require restructuring command flow—should I refactor?"
3. **Performance trade-offs**: "Caching this would use more memory—acceptable?"
4. **Breaking changes**: "This change modifies the schema structure—is backward compatibility needed?"
5. **Missing test data**: "No test fixture exists for this scenario—should I create one?"

## Important Implementation Details

### Schema System
- Main schemas are in `acorn-lib/src/schema/`:
  - `ResearchActivity` - Primary RAD schema
  - `pid::raid::Metadata` - RAiD metadata schema
- Schemas use `schemars` for JSON Schema generation
- Validation via `validator` crate with derive macros
- Serde trimming utilities: `string_trim`, `vec_string_trim`, `option_string_trim` from `serde_trim`

### CLI Architecture  
- Commands defined in `acorn-cli/src/commands/`
- Current commands: check, doctor, download, export, format, link, schema
- All commands accessed through main.rs match expression
- Arguments parsed via `clap` with `#[derive(Parser)]`
- `CommandLineHelp.md` is generated dynamically with `cargo run --bin acorn -- --markdown-help`; use `make help` after changing CLI arguments or command help text

### Analysis Features
- Analyzer module in `acorn-lib/src/analyzer/`
- Vale integration for prose analysis (vale.rs)
- Readability metrics (readability/ submodule)
- Website checking via `lychee-lib` (when std feature enabled)

### Export/PowerPoint
- Template generation in `acorn-cli/src/template/`
- PowerPoint manipulation in `acorn-lib/src/powerpoint/`
- Supports PDF and PPTX output formats
- Uses `rust-embed` for embedded assets (fonts, images, agent skills)

## Common Pitfalls to Avoid

### ❌ Missing Validation
```rust
// BAD
pub name: String,

// GOOD
#[validate(length(min = 1, max = 100))]
#[serde(deserialize_with = "string_trim")]
pub name: String,
```

### ❌ Ignoring Offline Mode
```rust
// BAD
let response = reqwest::get(url)?;

// GOOD
if offline {
    return Err(eyre!("Network unavailable in offline mode"));
}
let response = network_get_request(url)?;
```

### ❌ Mixing Label Output
```rust
// BAD
println!("Processing...");
println!("Done");
// GOOD
info!("{} Processing file", Label::run());
info!("{} Complete", Label::pass());
```

### ❌ Forgetting Nested Validation
```rust
// BAD
pub contact: ContactPoint,

// GOOD
#[validate(nested)]
pub contact: ContactPoint,
```

### ❌ No Doc Comments on Public Items
```rust
// BAD
pub fn process() { }

// GOOD
/// Processes research activity data with validation
/// 
/// # Examples
/// 
/// ```
/// let result = process()?;
/// ```
pub fn process() { }
```

## Debugging Workflow

### Common Issues

**Test failure after refactoring**
→ Check `#[cfg(test)]` modules in modified files
→ Run: `cargo test --lib -- --nocapture` to see full output
→ Use `RUST_BACKTRACE=1` for panic backtraces

**Compilation errors with features**
→ Run: `cargo check --no-default-features`
→ Run: `cargo check --all-features`

**CLI argument parsing fails**
→ Check `acorn-cli/src/cli/arguments.rs` for ValueEnum definitions
→ Verify clap derive attributes (e.g., `#[derive(Parser)]`)

**Export output malformed**
→ Check `acorn-cli/src/template/mod.rs` for HTML generation
→ Verify assets exist in `acorn-cli/assets/` (fonts, images, skills, styles)
→ Compare with working examples in `tests/fixtures/`

## Performance Considerations

- **Large file batches**: Use `rayon::par_iter()` for parallel processing
- **Memory usage**: Stream file operations where possible instead of loading entire files
- **Network I/O**: Check if `offline` flag is set before making requests
- **Regex patterns**: Use `fancy_regex` for complex patterns; cache compiled patterns

## File Organization Reference

### CLI Structure (acorn-cli/)
- Commands: `src/commands/{command}/mod.rs`
- CLI definitions: `src/cli/mod.rs` and `src/cli/arguments.rs`  
- Main entry: `src/main.rs`
- Templates: `src/template/mod.rs`
- Config: `src/config/mod.rs`
- I/O utilities: `src/io/mod.rs`

### Library Structure (acorn-lib/)
- Schemas: `src/schema/` (research_activity, pid, graph, validate, prompt)
- Analysis: `src/analyzer/` (mod.rs, vale.rs, readability/)
- Utilities: `src/util/` (mod.rs, constants.rs)
- I/O: `src/io/` (mod.rs, bagit.rs, raid.rs, citeas.rs)
- PowerPoint: `src/powerpoint/`
- Doctor: `src/doctor/mod.rs`
- Library exports: `src/lib.rs`, `src/prelude.rs`

### Testing
- Unit tests: `#[cfg(test)]` modules in respective files
- Test fixtures: `tests/fixtures/`
- Dev dependencies: `mockall`, `pretty_assertions`, `similar-asserts`

## Documentation Standards

- All public modules, functions, types need `///` doc comments (enforced by `missing-docs = "deny"`)
- Include examples in doc comments for complex public APIs
- Use proper rustdoc syntax with sections like `# Examples`, `# Errors`, `# Panics`
- No broken intra-doc links (enforced by `broken-intra-doc-links = "forbid"`)
- Update README.md if adding new user-facing features
- Update ARCHITECTURE.md for structural changes
- Update this file (AGENTS.md) if new patterns emerge

### Generating Documentation
```bash
make doc
# OR
cargo doc --no-deps --all-features --open
```

### Book Documentation
ACORN uses mdbook for comprehensive documentation:
```bash
make book        # Build the book
make book-write  # Serve and auto-rebuild on changes
```

## OKF Planning Wiki

The authoritative entry point for refactor, fix, migration, and implementation plans is **[.wiki/index.md](.wiki/index.md)**. The `.wiki/` directory is an Open Knowledge Format bundle with one plan concept per atomic topic.

Before implementing planned work, read the relevant concept and verify its evidence and status against the current code. New plans must use the Plan metadata and authoring rules in the wiki index, including difficulty, impact, and a non-empty package list. When work changes or finishes, update the concept's status, evidence, validation, and acceptance criteria together, then move its entry to the appropriate group in the nearest index.

## Repository Information

- **Repository**: https://code.ornl.gov/research-enablement/acorn
- **Documentation**: https://acorn.ornl.gov
- **License**: MIT
- **Version**: Defined in `[workspace.package].version` in `Cargo.toml`
- **Rust Edition**: 2021
- **Release Profile**: Optimized for size (`opt-level = "s"`) with LTO enabled
