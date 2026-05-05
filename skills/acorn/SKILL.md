---
name: acorn
description: 'Guidance for the ACORN CLI tool and research activity data (RAD). Use when: running or writing acorn commands; creating, editing, or validating RAD files (index.json, CITATION.cff, .docx); exporting to PDF, PowerPoint, BagIt, or Markdown; adding JSON-LD linked-data context; downloading RAD from buckets; gathering metadata from ORCID/GitHub/GitLab; checking prose readability or schema compliance; installing acorn via Cargo, Scoop, container, or binary; understanding RAD/RAiD schema structure; setting up CI pipelines for research content; or working with research project metadata, sponsors, keywords, sections, or contact information.'
---

# ACORN — Research Activity Data (RAD) Tool

ACORN validates, formats, analyzes, and exports research activity data stored as JSON or YAML files.

## RAD File Structure

A RAD file (typically `index.json`) describes a single research project:

```json
{
  "meta": {
    "id": "my-project",
    "status": "active",
    "draft": false,
    "archive": false,
    "keywords": ["automation", "data"],
    "sponsors": ["DOE"],
    "websites": [{ "title": "Repo", "url": "https://..." }]
  },
  "title": "My Project",
  "subtitle": "A short descriptor",
  "sections": {
    "mission": "...",
    "challenge": "...",
    "approach": ["..."],
    "impact": ["..."]
  },
  "contact": {
    "title": "PI",
    "first": "Jane",
    "last": "Doe",
    "email": "doe@example.gov"
  }
}
```

Inspect the full schema at any time:
```shell
acorn schema rad
acorn schema raid   # RAiD metadata schema
```

## Installation

