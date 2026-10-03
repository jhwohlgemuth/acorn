---
type: Plan
title: "Expose reusable host prerequisites for Xylem"
description: "Publish the managed command contract and add generic JSONC, configuration discovery, and project-cache helpers needed by Xylem without moving workflow policy into ACORN."
difficulty: medium
impact: high
package: [acorn-core, acorn-host, acorn-cmd, acorn-lib, workspace]
tags: [refactor, host, config, jsonc, filesystem, process, downstream]
status: planned
---

# Goal

Give Xylem public ACORN APIs for managed child processes, strict JSONC input,
ordered configuration discovery, and platform project-cache directories. Keep
the APIs application-neutral so ACORN can use the same implementations.

The dependent Xylem work is tracked in the
[workflow dashboard plan](https://code.ornl.gov/research-enablement/xylem/-/blob/main/.wiki/features/workflow-dashboard.md).

# Current evidence

- Xylem pins ACORN `0.3.2` at revision `5d29344660334e2f0fd09b96a6f84b4510e57286`.
- The current ACORN worktree contains the managed `acorn-cmd` execution API:
  process groups, concurrent stdout and stderr draining, streaming sinks,
  bounded capture, timeouts, cancellation, structured termination, and child
  cleanup. That work must be committed, released, and consumed through a
  pinned revision or compatible published version before Xylem relies on it.
- `acorn-host::config` reads caller-owned JSON and YAML types. It does not read
  JSONC or resolve an ordered list of default names.
- ACRE already searches `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, and
  `.acorn.yml` in order, but the resolver is attached to
  `ApplicationConfiguration` in `acorn-lib` and cannot accept another
  application's candidate names.
- Strict JSONC options are repeated in ACORN today. The
  [workspace duplication plan](workspace-duplication-consolidation.md) already
  assigns that policy to `acorn-core`; this plan consumes that owner instead of
  introducing another option literal.
- `standard_project_folder` resolves ACORN's cache directory, creates a
  namespace, and appends a random identifier. It is hard-coded to ACORN and
  cannot return Xylem's stable `runs` root.
- `acorn-tui` has useful design examples, but its event loop, theme, and most
  widgets are private and depend on the ACORN application. Extracting a shared
  TUI framework would add a larger package boundary than the Xylem dashboard
  needs.
- ACORN's host diagnostics capture a one-time system snapshot. They do not
  provide live process-tree metrics or workflow telemetry.

# Project code boundaries

- **Update:** the core JSONC utility owner, `acorn-host` configuration and
  filesystem adapters, their focused tests and documentation, ACORN's existing
  project-folder helper, workspace manifests and lockfile, the downstream-host
  compile fixture, and package release metadata.
- **Create:** generic ordered configuration-path resolution and project-cache
  directory error/contracts in the owning `acorn-host` modules. Prefer leaf
  test files when existing tests would otherwise mix unrelated host behavior.
- **Delete:** repeated strict JSONC option literals after the core owner lands.
  No command compatibility API is removed.

# Dependencies and ordering

1. Finish the JSONC-policy slice of
   [workspace duplication consolidation](workspace-duplication-consolidation.md)
   so `acorn-core` has one strict option owner.
2. Commit and validate the managed `acorn-cmd` implementation already present
   in the worktree. Preserve `output` and `checked_output` as compatibility
   adapters.
3. Add the host configuration and project-directory contracts below, then
   migrate ACORN's matching call sites.
4. Extend the downstream-host fixture so it compiles and exercises the public
   APIs without depending on `acorn-lib`.
5. Publish a compatible ACORN package set and give Xylem one commit or release
   to pin. Do not point Xylem at a sibling path dependency.

The JSONC slice takes the core utility and module-index locks. Host additions
take the `acorn-host` leaf, workspace graph, and package release locks. Managed
command changes take the command-runtime lock and must not overlap another
`acorn-cmd` API change.

# Public contracts

## Strict JSONC

Expose `acorn_core::util::strict_jsonc_options()` and use it for JSON
detection, JSONC value parsing, and CST parsing. The accepted syntax remains:

- line and block comments;
- trailing commas;
- quoted property names and double-quoted strings;
- no missing commas, single quotes, hexadecimal numbers, unary-plus numbers,
  or loose property names.

Add `acorn_host::config::read_jsonc<T>` and route `.jsonc` through the generic
`read<T>` dispatcher. Decode to a `serde_json::Value` with the shared policy
before deserializing the caller-owned type. Keep YAML behavior unchanged;
callers such as Xylem may continue applying merge keys before typed
deserialization.

## Ordered configuration discovery

Add a generic resolver with the behavior of:

```rust
resolve_config_path(
    explicit: Option<&Path>,
    directory: &Path,
    candidates: &[&str],
) -> Option<PathBuf>
```

An explicit path is returned unchanged, including when it does not exist, so
the later read reports that exact error. Without an explicit path, return the
first candidate that is a regular file. Return `None` when no candidate
exists. Do not embed ACORN or Xylem filenames in the generic helper.

Refactor ACRE's default-name branch to pass `DEFAULT_CONFIG_FILENAMES`.
Preserve its documented order and its existing explicit-path behavior. Xylem
uses the generic resolver directly and therefore reports a missing explicit
path instead of starting discovery.

## Project cache directories

Add an `acorn-host` filesystem function with the behavior of:

```rust
project_cache_directory(
    qualifier: &str,
    organization: &str,
    application: &str,
    namespace: &SafePath,
) -> Result<PathBuf, FsError>
```

Resolve the platform cache directory through `directories::ProjectDirs`,
create the namespace, and return its root without adding a random child.
`SafePath` rejects an empty, absolute, or traversing namespace before any
directory is created.

Refactor `standard_project_folder` to call the generic helper with ACORN's
constants and append its existing generated identifier. Preserve its current
ACORN path layout while removing the duplicated `ProjectDirs` logic.

## Managed command release contract

Publish the existing `CommandSpec::{execute,checked_execute}` API together
with `ExecutionOptions`, `CancellationToken`, `OutputSink`, `OutputStream`,
`CaptureLimits`, `ExecutionResult`, `ProcessTermination`, and structured
errors. Preserve separate stream ordering, exact truncation counts, duration,
exit code or signal, and guaranteed process-group cleanup.

The callback remains byte-oriented. ACORN does not add line framing, terminal
sanitization, journal records, workflow cancellation policy, or signal
handlers; those belong to the consumer.

# Explicit non-goals

- No Xylem filenames, run-store layout, journal types, scheduler state, or
  workflow events enter an ACORN crate.
- No shared TUI framework is extracted in this plan. Xylem may use Ratatui and
  ACORN's public color/schema types directly.
- No live system or process-tree sampler is added to ACORN.
- No ACRE schema or configuration selection behavior changes.
- No removal of `acorn-cmd` compatibility methods or macros.

# Validation

1. Test every strict JSONC allowance and rejection through the core option
   owner and `acorn-host::config` typed reader.
2. Test candidate precedence with zero, one, and all candidates present;
   explicit existing and missing paths; directories with candidate names; and
   relative and absolute explicit paths.
3. Re-run ACRE discovery and format tests to prove the four canonical names
   retain their order.
4. Test project-cache resolution with an injected platform root where
   possible, invalid namespaces, creation failures, and an ACORN
   `standard_project_folder` path that still ends in a generated child.
5. Run managed-command integration tests for mixed streams, bounded tails,
   sink failure, timeout, cancellation, non-zero exit, descendants, and reap
   behavior on Unix and Windows.
6. Compile and run `tests/downstream-host` using only `acorn-core`,
   `acorn-cmd`, and `acorn-host`.
7. Run the supported portable checks, package tests, `make lint`, and
   `make test`, then verify the published package graph resolves without path
   dependencies.

# Acceptance criteria

- Xylem can parse strict JSONC without carrying its own parser options.
- Any consumer can resolve an explicit config path or ordered default names
  without depending on `ApplicationConfiguration`.
- A missing explicit path is never replaced by a default candidate.
- Xylem can obtain and create its system cache `runs` namespace without
  copying ACORN's `ProjectDirs` logic or receiving a random child path.
- Managed commands expose streaming, bounded capture, cancellation, timeout,
  termination metadata, and descendant cleanup from a committed and published
  API.
- The downstream fixture proves the required surface without `acorn-lib`.
- ACORN retains its ACRE filename order, explicit-path behavior, and existing
  project-folder layout.
- The plan adds no Xylem workflow, journal, scheduler, telemetry, or TUI
  policy to ACORN.

# Related plans

- [Workspace duplication consolidation](workspace-duplication-consolidation.md)
- [Crate boundary architecture](crate-boundary-architecture.md)
- [Dependency feature defaults](portability/dependency-feature-defaults.md)
