---
type: Plan
title: "Support lightweight downstream native consumers"
description: "Extract narrow native host capabilities so downstream tools such as Xylem can use ACORN without enabling broad analysis or std graphs."
difficulty: high
impact: high
package: [workspace, acorn-core, acorn-cmd, acorn-host, acorn-lib]
tags: [refactor, portability, downstream, dependencies, crate-extraction]
status: planned
---

# Goal

Make ACORN's reusable native capabilities available to small downstream tools without requiring `acorn-lib` or its broad `analysis` dependency graph. Use Xylem as the first external acceptance consumer and as a concrete source of API requirements.

This plan follows the repository's existing target architecture:

- `acorn-core` owns portable value types and pure logic.
- `acorn-cmd` owns command construction, discovery, execution, and checked-output behavior.
- `acorn-host` owns concrete native filesystem, HTTP, download, source, temporary-directory, configuration, and terminal adapters.
- `acorn-lib` consumes those crates and stops masking ownership through compatibility re-exports.

The objective is not to move Xylem-specific workflow policy into ACORN. It is to expose the narrow, reusable primitives that let Xylem delete local infrastructure while keeping its own workflow model.

# Current evidence

Xylem currently enables `acorn-lib`'s `analysis` feature and `acorn-core`'s `alloc` feature, but uses only a small part of the resulting API:

- `acorn::io::{read_file, file_checksum, make_executable}`
- `acorn::util::Label`
- `acorn::{args, cmd}`
- `acorn_core::{Location, Scheme}`
- `SemanticVersion`, `Checksum`, and `StringConversion`

That dependency shape pulls in analysis, database, document, archive, and export functionality that is unrelated to Xylem. A baseline `cargo tree` review found roughly 671 normal package/version nodes in the Xylem CLI graph. Even isolated ACORN feature selections remain wider than their names suggest: `acorn-lib --no-default-features --features cmd` resolves roughly 239 nodes, while `acorn-core --no-default-features --features alloc` resolves roughly 93.

Xylem also maintains local versions of infrastructure already present, or naturally owned, in ACORN:

- remote allow-list policy, URL validation, redirect validation, redaction, probing, and bounded Git checks;
- download streaming, temporary-file cleanup, checksum verification, and atomic installation;
- process construction and checked execution;
- temporary directories and executable permissions;
- JSON/YAML configuration loading;
- URI parsing, including an SSH workaround because `Scheme` does not represent SSH;
- interpolation and command discovery.

The overlap is useful evidence for crate boundaries. The APIs should be extracted because two real consumers need them, not merely because they are theoretically reusable.

# Project code boundaries

## Update

- Workspace manifests, lockfile, feature declarations, portable checks, and package metadata.
- `acorn-core` location, URI, interpolation, checksum, semantic-version, and other dependency-light modules.
- The implementations currently under `acorn-lib/src/io/`, especially HTTP, download, source, temporary-directory, filesystem, and structured-data helpers.
- Command and process code covered by the [acorn-cmd extraction](../acorn-cmd-extraction.md).
- First-party ACORN imports, tests, documentation, and examples that consume moved APIs.

## Create

- `crates/acorn-host/`, with capabilities introduced one coherent slice at a time.
- A small downstream-consumer fixture that compiles against only `acorn-core`, `acorn-cmd`, and selected `acorn-host` features.
- Focused tests for host policy and failure cleanup that do not require the analyzer or schema stack.

## Delete

- Moved implementations from `acorn-lib` after first-party callers migrate.
- Obsolete feature forwarding and ownership-masking re-exports.
- Dependencies that become unused after each extraction slice.

Do not add generic host traits as part of this plan. The [host abstraction traits plan](host-abstraction-traits.md) remains deferred until multiple implementations prove a stable seam. This plan exposes concrete, feature-gated native APIs.

# Dependencies

This work depends on:

1. [Dependency feature defaults](dependency-feature-defaults.md), so narrow crate selections do not silently activate unrelated subsystems.
2. [acorn-cmd extraction](../acorn-cmd-extraction.md), so host-side Git probing and command execution compose with a dependency-light command crate.

The plan can proceed in capability slices after those foundations land. It does not depend on the deferred host-trait design, browser services, analyzer boundaries, or schema migration.

# Recommendation

## `acorn-core`

Keep only portable values, parsing, validation, and pure transformations here.

- Add `Scheme::SSH` and preserve SSH URIs without lossy reconstruction.
- Keep `Location`, `Checksum`, `SemanticVersion`, interpolation, and path-independent validation dependency-light.
- Correct feature relationships so `alloc` and other narrow selections do not enable unrelated dependencies.
- Keep interpolation pure, deterministic, and capable of handling placeholders whose source values contain spaces.

## `acorn-cmd`

Complete the existing extraction before moving host consumers.

- Preserve the existing argument and command-construction ergonomics.
- Add a public command specification and checked-output API suitable for downstream use.
- Own executable discovery and bounded, noninteractive execution.
- Keep shell linting optional.
- Do not depend on `acorn-core`, `acorn-lib`, schemas, or analysis code.

