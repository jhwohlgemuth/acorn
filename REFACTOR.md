# Command Refactor Items: Descriptions and Examples

This document captures the five prioritized refactor items discussed for the command layer, with concrete Rust examples.

## 1. Read-Transform Pipeline

### Description
Create one reusable pipeline that handles reading a ResearchActivity from disk, applying a transformation function, serializing with the selected MIME type, and writing back to disk.

This removes repeated read/serialize/write logic from format and link command handlers.

### Example
```rust
use acorn::schema::ResearchActivity;
use acorn::util::MimeType;
use color_eyre::eyre::{self, eyre};
use std::path::Path;

/// Read a file, transform it, and write it back
pub fn process_activity_file(
    path: &Path,
    mime: &MimeType,
    transform: impl FnOnce(ResearchActivity) -> eyre::Result<ResearchActivity>,
) -> eyre::Result<()> {
    let data = match ResearchActivity::from_path(path) {
        Ok(data) => data,
        Err(why) => return Err(eyre!("Failed to read activity: {why:?}")),
    };
    let result = match transform(data) {
        Ok(result) => result,
        Err(why) => return Err(eyre!("Failed to transform activity: {why:?}")),
    };
    let content = match result.serialize_with(mime) {
        Ok(content) => content,
        Err(why) => return Err(eyre!("Failed to serialize activity: {why:?}")),
    };
    match std::fs::write(path, content) {
        Ok(()) => Ok(()),
        Err(why) => Err(eyre!("Failed to write file: {why:?}")),
    }
}
```

Usage in format:
```rust
process_activity_file(&path, &mime, |data| Ok(data))?;
```

Usage in link:
```rust
process_activity_file(&path, &mime, |data| {
    let formatted = data.with_linked_data();
    Ok(formatted)
})?;
```

## 2. Export Strategy

### Description
Split export format behavior into separate strategy implementations. Instead of a large format switch in a single function, each export format owns its own implementation.

This keeps format-specific code isolated and makes new export formats easier to add.

### Example
```rust
use std::path::Path;
use color_eyre::eyre;

pub trait ExportHandler {
    fn export(&self, data: &ResearchActivity, output: &Path) -> eyre::Result<()>;
}

pub struct PdfExporter {
    pub options: CommandLineOptions,
}

impl ExportHandler for PdfExporter {
    fn export(&self, data: &ResearchActivity, output: &Path) -> eyre::Result<()> {
        let html = render_template(data, &self.options);
        generate_pdf(&html, output)
    }
}

pub struct PowerPointExporter {
    pub options: CommandLineOptions,
}

impl ExportHandler for PowerPointExporter {
    fn export(&self, data: &ResearchActivity, output: &Path) -> eyre::Result<()> {
        create_presentation(data, output, &self.options)
    }
}

pub fn make_exporter(format: &ExportFormat, options: CommandLineOptions) -> Box<dyn ExportHandler> {
    match format {
        ExportFormat::Pdf => Box::new(PdfExporter { options }),
        ExportFormat::Powerpoint => Box::new(PowerPointExporter { options }),
    }
}
```

## 4. Download Decomposition

### Description
Break the download command into focused helper functions so each step does one thing: transfer a single bucket, collect status, and report summary.

This reduces branching and improves testability.

### Example
```rust
use std::path::Path;
use color_eyre::eyre::{self, eyre};

/// Transfer a single bucket to a destination
pub fn transfer_bucket(
    bucket: &Bucket,
    destination: &Path,
    offline: bool,
    verbose: &Verbosity,
) -> eyre::Result<()> {
    let target = destination.join(&bucket.name);
    match std::fs::create_dir_all(&target) {
        Ok(()) => {}
        Err(why) => return Err(eyre!("Failed to create directory: {why:?}")),
    };
    info!("{} Downloading {} to {}", Label::run(), bucket.name, target.display());
    match bucket.download(&target, offline) {
        Ok(()) => {
            info!("{} {} downloaded", Label::pass(), bucket.name);
            Ok(())
        }
        Err(why) => Err(eyre!("Failed to download {}: {why:?}", bucket.name)),
    }
}

/// Log summary of download activity
pub fn log_download_activity(results: &[(&str, bool)]) {
    let passed = results.iter().filter(|(_, ok)| *ok).count();
    let failed = results.len() - passed;
    info!("{} {} succeeded, {} failed", Label::done(), passed, failed);
}
```

