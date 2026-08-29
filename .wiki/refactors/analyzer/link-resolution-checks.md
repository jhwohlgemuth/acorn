---
type: Plan
title: "Unify website and resolution checks"
description: "Route URL reachability and persistent-identifier resolution outcomes through one link-analysis contract."
difficulty: medium
impact: medium
package: [acorn-lib, acorn-cli]
tags: [refactor, analyzer, links, resolution, offline]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/analyzer/{check,service}.rs`, discovery/resolution producers, `io/http/`, gather/check CLI consumers, and focused tests.
- **Create:** shared link evidence/evaluation contracts in `acorn-analysis`, orchestration in `acorn-service`, and native evaluator implementation in `acorn-host` after extraction.
- **Delete:** direct provider-to-`Check` conversion and duplicate resolution streams after all producers use the shared contract.

# Dependencies

Use [check options context](check-options-context.md), [check batch monoid](check-batch-monoid.md), and consumer-owned host traits. This plan supplies the portable/native seam used by SWHID and other resolver features.

# Goal

Represent website reachability and persistent-identifier resolution as one link-analysis capability that produces `CheckCategory::Link` results through the existing structured analysis report.

# Current evidence

- `crates/acorn-lib/src/analyzer/check.rs` dispatches `CheckCategory::Link` to the path-oriented `Analysis::check_websites` method; the former standalone `Analysis::check_resolution` hook has been removed.
- `crates/acorn-lib/src/analyzer/service.rs` places website results in `AnalysisReport`, but skips the complete link category when either `offline` or `disable_website_checks` is set.
- `DiscoveryRecords::analyze` attaches completed resolution lifecycle checks to `AnalysisReport` as `CheckCategory::Link` results after path analysis, so offline and website-disable policy does not discard already available outcomes. Gather consumes the flattened report without appending a separate resolution-check stream.
- Remote provider matches still convert completed resolution outcomes directly during report construction, so the shared evidence/evaluator contract and complete remote migration remain unfinished.
- Gather disables website checks before calling `analyze_paths`. A direct merger of the existing skip conditions would incorrectly suppress checks derived from resolution outcomes that are already available and require no network access.
- Resolved raw gather output consumes stored DOI and ORCID outcomes for citation and person-name rendering, including remote OSTI matches. Remote search first maps provider and local data, then `--resolve` gates person, project, repository, and link-liveness requests. DOI citation style precedence is `--citation-format`, `ACORN_CITATION_FORMAT`, then IEEE. The unified link contract must preserve configuration and stderr diagnostics rather than treating resolution as reachability alone.

# Recommendation

Introduce one library-owned link-analysis contract and evaluator for both URL targets and persistent-identifier resolution outcomes. The contract should distinguish network operations from evaluation of existing outcomes rather than treating every link check as network-dependent.

Model the minimum shared evidence needed to create a `Check`: target or identifier, locator, source URI, optional status, and optional diagnostic metadata. Website implementations contribute unresolved URL targets; gather contributes completed or failed identifier-resolution outcomes. The evaluator performs allowed network work and converts every result into `CheckCategory::Link` values.

Replace `Analysis::check_websites` and `Analysis::check_resolution` with one clearly named link-check entry point. Prefer a focused composable trait or input type over retaining a general `Analysis` implementation whose other methods are no-ops. Keep standard-specific parsing at the boundary and move common request gating, outcome conversion, and check construction into the shared evaluator.

Route the unified results through `CategoryChecks` and `AnalysisReport`. Gather should consume the report output instead of manually chaining resolution checks. Preserve its structured report serialization, failure calculation, TUI behavior, database persistence, configured DOI citations, ORCID display names, normalized fallbacks, and detailed raw-mode stderr failures.

Separate policy flags during migration:

- `offline` prevents new network requests but does not discard checks generated from existing failed resolution outcomes.
- Website-disable behavior suppresses URL reachability requests without suppressing local resolution-outcome checks.
- `--resolve` remains the explicit authorization for gather to contact PID metadata providers.

Retain the current CLI flag as a compatibility boundary unless a separately documented CLI migration renames it. Internal names may become link-oriented once their semantics are explicit.

# Implementation sequence

1. Add the shared link evidence and outcome types with pure conversion tests for missing URLs, reachable and unreachable URLs, successful resolutions, failed resolutions, and unsupported identifiers.
2. Add one link evaluator that applies offline and website-disable policy before scheduling network work, then returns checks without printing or exiting.
3. Migrate standard-specific `check_websites` implementations to emit shared link evidence and remove duplicated link-result construction.
4. Migrate `DiscoveryRecords` to emit resolution evidence, remove its standalone `check_resolution`, and remove its no-op `Analysis` implementation if the focused link contract makes it unnecessary.
5. Integrate unified link checks into `AnalysisReport` and update gather to rely on the structured category output.
6. Remove obsolete methods, imports, and the CLI `async-trait` dependency if it no longer has another direct use.

# Validation

- Unit-test evidence-to-check conversion and keep new `test_*` functions alphabetically ordered.
- Test that offline analysis performs no website or PID network request while still reporting a supplied failed resolution outcome.
- Test website-disable behavior independently from offline behavior.
- Test gather with resolution disabled, successful resolution, failed resolution, unsupported identifiers, file input, literal text, and offline input.
- Test resolved raw DOI styles, CLI/environment/default style precedence, ORCID names, OSTI outcomes, normalized fallbacks, stderr diagnostics, and the absence of optional online enrichment without `--resolve` through the unified contract.
- Verify CLI and TUI gather output retain the same discovery, failure, and source fields.
- Verify discovered records and resolution metadata continue to persist through the existing database integration.
- Run `cargo fmt --all`, clippy for all affected targets with warnings denied, focused analyzer and gather tests, and `make test`.

# Acceptance criteria

- One link-analysis entry point replaces both `check_websites` and `check_resolution`.
- Website and PID-resolution checks are represented as `CheckCategory::Link` values in the structured analysis report.
- Gather does not manually append resolution checks outside the analysis report.
- Offline mode prevents network access without hiding checks derived from existing resolution outcomes.
- Website-disable behavior does not suppress resolution-outcome checks.
- Remote providers obtain and locally enrich data before optional resolution and link-liveness checks.
- `DiscoveryRecords` no longer implements unrelated analysis methods as no-ops.
- CLI output, citation-format precedence, TUI output, failure status, and database persistence remain behaviorally compatible.
- No obsolete direct dependency remains after the trait migration.

# Related concepts

- [Check batch monoid](check-batch-monoid.md)
- [Check options context](check-options-context.md)
- [Standards traversal](standards-traversal.md)
