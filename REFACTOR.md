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

## 3. Download Decomposition

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

## 4. Reporting Facade

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

## 5. Execution Template

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

# Rust Safety Boundary Issues (Bugs Rust Won't Catch)

Analysis of the codebase against rules from [Bugs Rust Won't Catch](https://corrode.dev/blog/bugs-rust-wont-catch/) — patterns that the borrow checker and clippy do not enforce but are common sources of CVEs in Rust systems code.

---

### MEDIUM IMPACT / LOW DIFFICULTY

#### 1. `from_utf8_lossy` at Unix boundaries — silent data corruption

- `acorn-lib/src/util/cmd.rs:55,57` — command stdout/stderr lossy-converted, corrupts non-UTF-8 output
- `acorn-lib/src/prelude.rs:61,64` — `CommandOutput` traits use lossy conversion (widely consumed)
- `acorn-lib/src/io/http/mod.rs:143` — HTTP response body lossy-converted
- `acorn-cli/src/template/mod.rs:101` — embedded asset data lossy-converted
- `acorn-lib/src/schema/prompt/mod.rs:207` — prompt response data lossy-converted

**Fix:** Prefer returning `Vec<u8>` or `OsString` across boundaries, letting callers handle encoding. Avoid lossy UTF-8 round-trips for command output, paths, and env values.

#### 2. `unwrap()` on `Option` — panics on unexpected None

- `acorn-lib/src/analyzer/mod.rs:731` — `self.config.unwrap()` panics if no config set
- `acorn-lib/src/analyzer/mod.rs:803` — `which(name).unwrap()` panics if binary not found
- `acorn-lib/src/analyzer/mod.rs:807` — `cmd!(name, ["--version"]).unwrap()` panics on command failure
- `acorn-lib/src/schema/mod.rs:585,681` — `iter().next().unwrap()` on potentially empty collections
- `acorn-lib/src/lib.rs:252` — same pattern
- `acorn-lib/src/util/mod.rs:858` — `.last().unwrap()` on potentially empty segments
- `acorn-cli/src/commands/export/mod.rs:74,81` — `Option::unwrap()` on CLI options

**Fix:** Use `ok_or_else(|| eyre!(...))`, `unwrap_or_default()`, or match with fallback logging.

#### 3. `unwrap()` on I/O operations — crash on transient failures

- `acorn-lib/src/io/mod.rs:853` — `set_permissions(path, 0o755).unwrap()` — dies on permission failure
- `acorn-cli/src/main.rs:146` — `build_global().unwrap()` — panics if thread pool already initialized
- `acorn-cli/src/io/mod.rs:56,58` — `stdout.write_all(...).unwrap()` and `stdout.flush().unwrap()` — panics on broken pipe
- `acorn-lib/src/analyzer/mod.rs:696` — `File::open(path).unwrap()` on download path

**Fix:** Propagate with `?` instead of `unwrap()`. stdout failures can use `let _ =` with a comment if acceptable.

---

### LOW IMPACT / LOW DIFFICULTY

#### 4. Numeric `as` casts — potential truncation on 32-bit platforms

- `acorn-lib/src/analyzer/readability/mod.rs` — 17 casts: `.count() as u32`, `.len() as u32`, `usize as f64`
- `acorn-lib/src/analyzer/check.rs` — 8 casts: `line_number as usize`, `index as u32`
- `acorn-lib/src/io/database/schema.rs` — 6 casts: `bool as i32`
- `acorn-lib/src/util/mod.rs` — 6 casts: `n as usize`, `n as char`, `bytes as f64`
- `acorn-lib/src/schema/hardware/memory.rs` — 2 casts: `value as f64`

**Fix:** Use `TryFrom<usize>::try_from(n)`?` for truncation-sensitive conversions; `as f64` is generally safe but should use `f64::from(n)` where possible.

---

# Numeric `as` Cast Refactoring (Phases 2-4)

**Constraints (from AGENTS.md + coding guidelines):**
- No `?` operator, no `.unwrap()`, no `.expect()`
- No `return` except as final expression
- Prefer functional style (combinators, `unwrap_or(fallback)`)
- `From::from()` only where std provides the impl

## Remaining

### Phase 2 — `TryFrom` + `unwrap_or(fallback)` (narrowing, safe saturating fallback)

For non-`Result` functions where truncation **could** theoretically occur but a saturating fallback is safe.

| # | File:Line | Current | Replacement | Fallback rationale |
|---|----------|---------|------------|--------------------|
| 5 | `readability/mod.rs:145,153,159,221,231` (×5) | `.count() as u32`, `.len() as u32` | `u32::try_from(count).unwrap_or(u32::MAX)` | Word/letter/sentence counts never exceed 4B in real text |
| 6 | `readability/mod.rs:406-418` (×10) | `.count() as isize` | `i64::try_from(count).unwrap_or(0)` | Regex match counts are tiny (single word) |
| 7 | `readability/mod.rs:419` (×1) | `count.max(1) as usize` | `usize::try_from(count.max(1)).unwrap_or(1)` | Always ≥ 1, positive; safe fallback |
| 8 | `check.rs:759` | `index as usize` (u64→usize) | `usize::try_from(index).unwrap_or(0)` | Index 0 is safe for `.get()` |
| 9 | `check.rs:780` | `max as usize` (u64→usize) | `usize::try_from(max).unwrap_or(usize::MAX)` | Saturated max length |
| 10 | `check.rs:761` | `u64 as u32` | `u32::try_from(v).unwrap_or(u32::MAX)` | Saturated line number |
| 11 | `util/mod.rs:758` | `(n % MODULUS) as usize` (u128→usize) | `usize::try_from(n % MODULUS).unwrap_or(0)` | Result < 32, always fits; `0` is unreachable but harmless |
| 12 | `database/mod.rs:239` | `DEFAULT_CACHE_TTL_SECONDS as i64` (u64→i64) | `i64::try_from(v).unwrap_or(3600)` | Const = 3600, always fits |
| 13 | `database/mod.rs:306` | `limit as i64` (usize→i64) | `i64::try_from(limit).unwrap_or(i64::MAX)` | Saturate DB limit at MAX |
| 14 | `util/mod.rs:904` | `size as u64` (f64→u64) | `u64::try_from(size).unwrap_or(u64::MAX)` | **HIGHEST PRIORITY** — catches NaN/inf/negative |

### Phase 3 — `#[allow(clippy::cast_*)]` with rationale (infallible by invariant)

For cases where the value is **guaranteed** to fit but Rust's type system can't prove it. Per AGENTS.md Strategy 6.

| # | File:Line | Cast | Allow lint | Rationale |
|---|----------|------|-----------|-----------|
| 15 | `pid/mod.rs:1275` | `remainder as u8` | `cast_possible_truncation` | `remainder = sum % 29`, always < 29 fits in u8 |
| 16 | `util/mod.rs:814` | `normalized as u8` (char→u8) | `cast_possible_truncation` | `normalized` is ASCII (0-9, A-Z) after `.to_ascii_uppercase()` + manual substitution |
| 17 | `util/mod.rs:894,902` | `bytes as f64` (u64→f64) | `cast_precision_loss` | Display formatting; precision loss >2^53 acceptable |
| 18 | `memory.rs:42,45` | `value as f64` (u64/i64→f64) | `cast_precision_loss` | Memory values are small (< exabytes), always exact in f64 |
| 19 | `readability/mod.rs:276,291` | `syllables as f64` (usize→f64) | `cast_precision_loss` | No `From` impl; counts never exceed 2^53 syllables |
| 20 | `readability/mod.rs:246-336` (×7) | `u32 as f64` | **Replace with `f64::from(val)`** — `From<u32> for f64` exists | No allow needed |

### Phase 4 — Enable clippy lints to prevent regressions

Add to `[workspace.lints.clippy]` in root `Cargo.toml`:

```toml
cast_possible_truncation = "deny"    # Prevents new narrowing as-casts
cast_sign_loss = "deny"               # Prevents sign-ignoring as-casts
cast_precision_loss = "deny"          # Prevents new precision-losing casts
cast_possible_wrap = "deny"           # Prevents wrapping as-casts
checked_conversions = "deny"          # Suggests TryFrom where possible
```

After Phases 1-3, these will pass cleanly (every existing cast will be either removed, replaced with `From`, wrapped in `TryFrom + unwrap_or`, or annotated with `#[allow]`). Any new `as` cast will fail CI.