## 9. Reporting Facade

### Description
Introduce a small report object that accumulates pass/fail entries and prints them consistently.

This centralizes output formatting and avoids repeated output code in each command.

### Example
```rust
use acorn::util::Label;
use owo_colors::OwoColorize;
use tracing::info;

pub struct Report {
    entries: Vec<ReportEntry>,
}

pub struct ReportEntry {
    label: String,
    passed: bool,
    detail: Option<String>,
}

impl Report {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn pass(mut self, label: impl Into<String>) -> Self {
        self.entries.push(ReportEntry {
            label: label.into(),
            passed: true,
            detail: None,
        });
        self
    }

    pub fn fail(mut self, label: impl Into<String>, detail: impl Into<String>) -> Self {
        self.entries.push(ReportEntry {
            label: label.into(),
            passed: false,
            detail: Some(detail.into()),
        });
        self
    }

    pub fn print(&self) {
        for entry in &self.entries {
            match entry.passed {
                true => info!("{} {}", Label::pass(), entry.label.cyan()),
                false => {
                    let detail = entry.detail.as_deref().unwrap_or("unknown");
                    info!("{} {} — {}", Label::fail(), entry.label.cyan(), detail);
                }
            }
        }
    }

    pub fn has_failures(&self) -> bool {
        self.entries.iter().any(|e| !e.passed)
    }
}
```

## 14. Execution Template

### Description
Use one execution template that standardizes command flow: preflight, setup, run, and mapped exit behavior.

This reduces repetitive run function scaffolding and keeps command behavior consistent.

### Example
```rust
use color_eyre::eyre;
use std::process::exit;

/// Common structure for command execution
pub fn execute_command<T>(
    name: &str,
    offline: bool,
    quiet: bool,
    supports_offline: bool,
    setup: impl FnOnce() -> eyre::Result<T>,
    run: impl FnOnce(T) -> eyre::Result<()>,
) {
    preflight(offline, quiet, supports_offline, name);
    let context = match setup() {
        Ok(ctx) => ctx,
        Err(why) => {
            eprintln!("{} Setup failed: {why:?}", Label::fail());
            exit(exitcode::DATAERR);
        }
    };
    match run(context) {
        Ok(()) => {}
        Err(why) => {
            eprintln!("{} {name} failed: {why:?}", Label::fail());
            exit(exitcode::SOFTWARE);
        }
    }
}
```

Usage:
```rust
pub fn run(path: PathBuf, offline: bool, verbose: Verbosity) {
    execute_command(
        "format",
        offline,
        verbose.is_silent(),
        true,
        || {
            let mime = MimeType::from_path(&path);
            let data = ResearchActivity::from_path(&path)?;
            Ok((mime, data))
        },
        |(mime, data)| {
            let content = data.serialize_with(&mime)?;
            std::fs::write(&path, content)?;
            Ok(())
        },
    );
}
```

---

# Arc Optimization Opportunities

This document lists locations in the ACORN codebase where `clone()` calls could be replaced with `Arc` for better performance and memory management, organized by impact level.

> **Note:** High Impact #1 (Bucket/BucketOptions in `acorn-lib/src/io/config.rs`) has already been implemented.

---

## HIGH IMPACT

These are the most impactful changes: large data structures being cloned in parallel contexts, or structs cloned many times across the codebase.

---

### 2. ResearchActivity in parallel schema validation

**File:** `acorn-lib/src/analyzer/mod.rs`
**Lines:** 129, 265

**What is cloned:**
- Line 129: `data.clone()` -- `Cff` struct passed to `collect_validation_checks`
- Line 265: `data.clone()` -- `ResearchActivity` struct passed to `collect_validation_checks`

Both occur inside `.par_iter().flat_map()` closures. `ResearchActivity` is a very large struct containing nested `ResearchActivityMetadata`, `Sections`, `ContactPoint`, `AspectFramework`, and `Other` -- all with many `String`, `Vec<String>`, and nested struct fields.

