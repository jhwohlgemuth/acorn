---
type: Plan
title: "CLI test coverage phase 1"
description: "High-priority test gaps: link command, assert_cmd stabilization, and error path coverage for core commands."
difficulty: medium
impact: high
package: [acorn-cli]
tags: [testing, coverage, cli, link, assert_cmd, error-paths]
status: planned
---

# Goal

Close the three highest-impact testing gaps: the untested `link` command, broken binary-level integration tests, and absent error paths in core commands.

# Package A — `link` command tests

## Current state

- Zero tests (only one CLI parse assertion for `--what-if` alias)
- `with_context()` JSON-LD injection is not wired up — tests cover current behavior only
- `standard` argument is parsed but ignored by `run()`
- `format_with()` performs media resolution, vocabulary matching, and telephone formatting

## Prerequisites

- Fixture: `tests/fixtures/data/valid_project/` (JSON with images, keywords, sponsors)
- Fixture: `tests/fixtures/data/valid_highlight/` (simpler valid RAD)
- Fixture: `tests/fixtures/data/invalid_project_a/` (invalid RAD)
- Fixture: `tests/fixtures/malformed-rad.json` (deserialization error)
- All tests use `temp_test_dir()` + `TestCleanup` for output directories

## Test cases

### TC-A1: Valid JSON produces .jsonld file

**GIVEN** a valid RAD JSON file at `data/valid_project/index.json`
**WHEN** `link` is run with that file path and a temp output directory
**THEN** a `.jsonld` file is created in the output directory
**AND** the file contains valid JSON
**AND** the file is readable as `ResearchActivity`

### TC-A2: Valid YAML produces .jsonld file

**GIVEN** a valid RAD YAML file at `data/invalid_project_b/index.yaml` (YAML format, valid enough for link)
**WHEN** `link` is run with that file path
**THEN** a `.jsonld` file is created with valid JSON content

### TC-A3: Dry run prints diff without writing

**GIVEN** a valid RAD file
**WHEN** `link` is run with `--dry-run` (`--what-if`)
**THEN** no `.jsonld` file is created
**AND** output contains a diff preview

### TC-A4: Missing path returns error

**GIVEN** a non-existent file path
**WHEN** `link` is run with that path
**THEN** the result is an error
**AND** the error message indicates the file was not found

### TC-A5: Invalid JSON returns deserialization error

**GIVEN** a malformed JSON file at `malformed-rad.json`
**WHEN** `link` is run with that file path
**THEN** the result is an error
**AND** the error message indicates a deserialization failure

### TC-A6: Ignore regex skips files

**GIVEN** a directory containing multiple RAD files
**WHEN** `link` is run with `--ignore "skip_me"`
**THEN** files matching the regex are not processed
**AND** non-matching files produce `.jsonld` output

### TC-A7: Multiple files processed in parallel

**GIVEN** a directory containing 3+ valid RAD files
**WHEN** `link` is run with the directory path
**THEN** all files produce `.jsonld` output
**AND** no panics or race conditions occur

## Risk notes

- `format_with()` performs filesystem I/O for image discovery — tests need the full fixture directory structure, not just the JSON file
- The `link` command uses `rayon::par_iter` for parallel processing — concurrent tests against the same temp dir could interfere; use separate temp dirs per test

---

# Package B — `assert_cmd` stabilization

## Current state

- `tests/check.rs`: 3 tests marked `#[ignore = "Broken for reasons unknown"]` — reference missing fixture (`dcat-invalid-2-errors.json`), unknown breakage cause
- `tests/help.rs`: 2 working tests (`test_command_help`, `test_subcommand_help`)
- CI uses `cargo test` which silently skips ignored tests
- `assert_cmd` 2.0.16 is a current dev-dependency of `acorn-cli`

## Plan

### Step 1: Delete broken tests

Remove the 3 ignored tests from `tests/check.rs`. Rationale:
- DCAT standard checking is already covered by 4 unit tests in `check/tests/mod.rs`
- The broken tests reference a missing fixture file
- The ignore reason ("Broken for reasons unknown") indicates no one understands what they test
- Keeping dead tests creates false confidence and maintenance burden

### Step 2: Rewrite `tests/check.rs`

Replace with stable binary-level tests:

#### TC-B1: Check help exits successfully

**WHEN** `acorn check --help` is invoked via `assert_cmd`
**THEN** exit code is 0
**AND** stdout contains expected flag descriptions

#### TC-B2: Check valid fixture exits successfully

**GIVEN** a valid RAD fixture at `data/valid_project/index.json`
**WHEN** `acorn check <fixture-path>` is invoked
**THEN** exit code is 0

#### TC-B3: Check invalid fixture exits successfully (no-fail default)

