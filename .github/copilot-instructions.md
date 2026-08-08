# ACORN Copilot Instructions

**USE [guidelines SKILL](./skills/guidelines/) WHENEVER CODE IS TOUCHED OR CONSIDERED**

**ACORN** (Accessible Content Optimization for Research Needs) is a Rust-based multi-tool for managing research activity data (RAD), validating against schemas, and generating formatted outputs (PDFs, PowerPoint, etc.).

## Project Structure

- **`crates/acorn-lib/`**: Core library with schemas, validation, analysis, and export logic
- **`crates/acorn-cli/`**: CLI application with 8 main commands
- **`crates/acorn-py/`**: Python bindings using maturin (for PyO3)
- **`docs/`**: mdBook documentation

### Key Modules

**Schema System** (`crates/acorn-lib/src/schema/`)
- `ResearchActivity`: Main RAD struct with metadata, sections, and contact info
- `Metadata` (RAiD): Research Activity Identifier metadata
- `pid/`: Persistent Identifier schemas and utilities
- `graph.rs`: Knowledge graph schema and structures
- `prompt.rs`: Prompt template schemas for guided data entry
- `validate.rs`: Validation metadata and utilities
- Custom validators via `serde_trim` and `validator` crate
- JSON Schema generation via `schemars`

**Commands** (`crates/acorn-cli/src/commands/`)
- `check`: Static analysis, readability, prose validation
- `doctor`: System diagnostics  
- `download`: Fetch data from repositories
- `export`: Generate PDFs/PowerPoint (fact-sheet, highlight, poster)
- `format`: In-place JSON/YAML formatting
- `gather`: Query and retrieve metadata from external APIs (ORCID, GitHub, GitLab)
- `link`: Add linked-data context
- `schema`: Output JSON schemas for RAD and RAiD metadata structures

**Analysis Pipeline** (`crates/acorn-lib/src/analyzer/`)
- Vale integration for prose linting
- Readability metrics (FKGL, ARI, CLI, FRES, GFI, Lix, SMOG)
- Website accessibility checks via `lychee-lib`

**Export** (`crates/acorn-lib/src/powerpoint/` + `crates/acorn-cli/src/template/`)
- HTML template generation with embedded fonts/images
- PowerPoint export via OOXML manipulation
- PDF generation with QR codes and branding

## Architecture Patterns

### No Unsafe Code
- Workspace-wide lint: `unsafe_code = "forbid"`
- All security-critical features use safe Rust abstractions

### No-std Support
- `acorn-lib` designed to work with `no_std` (see `src/prelude.rs`)
- Feature flags: `std`, `analyzer`, `doctor`, `powerpoint`
- Parallel processing with `rayon` (enabled by default in CLI)

### Error Handling & Logging
- **Errors**: `color-eyre` for rich, contextual error messages with backtraces
- **Logging**: `tracing` for structured logging with `tracing-subscriber`
- Environment variable: `ACORN_LOG_LEVEL` (trace, debug, info, warn, error)

### Asset Embedding
- Binary includes fonts, images, CSS via `rust-embed`
- Accessed via `Constant::from_asset()` in `util/mod.rs`

### Builder Pattern
- Use `bon` crate with `#[builder]` macro for complex structs
- Example: `ResearchActivity::builder()...build()`

## Development Workflow

### Installation
```bash
# Windows
./scripts/setup.bat  # Configures OpenSSL

# All platforms
cargo install cargo-nextest cargo-tarpaulin cargo-audit cargo-auditable
```

### Common Commands
| Task | Command |
|------|---------|
| Build | `make build` |
| Test | `make test` (or `cargo nextest run --run-ignored all`) |
| Coverage | `make cover` |
| Lint/Format | `make lint` |
| Security audit | `make audit` |
| Documentation | `make doc` |

### Testing Patterns
- Tests live inline with `#[cfg(test)]` modules
- Use `#[test]` functions; no external test frameworks
- `cargo nextest` for fast test execution
- `cargo tarpaulin` for coverage (LLVM engine)