## `acorn-host`

Expose concrete modules behind additive, narrowly composed features:

- `fs`: read files, calculate file checksums, set executable permissions, convert file URIs to paths, validate safe paths, and manage temporary directories.
- `config`: deserialize JSON, YAML, and YML into caller-owned types without depending on ACORN schemas.
- `http`: construct clients with exact allowed-authority checks, timeouts, redirect revalidation, and safe diagnostic redaction.
- `download`: stream to a sibling partial file, enforce optional size and checksum expectations, clean up failures, and atomically install the completed file.
- `source`: open local, file-URI, and HTTP sources represented by `acorn-core::Location`.
- `terminal`: expose small presentation primitives such as labels without pulling in analysis or reporting stacks.

Feature relationships must stay explicit. None of these features may activate analyzer, schema, database, document, archive, export, or service dependencies.

## Remote transport policy

The shared HTTP and source layer should implement the security-sensitive mechanics that Xylem currently owns locally:

- Accept allow-list entries as `HOST` or `HOST:PORT`, with exact normalized authority matching.
- Treat an absent or empty allow-list as unrestricted; callers can impose stricter workflow policy.
- Validate both the original request and every redirect target.
- Redact credentials, query strings, and fragments from errors and logs.
- Provide a bounded `HEAD` probe with a bounded `GET` fallback when a server does not support `HEAD` reliably.
- Provide bounded, noninteractive Git reachability probing by composing with `acorn-cmd`.

Offline mode, checksum-required policy, executable installation, retry strategy, and workflow-specific error messages remain downstream responsibilities unless another independent consumer proves a common abstraction.

# Implementation sequence

1. Tighten `acorn-core` feature edges and add SSH location support with focused tests.
2. Complete `acorn-cmd` extraction and migrate ACORN's internal command users.
3. Freeze the Xylem-shaped contracts as small compile-tested examples before moving implementations.
4. Create `acorn-host` and extract one capability at a time, beginning with filesystem and temporary-directory helpers, then configuration, HTTP, download, source, and terminal support.
5. Migrate ACORN's first-party callers directly to the owning crate after each slice; remove the corresponding implementation and dependency from `acorn-lib` without adding compatibility re-exports.
6. Add a downstream fixture whose manifest contains only the required `acorn-core`, `acorn-cmd`, and `acorn-host` features.
7. Migrate the sibling Xylem repository as the external acceptance test. Remove its `acorn-lib` `analysis` dependency and delete the local infrastructure superseded by the new APIs.
8. Update crate-boundary documentation, feature tables, examples, release notes, and the plan network as each slice lands.

# Validation

## Behavior

- Round-trip local, HTTP, HTTPS, and SSH locations, including authentication and port forms that must remain intact.
- Exercise exact authority matching, default ports, IPv6 authorities, redirects, and rejection of suffix or prefix lookalikes.
- Verify diagnostic redaction removes credentials, queries, and fragments.
- Cover `HEAD` success, `HEAD` rejection with `GET` fallback, timeouts, and oversized responses.
- Verify Git probes are noninteractive and bounded.
- Verify failed, cancelled, oversized, and checksum-mismatched downloads remove partial files and never replace a valid destination.
- Cover executable permissions, safe-path rejection, temporary-directory cleanup, and JSON/YAML/YML decoding.

## Dependency boundaries

- Check every supported `acorn-core`, `acorn-cmd`, and `acorn-host` feature combination with default features disabled.
- Add dependency-deny checks proving host features do not pull analyzer, schema, database, document, archive, export, or service crates.
- Compile the downstream fixture in CI.
- Require the migrated Xylem CLI graph to contain at least 50% fewer normal package/version nodes than the approximately 671-node baseline.

## Repository validation

- Run `make check-portable`, `make lint`, `make test`, and package verification for ACORN.
- Run Xylem's full unit, integration, and documentation test suite against the updated ACORN revision.
- Record before-and-after dependency trees for both repositories in the implementing change.

# Acceptance criteria

- Xylem no longer depends on `acorn-lib` or enables the `analysis` feature.
- Xylem uses ACORN-owned APIs for shared filesystem, command, configuration, HTTP/source, download, URI, interpolation, and terminal-label primitives where the contracts above apply.
- Xylem retains ownership of workflow schemas, concurrency, Conda/Pixi/container orchestration, retry policy, and other product-specific behavior.
- `Scheme::SSH` is supported without Xylem's URI parser workaround.
- `acorn-host` features are additive, individually checkable, and do not pull unrelated ACORN subsystems.
- ACORN first-party callers import moved APIs from their owning crates, and `acorn-lib` does not retain compatibility re-exports.
- The Xylem dependency graph meets the 50% reduction target while both repositories' relevant tests pass.

# Related plans

- [Crate boundary architecture](../crate-boundary-architecture.md)
- [acorn-cmd extraction](../acorn-cmd-extraction.md)
- [Dependency feature defaults](dependency-feature-defaults.md)
- [Host abstraction traits](host-abstraction-traits.md)
- [Portability roadmap](roadmap.md)

