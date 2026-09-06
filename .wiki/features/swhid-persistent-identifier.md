---
type: Plan
title: "Harden SWHIDs across the persistent-identifier lifecycle"
description: "Harden the implemented Software Hash Identifier lifecycle with conformance, resolver, filesystem, and repository diagnostics."
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, pid, swhid, software-heritage, software, discovery, resolution, cff, schema]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-schema/src/schema/pid/swhid/{mod.rs,tests.rs}`, `crates/acorn-lib/src/io/{swhid.rs,api/swhid.rs}`, `crates/acorn-lib/src/analyzer/host/discovery/`, `crates/acorn-cli/src/commands/swhid/mod.rs`, documentation, and focused tests.
- **Create:** pinned conformance/differential vectors and resolver/filesystem/repository fixtures. Future package extraction may move protocol DTOs to `acorn-api` and native calculation/resolution to `acorn-host`, but that is not part of this plan's current package boundary.
- **Delete:** no public SWHID surface; remove duplicated resolver-to-check conversion only after the shared link contract owns it.

# Dependencies

Coordinate resolver outcomes with [analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md) and preserve the portable schema boundary from [crate extraction](../refactors/crate-boundary-architecture.md).

# Goal

Finish hardening ACORN's first-class Software Hash Identifier (SWHID) support against the [ISO/IEC 18670:2025 standard](https://www.iso.org/standard/89985.html) and the [canonical SWHID v1.2 specification](https://www.swhid.org/specification/v1.2/).

# Current evidence

- The first-class lifecycle is implemented: portable parsing and discovery live in `crates/acorn-schema/src/schema/pid/swhid/`; RAD storage and CFF export are covered in `crates/acorn-schema/src/schema/research_activity/`; opt-in discovery resolution calls `crates/acorn-lib/src/io/api/swhid.rs`; local file, directory, and Git calculation lives in `crates/acorn-lib/src/io/swhid.rs`; and `acorn swhid` exposes calculation and verification.
- Parser tests cover all five core object kinds, canonical qualifier ordering, representative valid and invalid qualifiers, resolver URL normalization, and idutils parity. They are not pinned SWHID specification vectors and do not differentially compare the official implementation.
- Resolver behavior has no provider-specific fixture suite for status codes, malformed bodies, timeouts, or transport failures.
- Calculation tests cover one content vector and a normal SHA-1 Git repository without mutation. They do not cover cross-platform directory semantics, special files, repository failure states, or explicit SHA-256 repository rejection.

The feature itself is complete enough for public use, but this hardening plan remains active because its narrower acceptance criteria are not met. Reconcile or delete it only after the four remaining-work bullets below are completed or deliberately descoped.

# Remaining work

- Expand the parser and calculation conformance matrix with pinned public specification vectors and differential checks against the official implementation.
- Add fixture-backed Software Heritage resolver tests for success, 400, 401/403, 404, 429, malformed JSON, timeout, and transport failure. Confirm that no request occurs without `--resolve` or in offline mode.
- Add cross-platform directory-calculation vectors covering executable permissions, symlinks, non-UTF-8 names where supported, duplicate manifest names, and special-file rejection.
- Detect SHA-256 Git repositories before traversal and return an explicit unsupported-format diagnostic. Retain coverage for lightweight tags, missing or dangling objects, and ambiguous references.

# Validation

- Run the pinned SWHID conformance and differential tests without making the reference implementation a portable runtime dependency.
- Run focused PID, discovery, schema, CFF, resolver, calculation, and CLI tests.
- Run `cargo fmt --all`, `make lint`, and `make test`.
- Run `cargo check -p acorn-lib --no-default-features`, the alloc feature check, and `make check-wasm`.
- Run `make audit` if the remaining work changes dependencies.

# Acceptance criteria

- Public conformance vectors cover every supported SWHID object and qualifier form, plus invalid syntax and qualifier combinations.
- Resolver behavior is status-specific, fixture-tested, opt-in, and disabled by offline mode.
- Directory identifiers have documented, tested permission and symlink behavior across supported platforms.
- Unsupported Git object formats and repository states fail with actionable diagnostics without mutating repositories or working trees.
- Default, minimal, alloc, and WASM builds remain supported.

# Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Resolver behavior changes or is rate-limited. | Medium | Keep resolution optional and cover status-specific behavior with local fixtures. |
| Filesystem metadata yields platform-dependent identifiers. | High | Use explicit permission and symlink policies plus Unix and Windows vectors. |
| Unsupported Git formats are misidentified as SHA-1 repositories. | High | Detect the object format before traversal and reject unsupported formats explicitly. |

# Related concepts

- [Persistent identifier lifecycle](../../docs/src/concepts/persistent-identifiers.md)
- [Analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [API provider capability matrix](../refactors/api/provider-capability-matrix.md)
