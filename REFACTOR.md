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

# Arc Optimization

This document lists locations in the ACORN codebase where `clone()` calls could be replaced with `Arc` for better performance and memory management, organized by impact level.

> **Note:** High Impact #1 (Bucket/BucketOptions in `acorn-lib/src/io/config.rs`) has already been implemented.

---

## HIGH IMPACT

These are the most impactful changes: large data structures being cloned in parallel contexts, or structs cloned many times across the codebase.

---

### 1. PowerPoint export redundant clones

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

### 2. Vale struct in prose checking

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

### 3. CheckOptions cloned 3x in check command

**File:** `acorn-cli/src/commands/check/mod.rs`
**Lines:** 80, 113, 199

**What is cloned:**
- Line 80: `options.clone()` -- `CheckOptions` in `apply_early_exit_policy` (11 fields including `Vec<String>`)
- Line 113: `options.clone()` -- `CheckOptions` in `handle()`
- Line 199: `options.clone()` -- `CheckOptions` in `resolve_skipped_categories()`

**Why Arc is better:** `CheckOptions` is cloned 3 separate times in the check flow. While individual clones are cheap (primitives + one `Vec<String>`), the cumulative cost across many files adds up. More importantly, `CheckOptions` is passed by reference to many async operations that clone it internally. Sharing via `Arc<CheckOptions>` would allow all functions to share a single allocation.

**Estimated impact:** **MEDIUM-HIGH** -- Moderate struct size, but cloned frequently across the check command's hot path.

---

### 4. CommandOptions in export

**File:** `acorn-cli/src/commands/export/mod.rs`
**Lines:** 141, 165

**What is cloned:**
- Line 141: `CommandOptions::init().path(path.clone()).maybe_output(output.clone()).build()` -- creates new options but clones inputs
- Line 165: `options.clone()` inside async closure for parallel processing

**Why Arc is better:** The options struct is rebuilt and cloned multiple times within the export flow. Using `Arc` for the shared fields would reduce allocations, especially for the parallel processing closure at line 165.

**Estimated impact:** **MEDIUM** -- `CommandOptions` is moderate size (~5 Option fields), but cloned in a parallel context.

---

### 5. BucketOptions in download command

**File:** `acorn-cli/src/commands/download/mod.rs`
**Lines:** 51, 105

**What is cloned:**
- Line 51: `options.clone()` -- `BucketOptions`
- Line 105: Same pattern repeated for URL-based download

**Why Arc is better:** In the loop over buckets, `options.clone()` happens for each bucket. If a config file defines many buckets, this creates repeated allocations. `Arc<BucketOptions>` would share the options struct across all buckets.

**Estimated impact:** **MEDIUM** -- Depends on number of buckets; typically small but can grow.

---

### 6. GitLab Options cloned 4x

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

### 7. Cff in website checks

**File:** `acorn-lib/src/analyzer/mod.rs`
**Lines:** 144, 174

**What is cloned:**
- Line 144: `path.clone()` -- in async closure for website checks
- Line 174: `value.clone()` -- `Identifier.value` String cloned

**Why Arc is better:** The `Cff` struct contains many `Vec` fields (`identifiers`, `references`) that are cloned when the struct is moved into the async closure. Using `Arc<Cff>` for the website check would avoid deep-cloning these collections.

**Estimated impact:** **MEDIUM** -- `Cff` is smaller than `ResearchActivity` but still contains nested collections.

---

### 8. Bucket in file_paths method

**File:** `acorn-lib/src/io/config.rs`
**Lines:** 397-401

**What is cloned:**
- Line 397: `self.clone()` -- full `Bucket` struct destructured
- Line 398: `name.clone()` -- `Option<String>` cloned from the destructured bucket
- Line 401: `code_repository.clone()` -- cloned from the same bucket

**Why Arc is better:** The entire `Bucket` is cloned just to access `name` and `code_repository`. Using `&self` would eliminate this clone entirely since both fields are only read.

**Estimated impact:** **MEDIUM** -- Single clone but of a struct containing `Repository` enum.

---

### 9. Vale cloned 7x across trait implementations

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

### 10. PathBuf cloned in format command (per file, parallel)

**File:** `acorn-cli/src/commands/format/mod.rs`
**Lines:** 37, 47, 57

**What is cloned:**
- Line 37: `path.clone()` -- `PathBuf` in `par_iter`
- Line 47: `path.clone()` -- same path for context
- Line 57: `path.clone()` -- for display string

