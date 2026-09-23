---
type: Plan
title: "CLI test coverage initiative"
description: "Systematic improvement of CLI test coverage with phased priorities, starting with high-impact gaps."
difficulty: medium
impact: high
package: [acorn-cli]
tags: [testing, coverage, cli, integration, assert_cmd]
status: in-progress
---

# Goal

Establish systematic CLI test coverage across all commands, starting with high-impact gaps and progressing toward comprehensive coverage. Each phase produces testable, reviewable increments.

## Persistent benchmark evidence

Criterion results from every semantic version tag and relevant default-branch pipeline are saved as version-named baselines and retained in immutable `acorn-benchmarks` GitLab generic packages. Each package includes raw Criterion output, normalized JSON and CSV history, and a self-contained trend report. Comparisons are limited to matching runner and hardware fingerprints; the workflow is informational rather than a regression gate. Local commands and package contents are documented in `CONTRIBUTING.md`.

# Current state

| Command | Tests | Coverage | Priority |
|---------|-------|----------|----------|
| download | 106 | Excellent | — |
| check | 25 command + 7 integration | Good | — |
| doctor | 9 | Good | — |
| export | 24 | Good | — |
| sync | 12 | Good | — |
| format | 12 | Moderate | — |
| import | 17 | Good | — |
| link | 1 configured-gate binary test | Low | Phase 1 |
| create | 15 command + 1 integration | Moderate | — |
| serve | 0 | None | Phase 2 |
| gather | 24 | Good | — |
| swhid | 0 | None | Phase 2 |
| skill | 0 | None | Phase 3 |
| tui | 0 | None | Phase 3 |
| Binary (`assert_cmd`) | 41 `cargo_bin` call sites | Moderate | Phase 1 gaps only |

# Testing conventions

All CLI tests follow these patterns. New tests must adhere to them.

## Module structure

Every test module opens with the clippy suppression banner:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
```

## Naming

CLI test functions use `test_{command}_{scenario}`:

```rust
#[tokio::test]
async fn test_check_valid_project() -> Result<()> { ... }
```

## Assertions

- Always include a descriptive failure message: `assert!(result.is_ok(), "Valid project should pass check");`
- Use `pretty_assertions::assert_eq!` when comparing structured output
- Use `insta::assert_snapshot!` for complex output that benefits from visual review
- Never use `#[should_panic]` — test errors via `Result` assertions

## Fixtures

Load test data via `fixture_path()`:

```rust
use crate::test::util::fixture_path;
let path = fixture_path("data/valid_project/index.json");
```

All fixtures live under `tests/fixtures/` at the workspace root.

## Temp directories

Create isolated temp dirs with RAII cleanup:

```rust
use crate::test::util::{temp_test_dir, TestCleanup};
let temp_dir = temp_test_dir("export_json");
let _cleanup = TestCleanup::new(temp_dir.clone());
```

All temp artifacts go under `target/test_artifacts/`, never in the system temp directory.

## Error testing

Test errors via `Result` assertions, not panics:

```rust
let result = run(...).await;
assert!(result.is_err(), "Should fail for missing file");
let text = format!("{}", result.unwrap_err());
assert!(text.contains("not found"), "Error should mention missing file");
```

## Async tests

Command tests use `#[tokio::test]` returning `Result<()>`:

```rust
#[tokio::test]
async fn test_check_valid_project() -> Result<()> {
    let result = run(path, false, None, false, OFFLINE, &verbosity).await;
    assert!(result.is_ok(), "Valid project should pass check");
    Ok(())
}
```

## Offline mode

Define a constant and pass it to `run()`:

```rust
const OFFLINE: bool = true;
```

## Ignored tests

Use `#[ignore]` only for tests with genuine external dependencies:

```rust
#[ignore = "Requires Playwright for PDF generation"]
```

Never use `#[ignore]` for broken or unknown failures — fix or delete instead.

# Phases

- [Phase 1](cli-coverage-phase-1.md) — High priority: link command coverage and the remaining binary/error-path gaps
- Phase 2 — Medium priority: remaining flag coverage, binary tests for all commands
- Phase 3 — Low priority: edge cases, verbose modes, platform-gated tests

# Related testing plans

- [Semantic transformation laws](semantic-transformation-laws.md) — Enforce round-trip, canonicalization, preservation, explicit-loss, and commuting-path contracts across RAD transformations and selected custom codecs.
- [Informational Miri quality checks](miri-quality.md) — Run curated undefined-behavior checks through portable public APIs, including refactored PID boundaries.
- [Creusot verification for PID checksum calculations](creusot-pid-checksums.md) — Formally verify the four shared PID check-digit algorithms and PID-specific callers.

# Acceptance criteria

Each phase is complete when:

1. All new tests pass with `make test` and `make test-ci`
2. No new `#[ignore]` annotations without documented external dependency
3. All test modules include the clippy suppression banner
4. Test functions follow the `test_{command}_{scenario}` naming convention
5. Error tests verify both `is_err()` and error message content
