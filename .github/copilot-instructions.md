# ACORN Copilot Instructions

**ACORN** (Accessible Content Optimization for Research Needs) is a Rust-based multi-tool for managing research activity data (RAD), validating against schemas, and generating formatted outputs (PDFs, PowerPoint, etc.).

## Project Structure

- **`acorn-lib/`**: Core library with schemas, validation, analysis, and export logic
- **`acorn-cli/`**: CLI application with 6 main commands
- **`acorn-py/`**: Python bindings using maturin (for PyO3)
- **`docs/`**: mdBook documentation

### Key Modules

**Schema System** (`acorn-lib/src/schema/`)
- `ResearchActivity`: Main RAD struct with metadata, sections, and contact info
- `Metadata` (RAiD): Research Activity Identifier metadata
- Custom validators via `serde_trim` and `validator` crate
- JSON Schema generation via `schemars`

**Commands** (`acorn-cli/src/commands/`)
- `check`: Static analysis, readability, prose validation
- `doctor`: System diagnostics  
- `download`: Fetch data from repositories
- `export`: Generate PDFs/PowerPoint (fact-sheet, highlight, poster)
- `format`: In-place JSON/YAML formatting
- `link`: Add linked-data context

**Analysis Pipeline** (`acorn-lib/src/analyzer/`)
- Vale integration for prose linting
- Readability metrics (FKGL, ARI, CLI, FRES, GFI, Lix, SMOG)
- Website accessibility checks via `lychee-lib`

**Export** (`acorn-lib/src/powerpoint/` + `acorn-cli/src/template/`)
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
./setup.bat  # Configures OpenSSL

# All platforms
cargo install cargo-nextest cargo-tarpaulin cargo-audit cargo-auditable
```

### Common Commands
| Task | Command |
|------|---------|
| Build | `make build` |
| Test | `make test` (or `cargo nextest run --run-ignored all`) |
| Coverage | `make coverage` |
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
- Custom validators: `validate_attribute_doi()`, `validate_attribute_ror_list()`
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

**Python Bindings** (`acorn-py/`)
- Built with `maturin` (PyO3 wrapper)
- Compiled to wheels in `target/wheels/`
- Tests in `tests/`: `test_pid.py`, `test_schema.py`, `test_validate.py`

## When Adding Features

1. **New command**: Add module in `acorn-cli/src/commands/{name}/mod.rs`, add case to main.rs
2. **New schema field**: Update `ResearchActivity` or relevant struct, add validators, update tests
3. **New analysis**: Extend `analyzer/` module, integrate in `check` command
4. **Export template**: Add HTML in `acorn-cli/src/template/`, register in export pipeline
5. **Documentation**: Add rustdoc comments (`///`) to public items; forbidden lints will enforce this

## Documentation & Resources

- **Architecture**: [ARCHITECTURE.md](../ARCHITECTURE.md)
- **Contributing**: [CONTRIBUTING.md](../CONTRIBUTING.md)
- **WARP Guide**: [WARP.md](../WARP.md) (deployment and release processes)
- **Book**: https://acorn.ornl.gov (built from `docs/src/` with mdBook)
- **API Docs**: `make doc` generates via `cargo doc --no-deps`