**Why Arc is better:** `PathBuf` is small (~24 bytes with inline short string optimization), but cloned 3 times per file in a parallel context. Using a reference or `Arc<PathBuf>` would reduce allocations when processing thousands of files.

**Estimated impact:** **LOW** -- Small type, but frequent in parallel context.

---

### 11. PathBuf cloned in link command (per file, parallel)

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

### 12. Options cloned in resolve_paths

**File:** `acorn-cli/src/cli/mod.rs`
**Lines:** 96

**What is cloned:**
- Line 96: `ignore.clone()` and `filter.clone().map(regex_inverse)` -- both `Option<String>` cloned

**Why Arc is better:** Minor; these are `Option<String>` clones that could be avoided with references.

**Estimated impact:** **LOW** -- Small strings, single location.

---

### 13. Check struct cloned in render function

**File:** `acorn-cli/src/commands/check/mod.rs`
**Lines:** 171, 176

**What is cloned:**
- Line 171: `issue.clone()` -- `Check` struct cloned to access `.uri`
- Line 176: `issue.clone()` -- `Check` cloned to call `.with_index().report()`

**Why Arc is better:** `Check` struct contains several `Option` fields and an `ErrorKind` enum. Cloned twice per issue in the render function. Since the `Check` is only being read and then transformed, a reference-based approach or `&self` methods would avoid cloning.

**Estimated impact:** **LOW** -- Small struct, limited to render path.

---

### 14. Check struct cloned in filter_by_visibility

**File:** `acorn-cli/src/commands/check/mod.rs`
**Line:** 151

**What is cloned:**
- Line 151: `.cloned().collect()` -- every `Check` in the slice is cloned to create a new `Vec`

**Why Arc is better:** If `Check` were `Arc<Check>`, this would become a cheap pointer copy instead of a full struct clone for each element. Alternatively, using references and changing downstream code to accept `&[Check]` would eliminate the clone entirely.

**Estimated impact:** **LOW** -- One-time clone per check cycle, but of all results.

---

### 15. Database path cloned in main.rs

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
| 1 | acorn-cli/src/commands/export/powerpoint.rs | 104-179 | CommandOptions, paths, ResearchActivity | Export pipeline | HIGH |
| 2 | acorn-lib/src/analyzer/check.rs | 629, 632 | Vale | Parallel prose checks | HIGH |
| 3 | acorn-cli/src/commands/check/mod.rs | 80, 113, 199 | CheckOptions (3x) | Check command hot path | MED-HIGH |
| 4 | acorn-cli/src/commands/export/mod.rs | 141, 165 | CommandOptions | Export parallel processing | MEDIUM |
| 5 | acorn-cli/src/commands/download/mod.rs | 51, 105 | BucketOptions | Bucket loop | MEDIUM |
| 6 | acorn-cli/src/commands/gather/mod.rs | 60, 64, 78, 80 | gitlab::Options (4x) | API call chain | MEDIUM |
| 7 | acorn-lib/src/analyzer/mod.rs | 144, 174 | Cff | Website checks | MEDIUM |
| 8 | acorn-lib/src/io/config.rs | 397-401 | Bucket | file_paths method | MEDIUM |
| 9 | acorn-lib/src/analyzer/mod.rs | 413, 498, 517, 547, 567, 609, 686 | Vale (7x) | Trait implementations | MEDIUM |
| 10 | acorn-cli/src/commands/format/mod.rs | 37, 47, 57 | PathBuf | Per-file parallel ops | LOW |
| 11 | acorn-cli/src/commands/link/mod.rs | 29, 39, 49, 55 | PathBuf | Per-file parallel ops | LOW |
| 12 | acorn-cli/src/cli/mod.rs | 96 | Option<String> | resolve_paths options | LOW |
| 13 | acorn-cli/src/commands/check/mod.rs | 171, 176 | Check | Render path | LOW |
| 14 | acorn-cli/src/commands/check/mod.rs | 151 | Check | Visibility filter | LOW |
| 15 | acorn-cli/src/main.rs | 169 | Option<PathBuf> | Startup path handling | LOW |

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

# Rust Safety Boundary Issues (Bugs Rust Won't Catch)