**GIVEN** an invalid RAD fixture
**WHEN** `acorn check <fixture-path>` is invoked (no `--fail-on-error`)
**THEN** exit code is 0 (check reports issues but doesn't fail by default)

### Step 3: Add `tests/export.rs`

#### TC-B4: Export help exits successfully

**WHEN** `acorn export --help` is invoked
**THEN** exit code is 0

#### TC-B5: Export JSON to temp directory

**GIVEN** a valid RAD fixture
**WHEN** `acorn export <fixture> --format json --output <temp-dir>` is invoked
**THEN** exit code is 0
**AND** a JSON file exists in the temp directory

### Step 4: Add `tests/format.rs`

#### TC-B6: Format help exits successfully

**WHEN** `acorn format --help` is invoked
**THEN** exit code is 0

#### TC-B7: Format dry-run exits successfully

**GIVEN** a valid RAD fixture
**WHEN** `acorn format <fixture> --dry-run` is invoked
**THEN** exit code is 0
**AND** the fixture file is not modified

### Step 5: Add `tests/doctor.rs`

#### TC-B8: Doctor help exits successfully

**WHEN** `acorn doctor --help` is invoked
**THEN** exit code is 0

#### TC-B9: Doctor runs successfully

**WHEN** `acorn doctor` is invoked
**THEN** exit code is 0

## Risk notes

- Binary tests compile and invoke the full `acorn` binary — they are slower than unit tests
- Tests that write output must use `temp_test_dir()` + `TestCleanup` to avoid polluting the workspace
- The `check` command defaults to not failing on validation errors — tests asserting `.success()` are correct for this behavior
- CI runs `cargo test` without `--include-ignored` — deleted tests won't affect CI; new non-ignored tests will run automatically

---

# Package C — Error path tests for existing commands

## Current state

- `check`: 14 tests, all happy path or controlled invalid data — no file-system error paths
- `export`: 9 tests, 2 ignored — no error path coverage
- `format`: 4 tests — no error path coverage
- `doctor`: 9 tests — no `--fix` error path coverage

## Test cases

### TC-C1: Check with missing file

**GIVEN** a file path that does not exist
**WHEN** `check` is run with that path
**THEN** the result is an error
**AND** the error message mentions the file was not found

### TC-C2: Check with empty directory

**GIVEN** an empty directory
**WHEN** `check` is run with that directory path
**THEN** the result is an error or warning indicating no files found

### TC-C3: Check with invalid JSON file

**GIVEN** a file containing invalid JSON (not valid RAD)
**WHEN** `check` is run with that file
**THEN** the result is an error
**AND** the error message indicates a deserialization or schema failure

### TC-C4: Export with missing input path

**GIVEN** a file path that does not exist
**WHEN** `export` is run with that path
**THEN** the result is an error
**AND** the error message mentions the file was not found

### TC-C5: Export with missing output directory

**GIVEN** a valid RAD fixture
**WHEN** `export` is run with `--output /nonexistent/path/`
**THEN** the result is an error
**AND** the error message indicates the output directory does not exist

### TC-C6: Export crosswalk without --to flag

**GIVEN** a valid RAD fixture
**WHEN** `export` is run with `--crosswalk` but no `--to` target
**THEN** the result is an error
**AND** the error message indicates a target standard is required

### TC-C7: Format with corrupt file

**GIVEN** a file containing malformed JSON (not parseable)
**WHEN** `format` is run with that file
**THEN** the result is an error
**AND** the error message indicates a parse failure

### TC-C8: Format with missing file

**GIVEN** a file path that does not exist
**WHEN** `format` is run with that path
**THEN** the result is an error
**AND** the error message mentions the file was not found

### TC-C9: Doctor --fix returns not-implemented error

**WHEN** `doctor` is run with `--fix`
**THEN** the result is an error
**AND** the error message indicates the feature is not yet implemented

## Risk notes

- Error message text may change — assertions should match on key substrings (e.g., "not found", "deserialization"), not exact strings
- Some commands may panic instead of returning errors on bad input — these would be bugs to file, not test expectations
- `format` and `check` commands may handle missing files differently (one returns `Result::Err`, the other may log and skip) — verify actual behavior before writing assertions

---

# Acceptance criteria

Phase 1 is complete when:

1. All 22 test cases (7 link + 9 assert_cmd + 9 error path — TC-A1 through TC-C9, minus overlap) pass with `make test` and `make test-ci`
2. The 3 broken tests in `tests/check.rs` are removed
3. New `assert_cmd` test files exist: `tests/check.rs` (rewritten), `tests/export.rs`, `tests/format.rs`, `tests/doctor.rs`
4. A `#[cfg(test)] mod tests` module exists in `crates/acorn-cli/src/commands/link/mod.rs`
5. No new `#[ignore]` annotations were added
6. All test modules include the clippy suppression banner
7. All error tests verify both `is_err()` and error message content