## Code Patterns & Conventions

### Struct Validation
- Use `#[validate]` from `validator` crate
- Nested validation via `#[validate(nested)]`
- String trimming: `#[serde(deserialize_with = "string_trim")]`

### Clap CLI Arguments
- Derive-based argument parsing via `#[derive(Parser)]`
- Enums with `#[derive(ValueEnum)]` for subcommands
- Help text via doc comments: `/// Help description`

### Label Formatting (CLI Output)
- Use `Label` enum from `util/mod.rs`: `Label::valid()`, `Label::invalid()`, `Label::run()`, etc.
- Colored output via `owo_colors` (e.g., `.cyan().bold()`)

### Serde Customization
- `#[serde(deny_unknown_fields)]` to catch misconfigurations
- `#[serde(rename_all = "camelCase")]` for JSON schemas
- `#[skip_serializing_none]` to omit null fields in output

### Parallel Processing
- `rayon::ThreadPoolBuilder` (set via CLI `--threads` argument)
- Parallel iteration: `.par_iter()`, `.par_bridge()`

## Key Integration Points

**Workspace Configuration** (`Cargo.toml`)
- Workspace members: `acorn-cli`, `acorn-lib` (acorn-py excluded)
- Shared dependencies: `serde`, `serde_json`, `serde_yml`, `tracing`, `rayon`, `color-eyre`
- Lints: Missing docs forbidden, unsafe code forbidden, intra-doc links forbidden

**Configuration Files**
- `.env`: Local development settings (LOG_LEVEL, etc.)
- `buckets.json`/`buckets.yaml`: Repository configuration for download command
- `.vale/`: Vale prose linting config (git-ignored)

**Asset Embedding**
- CLI assets: `crates/acorn-cli/assets/` (fonts, images, styles)
- Library assets: `crates/acorn-lib/assets/constants/` (constants and configuration)
- All assets are embedded in the binary via `rust-embed` crate

**Python Bindings** (`crates/acorn-py/`)
- Built with `maturin` (PyO3 wrapper)
- Compiled to wheels in `target/wheels/`
- Tests in `crates/acorn-py/tests/`: `test_pid.py`, `test_schema.py`, `test_validate.py`

## When Adding Features

1. **New command**: Add module in `crates/acorn-cli/src/commands/{name}/mod.rs`, add case to main.rs
2. **New schema field**: Update `ResearchActivity` or relevant struct, add validators, update tests
3. **New analysis**: Extend `analyzer/` module, integrate in `check` command
4. **Export template**: Add HTML in `crates/acorn-cli/src/template/`, register in export pipeline
5. **New PID schema**: Add to `crates/acorn-lib/src/schema/pid/` for persistent identifier support
6. **New prompt template**: Add to `crates/acorn-lib/src/schema/prompt.rs` for guided data entry
7. **Documentation**: Add rustdoc comments (`///`) to public items; forbidden lints will enforce this

## Documentation & Resources

- **Architecture**: [ARCHITECTURE.md](../ARCHITECTURE.md)
- **Contributing**: [CONTRIBUTING.md](../CONTRIBUTING.md)
- **WARP Guide**: [WARP.md](../WARP.md) (deployment and release processes)
- **Book**: https://acorn.ornl.gov (built from `docs/src/` with mdBook)
- **API Docs**: `make doc` generates via `cargo doc --no-deps`

## Code Style
Errors should name variable `why`
```rust
match result {
    Ok(value) => value,
    Err(why) => {
        Err(("Failed to process data: {why:?}").into());
    }
}
```
Prefer shorthand syntax for generic types when possible:
```rust
pub fn do_something(value: impl Into<String>) {
    // code code code
}
```

<!-- mermaid-ai-skills:start -->
## Mermaid Diagrams

When the user asks to create, edit, or visualize a diagram, follow the
instructions in `.github/instructions/mermaid.instructions.md`.
<!-- mermaid-ai-skills:end -->
