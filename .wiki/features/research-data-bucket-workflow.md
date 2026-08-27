---
type: Plan
title: "Integrate the research data workflow with configured buckets"
description: "Connect the workflow and OpenAlex enrichment provider to configured bucket discovery, previews, and CLI application."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [research-activity, buckets, workflow, enrichment, openalex, linked-data]
status: in-progress
---

# Goal

Connect configured bucket discovery and CLI composition to the existing research data workflow and OpenAlex enrichment provider. Add provider robustness, deterministic previews, explicit apply authorization, and stable source identity without moving provider policy into the workflow engine.

# Current evidence

- `crates/acorn-lib/src/io/workflow/mod.rs` now defines `workflow::Workflow`, `workflow::Options`, `workflow::Source`, a provider-neutral `workflow::Enrichment` capability, immutable planning, dry-run application, and reports.
- `crates/acorn-lib/src/io/workflow/tests/mod.rs` verifies default checking, non-mutating planning, dry-run behavior, injected output enrichment, linked artifacts, and error handling for unavailable or offline enrichment.
- `crates/acorn-lib/src/schema/research_activity/output.rs` now provides a structured `ResearchOutput` target for publications, datasets, software, and other activity products, including work-level contributors, affiliations, funding, awards, locations, keywords, and access metadata.
- `crates/acorn-lib/src/io/api/openalex.rs` now provides a typed DOI work lookup and implements the workflow enrichment capability. OpenAlex supports direct work lookup by external DOI and returns structured work, authorship, location, and open-access metadata ([API overview](https://developers.openalex.org/api-reference/introduction), [single-work reference](https://developers.openalex.org/api-reference/works/get-a-single-work)).
- OpenAlex now recommends an API key for meaningful usage and accepts it as an `api_key` query parameter or bearer token; ACORN stores the relevant environment-variable names in `acorn::util::constants::env` ([authentication and pricing](https://developers.openalex.org/api-reference/authentication)).
- CiteAs remains integrated as a citation lookup used by discovery and GitLab intake. It does not currently expose the richer work graph needed by the structured output model.
- The CLI commands still discover and mutate files independently; configured bucket enumeration and workflow composition have not yet been routed through the library workflow.

# Recommendation

Keep `acorn::io::workflow` as the composition boundary for the current workspace, while preserving its injected capability shape for the eventual `acorn-service` boundary described by the crate architecture plan. Treat bucket files as the source of truth and use any database records as indexes, cache entries, and provenance rather than as an independent writable authority.

Use the stage order `read -> check -> format -> enrich -> format -> check -> link -> plan -> apply`. Validation and planning happen before writes. Enrichment remains opt-in because it performs network requests, consumes an external API budget, and can introduce provider assertions that users should preview. Checking remains enabled by default; formatting, enrichment, linking, and application are explicit.

Use the implemented OpenAlex provider as the first structured DOI enrichment provider, not as a transparent replacement for CiteAs. The current provider creates missing `ResearchOutput` values and preserves an existing output with the same normalized DOI; field-level merging and conflict provenance remain planned. CiteAs should continue to resolve rendered citation text until it implements the same workflow capability or is intentionally retired after coverage comparison. Provider selection belongs at the CLI composition root, not inside the workflow engine.

Next, add bucket source discovery and CLI composition without moving provider policy into `workflow`. The CLI should resolve configured bucket paths into `workflow::Source` values, select zero or one enrichment provider, call `Workflow::plan`, render a deterministic change summary, and call `Workflow::apply` only when authorized. Add merge identity, conflict reporting, provenance, and cache policy before enabling repeated unattended enrichment.

# Validation

- Add fixture-backed bucket discovery tests for local and remote bucket layouts, mixed supported file formats, invalid data, empty buckets, and duplicate source keys.
- Add mock HTTP coverage for OpenAlex success, not-found, authentication, rate-limit, malformed response, and retry behavior without relying on live network access.
- Add CLI integration tests for preview, dry-run, explicit apply, offline rejection, provider selection, partial source failure, and deterministic reruns.
- Run `make lint`, `make test`, and `make wasm` after CLI and bucket integration are complete.

# Acceptance criteria

- Bucket discovery produces stable `workflow::Source` identity from bucket and relative object key.
- OpenAlex failures use stable diagnostics for not-found, authentication, rate-limit, malformed-response, and retry exhaustion cases.
- Repeated enrichment merges compatible provider fields, reports populated-value conflicts, and records enough provenance to explain every external assertion.
- Cache and partial-provider failure policies produce deterministic reruns without silently discarding valid existing output metadata.
- CLI output clearly distinguishes checked, changed, enriched, linked, skipped, and failed sources before apply.
