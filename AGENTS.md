# Guide for AI Agents Working on ACORN

This document provides guidance for AI coding agents to work effectively on the ACORN project.

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
   - Test with `-vvvv` verbose flag

4. **Does it require external data or network?**
   - YES → Respect `offline` flag (passed through most command functions)
   - Use `acorn-lib/src/io/` utilities for file/network operations
   - Check `tests/fixtures/` for test data

## Before Making Changes

### Always Do First
```bash
make lint    # Format and check lints
make test    # Run all tests with cargo-nextest
```

### Understand the Scope
- Search existing code: Does similar functionality exist? (use `grep_search`)
- Check tests: What test coverage exists for the module you're changing?
- Validate assumptions: Read at least 20-30 lines of context around your change

## Common Tasks

### Adding a Schema Field
1. Edit struct in `acorn-lib/src/schema/research_activity/mod.rs` (or relevant schema file)
2. Add validation attributes:
   ```rust
   #[validate(length(min = 1, max = 100))]
   #[serde(deserialize_with = "string_trim")]
   pub new_field: String,
   ```
3. Add test case in `#[cfg(test)]` module
4. Update any related command that processes the field

### Adding a CLI Command
1. Create `acorn-cli/src/commands/{command_name}/mod.rs`
2. Add `pub fn run(...)` that takes parsed arguments
3. Return `color_eyre::eyre::Result<(), eyre::Report>`
4. Add command case in `acorn-cli/src/main.rs`
5. Add argument struct in `acorn-cli/src/cli/mod.rs` (or arguments.rs)

### Adding Analysis/Checking Logic
1. Add module in `acorn-lib/src/analyzer/`
2. Integrate result into `Check` struct (see `acorn-lib/src/analyzer/mod.rs`)
3. Call from `acorn-cli/src/commands/check/mod.rs`
4. Surface results using `Label::*()` for output

### Fixing Export/PowerPoint Issues
1. Check `acorn-lib/src/powerpoint/mod.rs` for OOXML manipulation
2. Check `acorn-cli/src/template/mod.rs` for HTML template generation
3. Test both PDF and PPTX targets from `acorn-cli/src/cli/arguments.rs`
4. Verify embedded assets are included (fonts, images in `acorn-cli/assets/`)

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
# All tests
cargo nextest run --run-ignored all

# Specific test module
cargo nextest run analyzer --

# With coverage report
cargo tarpaulin --out lcov --line --skip-clean --no-dead-code --engine llvm
```

### Test Fixtures
Located in `tests/fixtures/`:
- `buckets.json`: Repository configuration examples
- `raid/`: RAiD metadata test data
- `glob/`: Pattern matching test files

## Code Review Checklist (for agents completing work)

Before submitting changes, verify:

- [ ] No `unsafe` keyword appears (workspace forbids it)
- [ ] All public items have doc comments (`///`)
- [ ] No new warnings from `cargo clippy --no-deps`
- [ ] Code formatted: `cargo fmt` run successfully
- [ ] All tests pass: `cargo nextest run --run-ignored all`
- [ ] No intra-doc link errors (workspace forbids broken-intra-doc-links)
- [ ] Serde: Used `#[serde(deny_unknown_fields)]` for new structs
- [ ] Validation: Added appropriate `#[validate(...)]` attributes
- [ ] CLI output: Uses `Label::*()` for consistent styling
- [ ] No new lints: `cargo clippy --workspace --no-deps --all-targets`

## Handling Dependencies & Features

### Feature Flags
The crate uses feature flags for optional functionality:
- `std`: Standard library support (default in CLI, optional in lib)
- `analyzer`: Text analysis and readability metrics
- `doctor`: System diagnostics
- `powerpoint`: PowerPoint export functionality

When adding code that uses `std`:
```rust
#[cfg(feature = "std")]
use std::fs::File;
```

When adding code that uses analyzer features:
```rust
#[cfg(feature = "analyzer")]
use crate::analyzer::readability;
```

### Updating Dependencies
- Check `Cargo.toml` workspace.dependencies for shared versions
- Run `make audit` to check for security vulnerabilities
- Update `deny.toml` if adding new dependencies with license concerns

## Parallel Processing

ACORN uses rayon for parallel iteration over files/batches:

```rust
use rayon::prelude::*;

let results: Vec<_> = items
    .par_iter()
    .map(|item| process(item))
    .collect();
```

The `--threads` CLI argument (default: num CPU cores) controls the thread pool size.

## Error Handling Pattern

Always use `color-eyre` for rich error reporting:

```rust
use color_eyre::eyre::{self, eyre};

fn my_function() -> eyre::Result<String> {
    let file = std::fs::read_to_string("path")
        .wrap_err("Failed to read configuration file")?;
    
    if file.is_empty() {
        return Err(eyre!("Configuration file cannot be empty"));
    }
    
    Ok(file)
}
```

## Logging

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error, instrument};

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

Verbosity levels controlled by: `-v` (debug), `-vv` (trace), or `ACORN_LOG_LEVEL` env var.

## When to Ask for Clarification

Stop and ask the user if:

1. **Ambiguous requirements**: "Should this field be validated as email or just non-empty?"
2. **Architecture conflicts**: "Adding this feature would require restructuring command flow—should I refactor?"
3. **Performance trade-offs**: "Caching this would use more memory—acceptable?"
4. **Breaking changes**: "This change modifies the schema structure—is backward compatibility needed?"
5. **Missing test data**: "No test fixture exists for this scenario—should I create one?"

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
→ Verify assets exist in `acorn-cli/assets/`
→ Compare with working examples in `tests/fixtures/`

## Performance Considerations

- **Large file batches**: Use `rayon::par_iter()` for parallel processing
- **Memory usage**: Stream file operations where possible instead of loading entire files
- **Network I/O**: Check if `offline` flag is set before making requests
- **Regex patterns**: Use `fancy_regex` for complex patterns; cache compiled patterns

## File Organization Reference

- New CLI commands → `acorn-cli/src/commands/{command}/mod.rs`
- Schema additions → `acorn-lib/src/schema/research_activity/mod.rs`
- Validation logic → `acorn-lib/src/schema/validate.rs`
- Analysis features → `acorn-lib/src/analyzer/{feature}/mod.rs`
- Utilities & helpers → `acorn-lib/src/util/mod.rs`
- I/O operations → `acorn-lib/src/io/mod.rs`
- Tests → `#[cfg(test)]` modules in respective files
- Test fixtures → `tests/fixtures/`

## Documentation Standards

- All public modules, functions, types need `///` doc comments
- Include examples for complex public APIs
- Update README.md if adding new user-facing features
- Update ARCHITECTURE.md for structural changes
- Update WARP.md if changes affect deployment or release workflows
- Update this file if new patterns emerge