**Why Arc is better:** These are parallel iterations over file paths. Each path is read, deserialized into a potentially large `ResearchActivity` or `Cff`, then immediately cloned for validation. Using `Arc<ResearchActivity>` would allow the validation checks to share ownership without deep-copying the entire struct. The `collect_validation_checks` function only needs read access.

**Estimated impact:** **HIGH** -- `ResearchActivity` is the central data model; cloning it in parallel processing is one of the most expensive operations in the codebase.

**Suggested approach:**
```rust
// Before
.par_iter().flat_map(|path| {
    let data = ResearchActivity::read(path)?;
    collect_validation_checks(data.clone())
})

// After
.par_iter().flat_map(|path| {
    let data = Arc::new(ResearchActivity::read(path)?);
    collect_validation_checks(Arc::clone(&data))
})
```

---

### 3. ResearchActivity triple-cloned in website checks

**File:** `acorn-lib/src/analyzer/mod.rs`
**Lines:** 284, 292, 300

**What is cloned:**
- Line 284: `data.clone().meta.doi`
- Line 292: `data.clone().meta.websites`
- Line 300: `data.clone().contact.url`

Three separate clones of the full `ResearchActivity` struct within the same async closure, each to access a single nested field.

**Why Arc is better:** Three clones of a large struct to access individual fields. With `Arc<ResearchActivity>`, you'd clone only the `Arc` pointer (16 bytes on 64-bit) instead of the entire struct. Or simply borrow the fields directly since they're only read.

**Estimated impact:** **HIGH** -- Three unnecessary full struct clones in a loop over paths.

**Suggested approach:**
```rust
// Before
async move {
    let doi = data.clone().meta.doi;
    let websites = data.clone().meta.websites;
    let url = data.clone().contact.url;
    // ...
}

// After (Option 1: Arc)
async move {
    let doi = data.meta.doi.clone();
    let websites = data.meta.websites.clone();
    let url = data.contact.url.clone();
    // ...
}

// After (Option 2: References if lifetimes allow)
// Borrow fields directly without cloning the parent struct
```

---

### 4. PowerPoint export redundant clones

**File:** `acorn-cli/src/commands/export/powerpoint.rs`
**Lines:** 104-179

**What is cloned:**
- Line 104: `options.clone().unwrap()` -- `CommandOptions` (contains 3x `Option<PathBuf>`, 3x `Option<String>`)
- Line 106: `paths.clone()` -- `Vec<PathBuf>` cloned just for `.len()`
- Line 108: `paths.clone()` -- same `Vec` cloned again for `.iter()`
- Line 110: `path.clone()` -- per-item `PathBuf` clone
- Line 112: `path.clone()` -- same path cloned again for context
- Line 125: `path.clone()` -- options field cloned
- Line 145: `reference_path.clone()`
- Line 155: `reference_extract_path.clone()` -- per slide in iteration
- Line 156: `paths.clone()` -- same `Vec` cloned a third time
- Line 170: `data.clone()` -- `ResearchActivity` cloned per item in parallel iteration
- Line 174: `paths[0].clone()` -- yet another clone
- Line 179: `reference_extract_path.clone()` -- another clone

**Why Arc is better:** `paths` is cloned 4 times in this function. `CommandOptions` is cloned for destructuring. `ResearchActivity` is cloned inside a `par_iter` block. Using `Arc<PathBuf>` for paths and `&ResearchActivity` (or `Arc<ResearchActivity>`) would eliminate most of these. The `reference_extract_path` is cloned once per slide -- an `Arc<PathBuf>` would reduce this to a pointer copy.

**Estimated impact:** **HIGH** -- Many redundant clones in the same function, plus `ResearchActivity` clone in parallel context.

**Suggested approach:**
```rust
// Wrap paths in Arc once
let paths = Arc::new(paths);

// Use Arc::clone instead of .clone() for Vec<PathBuf>
let paths_len = paths.len();  // No clone needed
for path in paths.iter() {    // No clone needed
    // ...
}

// For ResearchActivity in par_iter
let data = Arc::new(data);
paths.par_iter().for_each(|path| {
    let data = Arc::clone(&data);
    // ...
});
```

