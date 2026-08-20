---
type: Plan
title: "Support SWHIDs through the persistent-identifier lifecycle"
description: "Complete conformance, resolver, filesystem, and repository diagnostics for first-class Software Hash Identifier support."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-schema, acorn-api]
tags: [feature, pid, swhid, software-heritage, software, discovery, resolution, cff, schema]
status: in-progress
---

# Goal

Finish hardening ACORN's first-class Software Hash Identifier (SWHID) support against the [ISO/IEC 18670:2025 standard](https://www.iso.org/standard/89985.html) and the [canonical SWHID v1.2 specification](https://www.swhid.org/specification/v1.2/).

# Remaining work

- Expand the parser and calculation conformance matrix with pinned public specification vectors and differential checks against the official implementation.
- Add fixture-backed Software Heritage resolver tests for success, 400, 401/403, 404, 429, malformed JSON, timeout, and transport failure. Confirm that no request occurs without `--resolve` or in offline mode.
- Add cross-platform directory-calculation vectors covering executable permissions, symlinks, non-UTF-8 names where supported, duplicate manifest names, and special-file rejection.
- Detect SHA-256 Git repositories before traversal and return an explicit unsupported-format diagnostic. Retain coverage for lightweight tags, missing or dangling objects, and ambiguous references.

# Validation

- Run the pinned SWHID conformance and differential tests without making the reference implementation a portable runtime dependency.
- Run focused PID, discovery, schema, CFF, resolver, calculation, and CLI tests.
- Run `cargo fmt --all`, `make lint`, and `make test`.
- Run `cargo check -p acorn-lib --no-default-features`, the alloc feature check, and `make wasm`.
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
- [Website and resolution checks](../refactors/analyzer/link-resolution-checks.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [API provider capability matrix](../refactors/api/provider-capability-matrix.md)
