---
type: Plan
title: Resolve workspace dependency audit findings
description: Triage and remediate the unignored RustSec findings in the workspace lockfile, then restore the dependency audit gate.
difficulty: high
impact: high
package: [acorn-lib, acorn-schema, acorn-server]
tags: [security, dependencies, audit]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Restore a passing `make audit` for the workspace. Own dependency and lockfile remediation separately from completed feature plans. Preserve supported behavior and feature boundaries while upgrading dependencies.

# Current evidence

On November 11, 2026, `cargo audit --no-fetch --json` reported 39 vulnerability entries in `Cargo.lock`; 38 remain after excluding `RUSTSEC-2023-0071`, which `make audit` already ignores. They span 18 package names: `ammonia`, `calamine`, `crossbeam-epoch`, `crypto-bigint`, `libsqlite3-sys`, `libz-sys`, `lopdf`, `pulldown-cmark`, `quinn-proto`, `rmcp`, `rustls`, `rustls-webpki`, `webpki-root-certs`, `webpki-roots`, `zip`, `zstd`, `zstd-safe`, and `zstd-sys`. No finding names `ron`. The scan used the locally cached advisory database; refresh it during triage.

The prior RON validation run passed dependency-usage analysis but failed the advisory scan. A November 11 `make audit` retry began rebuilding dependency-usage analysis and was interrupted after the independent lockfile scan confirmed the blocker; it is not a passing full audit.

The workspace directly declares `rmcp` 1.x and `rustls` in `Cargo.toml`, `ammonia` in `crates/acorn-schema/Cargo.toml`, and `zip` and `zstd` in `crates/acorn-lib/Cargo.toml`. The locked `rmcp` 1.7.0 and `zip` 6.0.0/8.6.0 findings require compatibility review because the advisory database lists fixes at `rmcp` 3.9.0 and `zip` 9.0.0. Other findings may be resolvable within compatible version ranges. An advisory finding in the lockfile does not by itself establish that a vulnerable API is reachable in a particular feature or target.

# Boundaries and ownership

- The workspace dependency owner holds the Workspace graph integration lock for manifest and `Cargo.lock` changes. Coordinate with active plans before claiming it.
- Group compatible updates separately from API migrations. Determine direct and transitive parents, enabled feature paths, affected APIs, and available fixes before editing manifests.
- Keep advisory ignores limited to documented, reviewed exceptions. Do not add an ignore solely to make the gate pass.
- The completed ACRE RON contract and its tests are regression boundaries, not a prerequisite for this plan.

# Acceptance criteria

1. Each current unignored finding has a recorded dependency path, affected feature or target, disposition, and owner.
2. Dependency changes preserve relevant CLI, library, portable/WASM, MCP, and archive behavior through focused checks.
3. A refreshed `make audit` passes, or every remaining exception has an explicit repository-level decision and rationale in the audit policy.
4. The final workspace test and lint gates pass after the dependency graph settles.

# Execution phases

## AUD-01 — Triage the advisory set and assign upgrade groups

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `.wiki/refactors/workspace-dependency-audit.md`
- Primary integration lock: Workspace graph
- Outcome: Refresh advisories, map each finding to its parent dependency and enabled paths, distinguish compatible updates from API migrations, and record exact write sets and owners for bounded remediation phases.
- Focused validation: Run a refreshed `cargo audit` report, `cargo tree -i` for affected crates, and validate the wiki.
- Handoff: Replace this phase with bounded upgrade phases using the verified paths and owners.