---

### 5. Vale struct in prose checking

**File:** `acorn-lib/src/analyzer/check.rs`
**Lines:** 629-633

**What is cloned:**
- Line 629: `vale.clone()` -- `Vale` struct cloned (contains `Option<SemanticVersion>`, `Option<PathBuf>`, `Option<ValeConfig>`)
- Line 632: `vale.clone()` -- cloned again inside the `.map()` closure for each path

**Why Arc is better:** `Vale` is cloned once, then cloned again per path in the async iterator. The cloned `Vale` is only used to call `.run()`, which takes `&self`. Wrapping in `Arc<Vale>` would reduce per-path cloning to a pointer copy.

**Estimated impact:** **HIGH** -- `Vale` cloning happens for every file being prose-checked, and prose checking is one of the more expensive operations.

**Suggested approach:**
```rust
// Before
let vale = Vale::new(...);
items.iter().map(|path| {
    let vale = vale.clone();
    async move { vale.run(path).await }
})

// After
let vale = Arc::new(Vale::new(...));
items.iter().map(|path| {
    let vale = Arc::clone(&vale);
    async move { vale.run(path).await }
})
```

---

## MEDIUM IMPACT

Structs that are cloned repeatedly but are smaller, or clones that occur less frequently.

---

### 6. CheckOptions cloned 3x in check command

**File:** `acorn-cli/src/commands/check/mod.rs`
**Lines:** 80, 113, 199

**What is cloned:**
- Line 80: `options.clone()` -- `CheckOptions` in `apply_early_exit_policy` (11 fields including `Vec<String>`)
- Line 113: `options.clone()` -- `CheckOptions` in `handle()`
- Line 199: `options.clone()` -- `CheckOptions` in `resolve_skipped_categories()`

**Why Arc is better:** `CheckOptions` is cloned 3 separate times in the check flow. While individual clones are cheap (primitives + one `Vec<String>`), the cumulative cost across many files adds up. More importantly, `CheckOptions` is passed by reference to many async operations that clone it internally. Sharing via `Arc<CheckOptions>` would allow all functions to share a single allocation.

**Estimated impact:** **MEDIUM-HIGH** -- Moderate struct size, but cloned frequently across the check command's hot path.

---

### 7. CommandOptions in export

**File:** `acorn-cli/src/commands/export/mod.rs`
**Lines:** 141, 165

**What is cloned:**
- Line 141: `CommandOptions::init().path(path.clone()).maybe_output(output.clone()).build()` -- creates new options but clones inputs
- Line 165: `options.clone()` inside async closure for parallel processing

**Why Arc is better:** The options struct is rebuilt and cloned multiple times within the export flow. Using `Arc` for the shared fields would reduce allocations, especially for the parallel processing closure at line 165.

**Estimated impact:** **MEDIUM** -- `CommandOptions` is moderate size (~5 Option fields), but cloned in a parallel context.

---

### 8. BucketOptions in download command

**File:** `acorn-cli/src/commands/download/mod.rs`
**Lines:** 51, 105

**What is cloned:**
- Line 51: `options.clone()` -- `BucketOptions`
- Line 105: Same pattern repeated for URL-based download

**Why Arc is better:** In the loop over buckets, `options.clone()` happens for each bucket. If a config file defines many buckets, this creates repeated allocations. `Arc<BucketOptions>` would share the options struct across all buckets.

**Estimated impact:** **MEDIUM** -- Depends on number of buckets; typically small but can grow.

---

### 9. GitLab Options cloned 4x

**File:** `acorn-cli/src/commands/gather/mod.rs`
**Lines:** 60, 64, 78, 80

**What is cloned:**
- Line 60: `options.clone()` -- `gitlab::Options` passed to `runners()`
- Line 64: `options.clone()` -- cloned again with modified identifier
- Line 78: `options.clone()` -- passed to `create_runner()`
- Line 80: `options.clone()` -- passed to `groups()`

**Why Arc is better:** `gitlab::Options` is cloned 4 times in quick succession. If this struct contains HTTP client state or authentication tokens, the clone cost could be non-trivial. Using `Arc` would share the options across API calls.

**Estimated impact:** **MEDIUM** -- Small number of clones, but potentially expensive struct.

