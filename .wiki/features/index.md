# Feature Plans

## In progress

- [Model acquisition and metadata hardening](model-acquisition-hardening.md) - Complete identity-safe Hugging Face requests, exact GGUF metadata, and ModelKit/ModelPack interoperability evidence.
- [Needle routing and native capability parity](needle-router-native-parity.md) - Bring bounded native ACORN workflows behind `acorn ask` with measured parity.
- [PowerApp and Power Automate JSONL intake](powerapps-jsonl-intake/index.md) - SharePoint is the standard-connector manual path: the owner three-row stage/inspect/apply/report demo passed. PA-22 retains SharePoint participant acceptance; PA-19 storage/catalog and PA-20 intake/export acceptance passed October 26; PA-21 retains browser edge cases. New PA-26 repairs saved exporter failure handling in isolation, then PA-23 accepts collected snapshots. PA-24 covers native restore; PA-25 optional cutover; PA-12 shared deployment. GitLab PA-15–17 remain independent; both pilots stay enabled.

## Planned

- [SharePoint JSONL download for ACORN intake](sharepoint-jsonl-download/index.md) - Add reusable OAuth 2.0/Entra browser/device authorization with ACRE auth profiles, verified account identity, scoped cache inspection/reset, cross-platform protected caching, existing Confirm controls and Graph file capture, consumed by SharePoint direct-URL intake and offline review; approved CLI tenant access is unverified, and both operational pilots remain unchanged.

- [Headless CI SharePoint JSONL download](sharepoint-headless-ci-download/index.md) - Follow SPD-26 shared functionality and acceptance on any one native OS with GitLab OIDC federation, ACRE workload profiles and read-only unattended capture/staging; seven bounded phases require approved app-only access and isolated live acceptance. No user credential cache or automatic apply/reporting.

- [Installable acornd background service](acornd-service.md) - Serve existing APIs and configured check/gather jobs from a persistent local or SSH-managed service on Linux, macOS, and Windows; release requires signed paired binaries and live host acceptance.
- [Automated JSONL management with the Vergil GitLab bot](vergil-gitlab-jsonl-automation/index.md) - PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to their prerequisites. Service deployment, callback integration, reporting setup and authority decisions remain planned/blocked. This GitLab-only track is independent of SharePoint manual reporting and adds no bridge or list cleanup. Implemented runtime contracts await live deployment and end-to-end acceptance: protected runtime, callbacks, durable admission/cleanup, human review/merge and scheduled reporting.

- [Sponsor highlights clarity](sponsor-highlights-clarity.md) - Add evidence-backed mission value, a consistent project synopsis, reviewed ASPECT graduation across schema, local CLI and other adapters, accessibility checks, and an approved distribution pilot.
- [Grouped report separators](grouped-report-separators.md) - Extend the existing local --combine PDF with ordered project groups and full-page office dividers; keep individual report publication unchanged.
- [RON support for ACRE configuration](acre-ron-configuration.md) - Add `.acorn.ron` loading, checking, persistence, and canonical formatting with the existing ACRE data model and discovery precedence.
- [Interactive metadata chat and terminal prompts](interactive-metadata-chat.md) - Add inline multi-turn `ask`, durable transcripts, Gum-like Confirm buttons within reusable terminal controls, and interactive enrichment feedback for `format`.
- [Generic ask executor adapters](ask-executor-adapters.md) - Add configured MCP, ACP, endpoint, and local-sidecar bindings after router contracts land; release follows native parity.
- [Swift SLM capability profile](swift-slm-profile.md) - Release Tongue, Redact, and Title after licensing, host, artifact, and quality gates.
- [Julia decision sidecar profile](julia-decision-sidecar.md) - Release advisory Choice and Noul decisions after independent parity and quality gates.
- [Release-gated Beam inference integration](beam-inference-integration.md) - Verify released serving contracts, then evaluate an explicitly configured hosted or operator-owned endpoint independently of local SLM sidecars.

- [Database-backed artifact generation](database-backed-artifact-generation.md) - Generate reproducible current or historical artifacts from canonical database state through existing typed renderers.
- [CEL metadata validation-profile pilot](cel-validation-profile-pilot.md) - Evaluate bounded, optional institutional rules against existing Rust/Rhai behavior before adopting a CEL runtime; see the [OPA/CEL review](../_assets/opa-cel-review.md).
- [dPID research-object support](dpid-research-object-support.md) - Recognize and resolve versioned dPIDs, import resolver RO-Crate metadata into RAD, and preserve manifest provenance through a read-only first release.
- [Deterministic and assisted gather extraction](gather-extraction.md) - Add bounded CSS extraction first, then an explicitly selected typed inference fallback for unresolved required fields.
- [Azure Cosmos DB bucket downloads](azure-cosmos-buckets.md) - After the provider-neutral source prerequisite, add a Cosmos DB for NoSQL container or logical partition as a read-only bucket source.

## Coordination

- [Ask routing and sidecar coordination](slm-sidecars.md) - Orders the four independently executable plans without owning phases.