Analysis of the codebase against rules from [Bugs Rust Won't Catch](https://corrode.dev/blog/bugs-rust-wont-catch/) — patterns that the borrow checker and clippy do not enforce but are common sources of CVEs in Rust systems code.

---

### HIGH IMPACT / LOW DIFFICULTY

#### 1. TOCTOU: `File::create` followed by write — potential symlink race

`File::create` follows symlinks. Between `File::create` and `io::copy`, an attacker with write access to a parent directory can swap the target with a symlink to an arbitrary file.

**Sites:**
- `acorn-lib/src/io/mod.rs:269` — zip archive creation (`File::create(&zip_file_path)`)
- `acorn-lib/src/io/mod.rs:455` — zip extraction (`File::create(&target)`)
- `acorn-lib/src/io/mod.rs:1226` — `write_file_bytes`
- `acorn-cli/src/commands/export/pdf.rs:36` — temp index.html for Playwright

**Fix:** Replace `File::create` with `OpenOptions::new().write(true).create_new(true).open(path)` which atomically fails if a symlink exists at the target location.

#### 2. Permissions set after creation — TOCTOU window

- `acorn-lib/src/io/mod.rs:853` — `make_executable` calls `set_permissions(0o755)` after the file already exists with default permissions, leaving a window where any user can open the file with default perms.

**Fix:** Use `OpenOptions::new().mode(0o755)` at creation time on Unix.

#### 3. Discarded `Result` values — silent error swallowing

- `acorn-cli/src/commands/export/powerpoint.rs:49,65,71,105` — `let _ = write_file(...)` silently discards write failures
- `acorn-cli/src/commands/export/powerpoint.rs:182` — `let _ = copy_image(...)` silently discards copy failures
- `acorn-cli/src/commands/download/mod.rs:81,95,131` — `let _ = database.insert(...)` silently discards insert failures
- `acorn-lib/src/io/api/gitlab.rs:725` — `let _ = dotenvy::from_filename(".env")`
- `acorn-lib/src/io/api/raid.rs:165` — `let _ = dotenvy::from_filename(".env")`
- `acorn-cli/src/commands/export/mod.rs:158` — `let _ = powerpoint::create(...)` silently discarded

**Fix:** Replace `let _ =` with `if let Err(e) = ...` and log or return the error. Add a `// safety:` comment if discarding is intentional.

---

### HIGH IMPACT / MEDIUM DIFFICULTY

#### 4. `unwrap()` on path operations — panic on non-UTF-8 paths

Same pattern as CVE-2026-35348 (uutils `sort --files0-from` panicked on non-UTF-8 filenames).

- `acorn-lib/src/analyzer/vale.rs:192` — `path.to_str().unwrap()` panics on non-UTF-8 paths
- `acorn-py/src/lib.rs:374` — `path.to_str().unwrap()`
- `acorn-lib/src/analyzer/mod.rs:878` — `path.parent().unwrap()` panics on root paths
- `acorn-cli/src/commands/export/powerpoint.rs:94` — `source.parent().unwrap().canonicalize().unwrap()`
- `acorn-cli/src/commands/export/pdf.rs:35` — `path.parent().unwrap()`

**Fix:** Use `?` to propagate, `unwrap_or_default()` for fallback, or `path.parent().ok_or_else(|| eyre!(...))`?` for paths that must have a parent.

#### 5. Variable-index slice indexing — panic on out-of-bounds

- `acorn-lib/src/analyzer/mod.rs:809` — `stdout[offset..]` on `Vec<u8>` panics if `offset > stdout.len()`
- `acorn-lib/src/analyzer/check.rs:835` — `source_text[prefix_start..]` panics if `prefix_start > len()`
- `acorn-lib/src/schema/hardware/memory.rs:150-151` — `s[..split]` and `s[split..]` panics on bad split
- `acorn-lib/src/schema/pid/mod.rs:519,651` — `value[..value.len().saturating_sub(1)]` panics on empty

**Fix:** Use `.get(..)` or `.get_mut(..)` which return `Option`, or add explicit bounds checks before indexing.

---

### MEDIUM IMPACT / LOW DIFFICULTY

#### 6. `from_utf8_lossy` at Unix boundaries — silent data corruption

- `acorn-lib/src/util/cmd.rs:55,57` — command stdout/stderr lossy-converted, corrupts non-UTF-8 output
- `acorn-lib/src/prelude.rs:61,64` — `CommandOutput` traits use lossy conversion (widely consumed)
- `acorn-lib/src/io/http/mod.rs:143` — HTTP response body lossy-converted
- `acorn-cli/src/template/mod.rs:101` — embedded asset data lossy-converted
- `acorn-lib/src/schema/prompt/mod.rs:207` — prompt response data lossy-converted

**Fix:** Prefer returning `Vec<u8>` or `OsString` across boundaries, letting callers handle encoding. Avoid lossy UTF-8 round-trips for command output, paths, and env values.

#### 7. `unwrap()` on `Option` — panics on unexpected None

- `acorn-lib/src/analyzer/mod.rs:731` — `self.config.unwrap()` panics if no config set
- `acorn-lib/src/analyzer/mod.rs:803` — `which(name).unwrap()` panics if binary not found
- `acorn-lib/src/analyzer/mod.rs:807` — `cmd!(name, ["--version"]).unwrap()` panics on command failure
- `acorn-lib/src/schema/mod.rs:585,681` — `iter().next().unwrap()` on potentially empty collections
- `acorn-lib/src/lib.rs:252` — same pattern
- `acorn-lib/src/util/mod.rs:858` — `.last().unwrap()` on potentially empty segments
- `acorn-cli/src/commands/export/mod.rs:74,81` — `Option::unwrap()` on CLI options

**Fix:** Use `ok_or_else(|| eyre!(...))`?`, `unwrap_or_default()`, or match with fallback logging.

#### 8. `unwrap()` on I/O operations — crash on transient failures

- `acorn-lib/src/io/mod.rs:853` — `set_permissions(path, 0o755).unwrap()` — dies on permission failure
- `acorn-cli/src/main.rs:146` — `build_global().unwrap()` — panics if thread pool already initialized
- `acorn-cli/src/io/mod.rs:56,58` — `stdout.write_all(...).unwrap()` and `stdout.flush().unwrap()` — panics on broken pipe
- `acorn-lib/src/analyzer/mod.rs:696` — `File::open(path).unwrap()` on download path

**Fix:** Propagate with `?` instead of `unwrap()`. stdout failures can use `let _ =` with a comment if acceptable.

---

### LOW IMPACT / LOW DIFFICULTY

#### 9. Numeric `as` casts — potential truncation on 32-bit platforms

- `acorn-lib/src/analyzer/readability/mod.rs` — 17 casts: `.count() as u32`, `.len() as u32`, `usize as f64`
- `acorn-lib/src/analyzer/check.rs` — 8 casts: `line_number as usize`, `index as u32`
- `acorn-lib/src/io/database/schema.rs` — 6 casts: `bool as i32`
- `acorn-lib/src/util/mod.rs` — 6 casts: `n as usize`, `n as char`, `bytes as f64`
- `acorn-lib/src/schema/hardware/memory.rs` — 2 casts: `value as f64`

**Fix:** Use `TryFrom<usize>::try_from(n)`?` for truncation-sensitive conversions; `as f64` is generally safe but should use `f64::from(n)` where possible.

---

# Numeric `as` Cast Refactoring (Phases 2–4)

**Constraints (from AGENTS.md + coding guidelines):**
- No `?` operator, no `.unwrap()`, no `.expect()`
- No `return` except as final expression
- Prefer functional style (combinators, `unwrap_or(fallback)`)
- `From::from()` only where std provides the impl

## Completed

### ✅ Phase 1 — `From::from()` (lossless, infallible, mechanical)

| File | Change | Rationale |
|------|--------|-----------|
| `acorn-lib/src/io/database/schema.rs:675,853` | `bool as i32` → `i32::from(bool)` | `From<bool> for i32` |
| `acorn-lib/src/io/database/schema.rs:739` | `is_deprecated/.../is_open_source as i32` (×3) → `i32::from(...)` | Same |
| `acorn-lib/src/io/database/schema.rs:769` | `is_reachable as i32` → `i32::from(is_reachable)` | Same |
| `acorn-macros/src/lib.rs:68` | macro codegen `value as i32` → `i32::from(value)` | Same |
| `acorn-lib/src/io/database/tests/mod.rs:106` | test `value as i32` → `i32::from` | Same |
| `acorn-lib/src/util/mod.rs:758` | `*byte as char` → `char::from(*byte)` | `From<u8> for char` |

**Note:** These are widening casts where `From` is unavailable in std. No clippy lint fires for them. Kept as-is:

| File | Cast | Reason From unavailable |
|------|------|------------------------|
| `acorn-lib/src/analyzer/check.rs:740,745,748` (×4) | `u32 as usize` | `From<u32>` for `usize` not implemented |
| `acorn-lib/src/io/mod.rs:1101` | `count as u64` | `From<usize>` for `u64` not implemented (arch-dependent) |
| `acorn-cli/src/commands/export/mod.rs:203` | `items.len() as u64` | Same |
| `acorn-lib/src/util/mod.rs:756,781` | `.len() as u128` | `From<usize>` for `u128` not implemented; also in `const` context |
| `acorn-lib/src/util/mod.rs:815` | `index as u128` | Same |

---

## Remaining

### Phase 2 — `TryFrom` + `unwrap_or(fallback)` (narrowing, safe saturating fallback)

For non-`Result` functions where truncation **could** theoretically occur but a saturating fallback is safe.

| # | File:Line | Current | Replacement | Fallback rationale |
|---|----------|---------|------------|--------------------|
| 10 | `readability/mod.rs:145,153,159,221,231` (×5) | `.count() as u32`, `.len() as u32` | `u32::try_from(count).unwrap_or(u32::MAX)` | Word/letter/sentence counts never exceed 4B in real text |
| 11 | `readability/mod.rs:406-418` (×10) | `.count() as isize` | `i64::try_from(count).unwrap_or(0)` | Regex match counts are tiny (single word) |
| 12 | `readability/mod.rs:419` (×1) | `count.max(1) as usize` | `usize::try_from(count.max(1)).unwrap_or(1)` | Always ≥ 1, positive; safe fallback |
| 13 | `check.rs:759` | `index as usize` (u64→usize) | `usize::try_from(index).unwrap_or(0)` | Index 0 is safe for `.get()` |
| 14 | `check.rs:780` | `max as usize` (u64→usize) | `usize::try_from(max).unwrap_or(usize::MAX)` | Saturated max length |
| 15 | `check.rs:761` | `u64 as u32` | `u32::try_from(v).unwrap_or(u32::MAX)` | Saturated line number |
| 16 | `util/mod.rs:758` | `(n % MODULUS) as usize` (u128→usize) | `usize::try_from(n % MODULUS).unwrap_or(0)` | Result < 32, always fits; `0` is unreachable but harmless |
| 17 | `database/mod.rs:239` | `DEFAULT_CACHE_TTL_SECONDS as i64` (u64→i64) | `i64::try_from(v).unwrap_or(3600)` | Const = 3600, always fits |
| 18 | `database/mod.rs:306` | `limit as i64` (usize→i64) | `i64::try_from(limit).unwrap_or(i64::MAX)` | Saturate DB limit at MAX |
| 19 | `util/mod.rs:904` | `size as u64` (f64→u64) | `u64::try_from(size).unwrap_or(u64::MAX)` | **HIGHEST PRIORITY** — catches NaN/inf/negative |

### Phase 3 — `#[allow(clippy::cast_*)]` with rationale (infallible by invariant)

For cases where the value is **guaranteed** to fit but Rust's type system can't prove it. Per AGENTS.md Strategy 6.

| # | File:Line | Cast | Allow lint | Rationale |
|---|----------|------|-----------|-----------|
| 20 | `pid/mod.rs:1275` | `remainder as u8` | `cast_possible_truncation` | `remainder = sum % 29`, always < 29 fits in u8 |
| 21 | `util/mod.rs:814` | `normalized as u8` (char→u8) | `cast_possible_truncation` | `normalized` is ASCII (0-9, A-Z) after `.to_ascii_uppercase()` + manual substitution |
| 22 | `util/mod.rs:894,902` | `bytes as f64` (u64→f64) | `cast_precision_loss` | Display formatting; precision loss >2^53 acceptable |
| 23 | `memory.rs:42,45` | `value as f64` (u64/i64→f64) | `cast_precision_loss` | Memory values are small (< exabytes), always exact in f64 |
| 24 | `readability/mod.rs:276,291` | `syllables as f64` (usize→f64) | `cast_precision_loss` | No `From` impl; counts never exceed 2^53 syllables |
| 25 | `readability/mod.rs:246-336` (×7) | `u32 as f64` | **Replace with `f64::from(val)`** — `From<u32> for f64` exists | No allow needed |

### Phase 4 — Enable clippy lints to prevent regressions

Add to `[workspace.lints.clippy]` in root `Cargo.toml`:

```toml
cast_possible_truncation = "deny"    # Prevents new narrowing as-casts
cast_sign_loss = "deny"               # Prevents sign-ignoring as-casts
cast_precision_loss = "deny"          # Prevents new precision-losing casts
cast_possible_wrap = "deny"           # Prevents wrapping as-casts
checked_conversions = "deny"          # Suggests TryFrom where possible
```

After Phases 1–3, these will pass cleanly (every existing cast will be either removed, replaced with `From`, wrapped in `TryFrom + unwrap_or`, or annotated with `#[allow]`). Any new `as` cast will fail CI.