---

### 10. Cff in website checks

**File:** `acorn-lib/src/analyzer/mod.rs`
**Lines:** 144, 174

**What is cloned:**
- Line 144: `path.clone()` -- in async closure for website checks
- Line 174: `value.clone()` -- `Identifier.value` String cloned

**Why Arc is better:** The `Cff` struct contains many `Vec` fields (`identifiers`, `references`) that are cloned when the struct is moved into the async closure. Using `Arc<Cff>` for the website check would avoid deep-cloning these collections.

**Estimated impact:** **MEDIUM** -- `Cff` is smaller than `ResearchActivity` but still contains nested collections.

---

### 11. Bucket in file_paths method

**File:** `acorn-lib/src/io/config.rs`
**Lines:** 397-401

**What is cloned:**
- Line 397: `self.clone()` -- full `Bucket` struct destructured
- Line 398: `name.clone()` -- `Option<String>` cloned from the destructured bucket
- Line 401: `code_repository.clone()` -- cloned from the same bucket

**Why Arc is better:** The entire `Bucket` is cloned just to access `name` and `code_repository`. Using `&self` would eliminate this clone entirely since both fields are only read.

**Estimated impact:** **MEDIUM** -- Single clone but of a struct containing `Repository` enum.

---

### 12. Vale cloned 7x across trait implementations

**File:** `acorn-lib/src/analyzer/mod.rs`
**Lines:** 413, 498, 517, 547, 567, 609, 686

**What is cloned:**
- Line 413: `self.clone().command()` -- `Vale` cloned to call `.command()`
- Line 498: `self.clone().download_checksums()` -- `Vale` cloned for async call
- Line 517: `self.clone().extract()` -- `Vale` cloned
- Line 547: `self.clone().command()` -- cloned again
- Line 567: `self.clone().command()` -- cloned again
- Line 609: `self.clone().command()` -- cloned again
- Line 686: `self.clone().command()` -- cloned again (in `with_system_command`)

**Why Arc is better:** `Vale` is cloned 7 times throughout its trait implementations, mostly just to call `.command()`. The `.command()` method takes `self` by value but only reads fields. Changing the signature to take `&self` would eliminate all these clones.

**Estimated impact:** **MEDIUM** -- Many clones but struct is small; the pattern is the issue (consuming `self` for a read-only operation).

**Suggested approach:**
```rust
// Before
fn command(self) -> Command {
    // ...
}

// After
fn command(&self) -> Command {
    // ...
}
```

---

## LOW IMPACT

Small structs, single clones, or clones of types that are already cheap (e.g., `PathBuf` is ~24 bytes, `String` in trivial contexts).

---

### 13. PathBuf cloned in format command (per file, parallel)

**File:** `acorn-cli/src/commands/format/mod.rs`
**Lines:** 37, 47, 57

**What is cloned:**
- Line 37: `path.clone()` -- `PathBuf` in `par_iter`
- Line 47: `path.clone()` -- same path for context
- Line 57: `path.clone()` -- for display string

**Why Arc is better:** `PathBuf` is small (~24 bytes with inline short string optimization), but cloned 3 times per file in a parallel context. Using a reference or `Arc<PathBuf>` would reduce allocations when processing thousands of files.

**Estimated impact:** **LOW** -- Small type, but frequent in parallel context.

---

### 14. PathBuf cloned in link command (per file, parallel)

**File:** `acorn-cli/src/commands/link/mod.rs`
**Lines:** 29, 39, 49, 55

**What is cloned:**
- Line 29: `path.clone()` -- in `par_iter`
- Line 39: `path.clone()` -- for context
- Line 49: `path.clone()` -- for display
- Line 55: `path.clone()` -- for filepath conversion

**Why Arc is better:** Same pattern as format command. 4 clones of `PathBuf` per file in parallel processing.

**Estimated impact:** **LOW** -- Small type but frequent.

---

### 15. Options cloned in resolve_paths

**File:** `acorn-cli/src/cli/mod.rs`
**Lines:** 96

**What is cloned:**
- Line 96: `ignore.clone()` and `filter.clone().map(regex_inverse)` -- both `Option<String>` cloned

