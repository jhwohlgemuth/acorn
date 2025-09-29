# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

ACORN (Accessible Content Optimization for Research Needs) is a Rust-based CLI tool and library for maintaining and validating research activity data (RAD). The project consists of two main crates:

- `acorn-cli`: Command-line interface providing commands for checking, formatting, exporting, and managing research data
- `acorn-lib`: Core library containing schemas, validation logic, and utilities

## Development Commands

### Build Commands
```bash
# Clean build artifacts
make clean

# Build the project (release build)
make build

# Build with cargo directly
cargo build --release --package acorn-cli
```

### Testing Commands
```bash
# Run tests with nextest (preferred)
make test

# Run tests with cargo (CI)
make test-ci

# Run specific test
cargo nextest run <test_name>

# Run tests for specific package
cargo test --package acorn-lib
cargo test --package acorn-cli
```

### Linting and Formatting
```bash
# Format and lint code
make lint

# Format code only
cargo fmt --verbose

# Run clippy
cargo clippy --workspace --no-deps --all-targets --fix --allow-dirty
```

### Documentation
```bash
# Generate documentation
make doc

# View documentation locally
cargo doc --no-deps --all-features --open
```

### Coverage and Audit
```bash
# Generate test coverage report
make coverage

# Security audit
make audit
```

### Docker Commands
```bash
# Build acorn Docker image
make acorn-image

# Build bucket Docker image  
make bucket-image

# Clean Docker images
make acorn-clean
make bucket-clean
```

### Single Test Execution
```bash
# Run a single test file
cargo nextest run --package acorn-lib analyzer::tests

# Run single test function
cargo nextest run --package acorn-cli commands::check::tests::test_check_command
```

## Architecture

### Workspace Structure
The project uses Cargo workspaces with two main crates:

```
acorn/
├── acorn-cli/          # CLI application
│   ├── src/
│   │   ├── cli/        # Command-line interface definitions
│   │   ├── commands/   # Command implementations (check, export, etc.)
│   │   ├── template/   # Export template generation
│   │   └── main.rs     # Entry point
│   └── Cargo.toml
├── acorn-lib/          # Core library
│   ├── src/
│   │   ├── analyzer/   # Text analysis and readability checks
│   │   ├── doctor/     # System diagnostics
│   │   ├── powerpoint/ # PowerPoint export functionality
│   │   ├── schema/     # Data schemas and validation
│   │   ├── util/       # Common utilities and CLI helpers
│   │   └── lib.rs      # Library entry point
│   └── Cargo.toml
├── acorn-py/           # Python package project
│   ├── src/
│   │   └── lib.rs      # Library entry point
│   ├── Cargo.toml      # Rust configuration
│   └── pyproject.toml  # Python configuration
└── Cargo.toml          # Workspace configuration
```

### Core Components

**Schema System (`acorn-lib/src/schema/`)**
- `ResearchActivity`: Main data structure for research activity data (RAD)
- `raid::Metadata`: RAiD (Research Activity Identifier) metadata schema
- Validation logic using Rust's type system and the `validator` crate
- JSON Schema generation via `schemars`

**Command Structure (`acorn-cli/src/commands/`)**
- `check`: Static analysis and validation of research data
- `doctor`: System diagnostics and environment setup
- `download`: Fetch data from remote buckets (GitLab/GitHub repositories)  
- `export`: Generate fact sheets, highlights, and posters from research data
- `format`: Format JSON/YAML files in-place
- `link`: Add linked data context to research activity data
- `schema`: Output JSON schemas for RAD and RAiD

**Analysis Pipeline (`acorn-lib/src/analyzer/`)**
- Vale integration for prose analysis
- Readability metrics (Flesch-Kincaid, etc.)
- Website accessibility checking via `lychee-lib`

**Export System (`acorn-lib/src/powerpoint/` and `acorn-cli/src/template/`)**
- HTML template generation
- PowerPoint export via OOXML manipulation
- PDF generation with embedded QR codes and branding

### Key Design Patterns

**Error Handling**: Uses `color-eyre` for rich error reporting and `tracing` for structured logging.

**Parallel Processing**: Leverages `rayon` for parallel file processing and validation.

**CLI Architecture**: Uses `clap` for argument parsing with derive macros and extensive help documentation.

**Asset Embedding**: Uses `rust-embed` to include fonts, images, and configuration files in the binary.

## Configuration and Environment

### Environment Variables
- `ACORN_LOG_LEVEL`: Configure logging level (trace, debug, info, warn, error)
- Standard Rust environment variables (e.g., `RUST_LOG`)

### Configuration Files
- `.env`: Environment configuration for local development
- `buckets.json` or `buckets.yaml`: Repository configuration for download command
- Vale configuration in `.vale` directory (not tracked in git)

## Repository Workflows

### Git Integration
Commands support Git-based workflows:
```bash
# Check files changed in specific commit
acorn check --commit HEAD

# Format files in current branch vs main
acorn format --branch main

# Export files from merge request
acorn export --merge-request
```

### CI/CD Pipeline
- GitLab CI with stages: prepare, test, build, docs, release
- Automated testing on Rust stable
- Binary artifact generation for releases
- Docker image building and publishing

## Development Guidelines

### Code Organization
- Workspace-level dependencies in root `Cargo.toml`
- Feature flags: `cli` feature in `acorn-lib` for CLI-specific functionality
- Strict linting: `unsafe_code = "forbid"`, `missing-docs = "deny"`

### Testing Strategy
- Unit tests alongside source code
- Integration tests in `tests/` directories
- Test fixtures in `tests/fixtures/`
- Use `pretty_assertions` for better test output
- Use `mockall` for mocking in tests

### Documentation Standards
- Comprehensive rustdoc comments required
- Examples in documentation
- Links to external schemas and specifications
- Architecture decision documentation in `ARCHITECTURE.md` (currently under construction)

### Binary Distribution
- Cross-compilation support for multiple targets
- Release binaries available via GitLab package registry
- Docker images published to multiple registries
- Cargo package published to crates.io

## Common Development Tasks

### Adding a New Command
1. Add command variant to `acorn-cli/src/cli/mod.rs`
2. Create implementation in `acorn-cli/src/commands/`
3. Wire up in `acorn-cli/src/main.rs`
4. Add tests and documentation

### Adding Schema Validation
1. Define schema in `acorn-lib/src/schema/`
2. Implement validation logic using `validator` crate
3. Add fuzzy matching for controlled vocabularies
4. Update constants in `assets/constants/`

### Working with External APIs
- GitHub and GitLab API integration for bucket downloads
- CiteAs API for DOI citation generation
- Structured error handling for network operations