### Cargo (all platforms)
Requires [Rust/Cargo](https://rustup.rs/):
```shell
cargo install acorn-cli
acorn help
```

### Scoop (Windows)
```shell
scoop bucket add extras
scoop install acorn
```

### Pre-compiled binary

**Windows (PowerShell):**
```powershell
irm -OutFile acorn.exe -Uri https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-pc-windows-gnu/v0.1.50/acorn.exe
.\acorn.exe help
```

**Linux:**
```shell
curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-musl/v0.1.50/acorn
chmod +x acorn
./acorn help
```

Move the binary to a directory on your `PATH` for system-wide access.

### Container (Docker / Podman)
```shell
docker pull savannah.ornl.gov/research-enablement/acorn/runner

# Basic use
docker run --rm savannah.ornl.gov/research-enablement/acorn/runner help

# Mount local files
docker run --rm -v $(pwd):/data savannah.ornl.gov/research-enablement/acorn/runner:latest check /data/project
```

### Install from source
```shell
git clone https://code.ornl.gov/research-enablement/acorn.git
cd acorn
cargo install --path ./acorn-cli
acorn help
```

### Verify installation
```shell
acorn --version
acorn help
```

---

## Global Flags

| Flag | Purpose |
|------|---------|
| `-X` / `--offline` | Block all network calls |
| `-t N` / `--threads N` | Limit rayon thread count |
| `-v` / `-vv` / `-vvv` | Increase log verbosity |
| `-q` | Quiet mode |

## Running from Source

Replace `acorn` with `cargo run --` to run the development version without installing:

```shell
cargo run -- check /path/to/file.json
cargo run -- export /path/to/folder --format pdf
cargo run -- doctor --fix
```

Pass global flags before the subcommand (after `--`):

```shell
cargo run -- --offline check ./content --skip prose
cargo run -- -vv check ./content
```

Use `--release` for a faster build when profiling or processing large datasets:

```shell
cargo run --release -- export ./content --format powerpoint
```

---

## Commands

### `acorn check` — Validate & Analyze

Runs schema validation, prose linting (Vale), readability scoring, and website accessibility checks.

```sh
acorn check /path/to/file.json
acorn check /path/to/folder              # all supported files
acorn check --commit HEAD                # files changed in last commit
acorn check --branch main                # files changed vs branch
acorn check --merge-request              # files in open MR/PR
```

**Skip categories** (`--skip`):
| Value | Skips |
|-------|-------|
| `schema` | JSON schema validation |
| `prose` | Vale prose linting |
| `readability` | Readability metric scoring |
| `conventions` | Convention checks |

```shell
acorn check . --skip prose,schema
acorn check . --disable-website-checks   # skip URL reachability
acorn check . --no-fail                  # always exit 0 (debugging)
acorn check . --ignore "[/]draft\.json$" # exclude by path regex
```

**Readability metrics** (`--readability-test`): `fkgl` *(default)*, `ari`, `cli`, `fres`, `gfi`, `lix`, `smog`

```shell
# Score with default metric (FKGL)
acorn check /path/to/file.json

# Score with a specific metric
acorn check /path/to/file.json --readability-test ari

# Run only readability (skip everything else)
acorn check /path/to/file.json --skip conventions,prose,schema
```

**Analyzing CFF files** (`CITATION.cff`): the standard is auto-inferred when the path contains only `.cff` files, or set it explicitly:

```shell
# Auto-inferred from extension
acorn check CITATION.cff

# Explicit standard
acorn check CITATION.cff --standard cff

# Check all CFF files in a folder
acorn check /path/to/folder --standard cff
```

**Analyzing DOCX files**: the standard is auto-inferred when the path contains only `.docx` files, or set it explicitly:

```shell
# Auto-inferred from extension
acorn check report.docx

# Explicit standard
acorn check report.docx --standard docx

# Prose and readability only (schema not applicable to DOCX)
acorn check report.docx --standard docx --skip conventions,schema
```

Other available standards (`--standard`): `rads` *(default)*, `cff`, `docx`, `text`, `datacite`, `dcat`, `dcmi`, `invenio`, `huwise`

---

### `acorn format` — Normalize In-Place

Formats RAD files in-place (whitespace, key ordering, string trimming). Includes a subset of `check` fixes.

```shell
acorn format /path/to/file.json
acorn format /path/to/folder --dry-run   # preview diff without writing
acorn format --commit HEAD
acorn format --branch main
```

---

### `acorn export` — Generate Outputs

Exports RAD to rendered formats. Default output directory: `./export`.

```shell
acorn export /path/to/folder
acorn export /path/to/file.json --format pdf
acorn export /path/to/folder --target highlight --format powerpoint
acorn export /path/to/folder --format bag       # BagIt archive
acorn export --commit HEAD --format markdown
acorn export /path/to/folder --combine          # merge into single artifact
```

**Formats** (`--format`): `pdf` *(default)*, `powerpoint`, `bag`, `json`, `yaml`, `markdown`

**Export targets** (`--target`): `highlight`, `poster`, `fact-sheet`

---

### `acorn link` — Add Linked-Data Context

Injects JSON-LD `@context` into RAD files for semantic web / knowledge graph use.

```shell
acorn link /path/to/folder
acorn link /path/to/folder --dry-run     # preview diff
acorn link --commit HEAD
```

---

### `acorn gather` — Extract Metadata

Queries external APIs (ORCID, GitHub, GitLab) to pre-populate RAD fields.

```shell
acorn gather /path/to/folder
```

---

### `acorn download` — Fetch RAD from Buckets

Downloads RAD files from a Git repository (bucket) or a config file listing multiple buckets.

```shell
acorn download https://github.com/org/repo
acorn download --config /path/to/buckets.json
acorn download --config /path/to/buckets.yml --output ./content
```

Default output directory: `./content`

---

### `acorn doctor` — System Diagnostics

Checks and optionally repairs system requirements (Vale, dependencies, etc.).

```shell
acorn doctor                      # print diagnostics
acorn doctor --fix                # auto-correct issues
acorn doctor --fix --interactive  # TUI for selective fixes
acorn doctor --report             # JSON report for bug reports
acorn doctor --check network,software
```

**Check categories**: `all` *(default)*, `system`, `memory`, `network`, `gpu`, `software`

---

## Common Workflows

### Validate a new RAD file
```shell
acorn check ./content/my-project/index.json
```

### Format then export a folder
```shell
acorn format ./content
acorn export ./content --format pdf
```

### CI pipeline (fail on errors)
```shell
acorn check --commit HEAD --skip readability
```

### Preview format changes before committing
```shell
acorn format ./content --dry-run
```

### Work offline
```shell
acorn --offline check ./content --skip prose
```

### Generate schema for editor integration
```shell
acorn schema rad > rad-schema.json
```

