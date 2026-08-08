# Architecture
> This file describes the high-level architecture and some specific design decisions of ACORN

## Workspace Structure
The project keeps ACORN product crates under `crates/` and repository tooling at the workspace root:

```
acorn/
├── crates/
│   ├── acorn-cli/      # CLI application and embedded presentation assets
│   ├── acorn-lib/      # Compatibility library, schemas, analysis, and host services
│   ├── acorn-macros/   # Procedural macros
│   ├── acorn-py/       # Python bindings and package configuration
│   └── acorn-tui/      # Terminal user interface
├── docs/               # Public mdBook documentation
├── fuzz/               # Cargo fuzz targets
├── tests/              # Shared fixtures and integration data
├── xtask/              # Repository maintenance commands
└── Cargo.toml           # Workspace configuration
```

### Core Components

**Schema System (`crates/acorn-lib/src/schema/`)**
- `ResearchActivity`: Main data structure for research activity data (RAD)
- `raid::Metadata`: RAiD (Research Activity Identifier) metadata schema
- Validation logic using Rust's type system and the `validator` crate
- JSON Schema generation via `schemars`

**Command Structure (`crates/acorn-cli/src/commands/`)**
- `check`: Static analysis and validation of research data
- `doctor`: System diagnostics and environment setup
- `download`: Fetch data from remote buckets (GitLab/GitHub repositories)  
- `export`: Generate fact sheets, highlights, and posters from research data
- `format`: Format JSON/YAML files in-place
- `link`: Add linked data context to research activity data
- `schema`: Output JSON schemas for RAD and RAiD

**Analysis Pipeline (`crates/acorn-lib/src/analyzer/`)**
- Vale integration for prose analysis
- Readability metrics (Flesch-Kincaid, etc.)
- Website accessibility checking using `lychee-lib`

**Export System (`crates/acorn-lib/src/powerpoint/` and `crates/acorn-cli/src/template/`)**
- HTML template generation
- PowerPoint export via OOXML manipulation
- PDF generation with embedded QR codes and branding

### Key Design Patterns

**Error Handling**: Uses `color-eyre` for rich error reporting and `tracing` for structured logging.

**Parallel Processing**: Leverages `rayon` for parallel file processing and validation.

**CLI Architecture**: Uses `clap` for argument parsing with derive macros and extensive help documentation.

**Asset Embedding**: Uses `rust-embed` to include fonts, images, agent skill files (`crates/acorn-cli/assets/skills/`), and styles in the binary.

## Configuration and Environment

### Environment Variables
- `ACORN_LOG_LEVEL`: Configure logging level (trace, debug, info, warn, error)
- Standard Rust environment variables (e.g., `RUST_LOG`)

### Configuration Files
- `.env`: Environment configuration for local development
- `buckets.json` or `buckets.yaml`: Repository configuration for download command
- Vale configuration in `.vale` directory (not tracked in git)