**Why Arc is better:** Minor; these are `Option<String>` clones that could be avoided with references.

**Estimated impact:** **LOW** -- Small strings, single location.

---

### 16. Check struct cloned in render function

**File:** `acorn-cli/src/commands/check/mod.rs`
**Lines:** 171, 176

**What is cloned:**
- Line 171: `issue.clone()` -- `Check` struct cloned to access `.uri`
- Line 176: `issue.clone()` -- `Check` cloned to call `.with_index().report()`

**Why Arc is better:** `Check` struct contains several `Option` fields and an `ErrorKind` enum. Cloned twice per issue in the render function. Since the `Check` is only being read and then transformed, a reference-based approach or `&self` methods would avoid cloning.

**Estimated impact:** **LOW** -- Small struct, limited to render path.

---

### 17. Check struct cloned in filter_by_visibility

**File:** `acorn-cli/src/commands/check/mod.rs`
**Line:** 151

**What is cloned:**
- Line 151: `.cloned().collect()` -- every `Check` in the slice is cloned to create a new `Vec`

**Why Arc is better:** If `Check` were `Arc<Check>`, this would become a cheap pointer copy instead of a full struct clone for each element. Alternatively, using references and changing downstream code to accept `&[Check]` would eliminate the clone entirely.

**Estimated impact:** **LOW** -- One-time clone per check cycle, but of all results.

---

### 18. Database path cloned in main.rs

**File:** `acorn-cli/src/main.rs`
**Line:** 169

**What is cloned:**
- Line 169: `database_path.clone()` -- `Option<PathBuf>`

**Why Arc is better:** Single clone at startup, trivial impact.

**Estimated impact:** **LOW** -- One-time clone at startup.

---

## Summary Table

| Priority | File | Lines | Struct Being Cloned | Context | Impact |
|----------|------|-------|-------------------|---------|--------|
| ~~1~~ | ~~acorn-lib/src/io/config.rs~~ | ~~279-311, 344-383~~ | ~~BucketOptions, Bucket~~ | ~~Parallel file operations~~ | ~~HIGH (DONE)~~ |
| 2 | acorn-lib/src/analyzer/mod.rs | 129, 265 | ResearchActivity, Cff | Parallel schema checks | HIGH |
| 3 | acorn-lib/src/analyzer/mod.rs | 284, 292, 300 | ResearchActivity (3x) | Parallel website checks | HIGH |
| 4 | acorn-cli/src/commands/export/powerpoint.rs | 104-179 | CommandOptions, paths, ResearchActivity | Export pipeline | HIGH |
| 5 | acorn-lib/src/analyzer/check.rs | 629, 632 | Vale | Parallel prose checks | HIGH |
| 6 | acorn-cli/src/commands/check/mod.rs | 80, 113, 199 | CheckOptions (3x) | Check command hot path | MED-HIGH |
| 7 | acorn-cli/src/commands/export/mod.rs | 141, 165 | CommandOptions | Export parallel processing | MEDIUM |
| 8 | acorn-cli/src/commands/download/mod.rs | 51, 105 | BucketOptions | Bucket loop | MEDIUM |
| 9 | acorn-cli/src/commands/gather/mod.rs | 60, 64, 78, 80 | gitlab::Options (4x) | API call chain | MEDIUM |
| 10 | acorn-lib/src/analyzer/mod.rs | 144, 174 | Cff | Website checks | MEDIUM |
| 11 | acorn-lib/src/io/config.rs | 397-401 | Bucket | file_paths method | MEDIUM |
| 12 | acorn-lib/src/analyzer/mod.rs | 413, 498, 517, 547, 567, 609, 686 | Vale (7x) | Trait implementations | MEDIUM |
| 13-18 | Various | Various | PathBuf, Check, Option<String> | Per-file parallel ops | LOW |

---

## Implementation Notes

### When to use Arc vs &self

- **Use `Arc`** when:
  - Data needs to be shared across multiple async closures or threads
  - Data is cloned multiple times in a loop
  - Data is passed through multiple layers of functions

- **Use `&self` or references** when:
  - Data is only read and doesn't need to outlive the current scope
  - Clone is only needed due to method signature taking ownership (change signature instead)
  - Single clone that could be avoided with proper borrowing

