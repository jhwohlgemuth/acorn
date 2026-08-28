---
type: Plan
title: "Integrate research data workflow enrichment with format and gather"
description: "Connect ordered metadata enrichment providers to format previews and gathered research activity context."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [research-activity, workflow, enrichment, openalex, orcid, ror, linked-data]
status: complete
---

# Goal

Make `acorn format --enrich` the primary enrichment interface and let `acorn gather --enrich` enrich gathered RAD documents for more complete analysis context. Configured buckets remain transfer sources that users download or copy before running local workflows.

# Current evidence

- `crates/acorn-lib/src/io/workflow/mod.rs` now defines `workflow::Workflow`, `workflow::Options`, a provider-neutral `workflow::Enrichment` capability over the existing `acorn::Location`, immutable planning, dry-run application, and reports. Planned changes can also retain analyzer `Check` values as inputs to later automated fixes.
- `crates/acorn-lib/src/io/workflow/tests/mod.rs` verifies default checking, non-mutating planning, dry-run behavior, injected output enrichment, linked artifacts, and error handling for unavailable or offline enrichment.
- `crates/acorn-lib/src/schema/research_activity/output.rs` now provides a structured `ResearchOutput` target for publications, datasets, software, and other activity products, including work-level contributors, affiliations, funding, awards, locations, keywords, and access metadata.
- `crates/acorn-lib/src/io/api/openalex.rs` now provides a typed DOI work lookup and implements the workflow enrichment capability. OpenAlex supports direct work lookup by external DOI and returns structured work, authorship, location, and open-access metadata ([API overview](https://developers.openalex.org/api-reference/introduction), [single-work reference](https://developers.openalex.org/api-reference/works/get-a-single-work)).
- OpenAlex now recommends an API key for meaningful usage and accepts it as an `api_key` query parameter or bearer token; ACORN stores the relevant environment-variable names in `acorn::util::constants::env` ([authentication and pricing](https://developers.openalex.org/api-reference/authentication)).
- `crates/acorn-lib/src/io/enrichment.rs` composes OSTI, OpenAlex, CiteAs, ORCID, and ROR in deterministic order, continues after per-identifier provider failures without discarding earlier successes, preserves existing values, and records metadata-level field provenance.
- OpenAlex merges missing structured metadata into an existing output with the same DOI instead of treating identifier presence as a fully enriched record.
- `acorn format --enrich` uses the workflow planner, supports repeated or comma-delimited `--provider`, honors `--dry-run` for the combined enrichment and formatting change, and rejects offline enrichment before any formatting write.
- `acorn gather --enrich` enriches valid RAD inputs in memory before discovery; `--raw --enrich` emits enriched candidates as JSON lines and never rewrites source files.
- With no path, both commands search the current directory. They do not enumerate configured buckets because bucket configuration represents data intended to be downloaded or copied first.

# Recommendation

Keep `acorn::io::workflow` as the composition boundary for the current workspace, while preserving its injected capability shape for the eventual `acorn-service` boundary described by the crate architecture plan. Treat bucket files as the source of truth and use any database records as indexes, cache entries, and provenance rather than as an independent writable authority.

Use the stage order `read -> check -> format -> enrich -> format -> check -> link -> plan -> apply`. Validation and planning happen before writes. Enrichment remains opt-in because it performs network requests, consumes an external API budget, and can introduce provider assertions that users should preview. Checking remains enabled by default; formatting, enrichment, linking, and application are explicit.

Use OpenAlex for structured scholarly outputs and CiteAs as the fallback DOI metadata source. OSTI contributes DOE CODE metadata, while ORCID and ROR fill missing contact and organization identifiers only after exact matching and disambiguation. Provider selection belongs at the CLI composition root, not inside the workflow engine.

Keep provider policy in the shared enrichment pipeline rather than in the workflow engine. Future work should add provider-result caching and richer field-level conflict diagnostics before enabling unattended enrichment at scale.

# Validation

- Add mock HTTP coverage for OpenAlex success, not-found, authentication, rate-limit, malformed response, and retry behavior without relying on live network access.
- CLI tests cover enrichment dry-run immutability, offline rejection without writes, and gathered JSONL enrichment without live network access.
- OpenAlex unit tests cover merging missing fields into matching DOI outputs and retaining an earlier success after a later lookup failure.
- `make lint` and `make wasm` pass. The full 1,312-test nextest run passes 1,311 tests; the unrelated retry-resume timeout passes when rerun alone.

# Acceptance criteria

- [x] `format --enrich` applies providers in deterministic order and supports explicit provider selection.
- [x] `format --enrich --dry-run` previews the combined change without writing.
- [x] `gather --enrich` enriches RAD input in memory and `--raw --enrich` emits JSON lines.
- [x] ORCID and ROR use exact matching and disambiguation rather than selecting the first result.
- [x] Existing output values win, missing fields are enriched by DOI, and per-identifier provider failures do not discard successful contributions.
- [x] Offline format enrichment fails before source files are rewritten.
- [x] Enriched field paths and lookup sources are recorded in RAD metadata provenance.
- [x] No standalone `acorn enrich` command or configured-bucket enumeration is introduced.
