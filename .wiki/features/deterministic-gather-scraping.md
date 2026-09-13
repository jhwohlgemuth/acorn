---
type: Plan
title: "Deterministic gather scraping"
description: "Extend gather with bounded HTTP fetching and typed CSS extraction without scripting or inference."
difficulty: medium
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [feature, gather, scraping, html, css, http]
status: planned
---

# Project code boundaries

- **Update:** Workspace and `acorn-lib` manifests, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/gather/mod.rs`, `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, focused HTTP policy integration, dedicated tests, documentation, and generated help.
- **Create:** `crates/acorn-lib/src/scrape/{mod,fetch,extract,config}.rs` and fixtures under `tests/fixtures/scrape/`.
- **Delete:** None.

# Dependencies

This plan consumes the existing HTTP/offline and gather discovery boundaries. It is independent of [constrained Rhai scripting](rhai-scripting.md). [Structured extraction inference](structured-extraction-inference.md) may consume its normalized extraction request and provenance after this plan lands.

# Goal

Allow `acorn gather` to fetch multiple HTTP(S) inputs under explicit transport limits and deterministically map CSS selections into validated RAD candidate fields.

# Current evidence

- `gather` accepts files, directories, URLs, and text and already performs identifier discovery, but it has no HTML extraction configuration.
- ACORN has shared HTTP and offline behavior, while the existing `--threads` setting controls Rayon CPU work rather than asynchronous fetch concurrency.
- HTML parsing dependencies used by templates do not establish a scraping policy or field-mapping contract.

# Decisions

- Add `--scrape` and repeatable `--selector FIELD=CSS` arguments. Split on the first `=` so CSS pseudo-classes, attribute values, and URL-like values may contain colons.
- Validate `FIELD` against a documented RAD candidate mapping. Define match cardinality, document order, whitespace normalization, text versus attribute extraction, duplicate handling, and required-field behavior.
- Parse the original response for deterministic extraction. Sanitization is not part of this plan because no content is sent to a model or rendered as trusted markup.
- Use bounded asynchronous concurrency with a scraping-specific limit; do not run network I/O on Rayon or silently reinterpret the global CPU thread count.
- Reuse the existing offline policy. Do not introduce scripting-oriented `allow_fs` or claim that `allow_net` already exists.
- Define maximum response bytes, accepted content types and encodings, decompression bounds, redirect count and destination checks, timeout, private/link-local address policy, credential forwarding, cache partitioning, and diagnostic redaction before enabling remote fetches.
- Record source URI, final URI, retrieval time, selector mapping, and extracted field provenance. Validate every candidate before persistence.
- Add one optional `scrape` feature with its exact HTTP/parser dependencies. Do not enable `percy-dom`, `ammonia`, or a direct `selectors` dependency unless the implementation demonstrates a specific need.

# Validation

- Unit-test selector mapping, CSS values containing colons and equals signs, zero/one/many matches, attribute extraction, normalization, candidate validation, and stable ordering.
- Test offline rejection, redirects to denied destinations, private-network policy, oversized/decompression-bomb responses, invalid content types, timeouts, credential stripping, and bounded concurrency with a local fake server.
- Run the scrape-only, default, all-feature, no-default-feature, and WASM checks; inspect feature trees; run lint, focused tests, the full suite, help generation, and dependency/license checks.

# Acceptance criteria

- `acorn gather URL... --scrape --selector FIELD=CSS` fetches with bounded async concurrency and produces deterministically ordered, validated candidates.
- Selector syntax supports valid CSS containing colons and reports malformed or unknown field mappings before any fetch.
- Offline, transport, size, redirect, SSRF, credential, and content-type policies fail closed with actionable errors.
- Every extracted value retains source and selector provenance.
- No Rhai, inference, sidecar, MCP, JSON-RPC, workflow-hook, or TUI scripting behavior is introduced.
- Scraping dependencies remain absent unless the `scrape` feature is enabled.

# Follow-on plan

[Structured extraction inference](structured-extraction-inference.md) may add a best-effort fallback only after deterministic extraction and its security boundary pass these acceptance criteria.
