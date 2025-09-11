# Architecture
> This file describes the high-level architecture and some specific design decisions of ACORN

## Workspace Structure
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
│   │   ├── powerpoint/ # PowerPoint import/export functionality
│   │   ├── schema/     # Data schemas and validation
│   │   ├── util/       # Common utilities and CLI helpers
│   │   └── lib.rs      # Library entry point
│   └── Cargo.toml
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
- Website accessibility checking using `lychee-lib`

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