### Performance Considerations

- `Arc::clone()` is very cheap (atomic increment, ~nanoseconds)
- Deep cloning large structs can be expensive (microseconds to milliseconds)
- The benefit of `Arc` scales with:
  - Size of the cloned struct
  - Frequency of cloning
  - Number of parallel operations

### Testing Arc Changes

After implementing Arc optimizations:
1. Run `cargo test` to ensure no regressions
2. Run `cargo clippy` to catch any new warnings
3. Consider benchmarking before/after if performance is critical

---

# Hardware Schema Improvements

Remaining suggestions for the hardware data model that were not yet implemented.

## 1. Add DPU Resource Variant

**Status:** Not implemented
**Effort:** Medium

### Description
Add a DPU (Data Processing Unit) variant to the `Resource` enum to cover NVIDIA BlueField, Intel IPU, AMD Pensando — devices increasingly used in cloud/data center research.

### Example
```rust
/// Data processing unit
///
/// A programmable processor optimized for data movement, networking, and
/// infrastructure tasks, offloading them from the CPU
DPU {
    /// DPU architecture or product family
    #[serde(alias = "arch")]
    architecture: Option<Architecture>,
    /// Number of DPUs required (if applicable)
    #[serde(default = "default_count")]
    count: Option<u32>,
    /// On-chip memory in GB (if applicable)
    #[serde(alias = "mem")]
    memory: Option<u32>,
    /// Hardware vendor (e.g., NVIDIA, Intel, AMD)
    vendor: Option<Vendor>,
},
```

### Required changes
1. Add variant to `Resource` enum in `hardware.rs`
2. Add `visit_str` arm: `"DPU" | "dpu" => Ok(Resource::DPU { ... })`
3. Add `visit_map` arm: `| "DPU" => { ... }`
4. Add test coverage in `tests/mod.rs`

---

## 2. Target Architecture Validation

**Status:** Not implemented
**Effort:** High — breaking change

### Description
Currently, `Architecture` mixes ISA-level (X86_64, RiscV), GPU µarch (Ampere, RDNA2), and FPGA families (Cyclone V, Versal) in one enum. Nothing prevents assigning `Architecture::Ampere` to a CPU resource. This would require splitting into separate enums.

### Options

**Option A: Separate enums per resource type**
```rust
pub enum CpuArchitecture { X86_64, AArch64, RiscV, LoongArch, ... }
pub enum GpuArchitecture { Ampere, Hopper, Blackwell, RDNA2, RDNA3, ... }
pub enum FpgaArchitecture { VirtexUltraScalePlus, Zynq, CycloneV, Agilex, Versal, ... }
pub enum AcceleratorArchitecture { Gaudi2, Trainium, Wse2, Lpu, ... }
```

**Option B: Keep unified but add validation**
Use `#[validate(...)]` or a custom `is_valid_for(resource_type)` method.

### Required changes
1. Decide on approach (separation vs validation)
2. Update `Resource` variants to use typed architecture enums
3. Update all construction sites and tests
4. Migration path for existing serialized data

---

## 3. Memory Unit Disambiguation

**Status:** Not implemented
**Effort:** Medium

### Description
The `memory` field uses different units per resource variant:
- CPU/GPU/TPU: GB
- NPU: MB (on-chip SRAM)
- FPGA: KB (block RAM)

This creates risk of 1000× errors.

### Options

**Option A: Suffixed field names**
```rust
CPU { memory_gb: Option<u32>, ... }
NPU { memory_mb: Option<u32>, ... }
FPGA { memory_kb: Option<u32>, ... }
```

**Option B: Unit enum**
```rust
pub enum MemoryAmount { GB(u32), MB(u32), KB(u32) }
```

**Option C: Memory struct**
```rust
pub struct Memory {
    amount: u32,
    unit: MemoryUnit,
}
pub enum MemoryUnit { GB, MB, KB }
```

### Required changes
1. Decide on approach (simplest: suffixed field names)
2. Update `Resource` variant field definitions
3. Update deserializer arms (visit_str, visit_map)
4. Update all test constructions
5. Document unit conventions
