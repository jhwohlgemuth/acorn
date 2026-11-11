# Feature Plans

> **Sanitized copy:** Account labels and related assets have been redacted. Historical checksums and import results describe the originals; see [privacy and reuse limits](../privacy.md).

## In progress

- [Needle routing and native capability parity](needle-router-native-parity.md) - Native routes cover 46/46 bounded rows; PAR-34's synthetic safety matrix is accepted and PAR-16 is ready to release explicit dispatch. Automatic Needle selection remains preview-only after failed calibration.

## Planned

- [Interactive metadata chat and terminal prompts](interactive-metadata-chat.md) - Inline multi-turn `ask`, durable transcripts, reusable terminal controls, and interactive format review are in place; the TUI consumer seam, multi-select control, and release validation remain.
- [SharePoint JSONL download for ACORN intake](sharepoint-jsonl-download/index.md) - Complete reusable OAuth 2.0/Entra browser/device authorization with ACRE auth profiles, verified account identity, scoped cache inspection/reset, cross-platform protected caching, existing Confirm controls and Graph file capture, consumed by SharePoint direct-URL intake and offline review; approved CLI tenant access is unverified, and it requires no operational app/flow changes.
- [Model acquisition and metadata hardening](model-acquisition-hardening.md) - Complete identity-safe Hugging Face requests, exact GGUF metadata, and ModelKit/ModelPack interoperability evidence.
- [RON support for ACRE configuration](acre-ron-configuration.md) - Dependency/format registration and the private data adapter are landed; complete configuration lifecycle, CLI discovery, canonical formatting, and final documentation remain.

- [Power Automate Teams workflow bot](powerautomate-teams-bot/index.md) - Package a Workflows-authored Teams assistant as solution-aware flows with Dataverse sessions, requests, jobs, idempotent effects and receipts. The owner-only summarize request/reply slice is repaired and accepted; remaining phases decide the supported Teams interaction surface, then add ALM, durable routing, centralized responses, bounded capabilities, recovery and governed pilot release. The Sponsor Highlights intake and Vergil automation plans retain their own systems.

- [Intake approval pilot acceptance and cutover](powerapps-jsonl-intake/approval-release/index.md) - Join both consistent apps, review ledger, approved export and actual ACORN PDF in isolated acceptance before an explicitly authorized operational cutover.
- [Shared Power Platform intake deployment](powerapps-jsonl-intake/shared-deployment/index.md) - After PA-41A and target readiness, import the accepted solution into a named shared environment, accept it in isolation, then switch future links and notifications with rollback.

- [Headless CI SharePoint JSONL download](sharepoint-headless-ci-download/index.md) - Follow SPD-26 shared functionality and acceptance on any one native OS with GitLab OIDC federation, ACRE workload profiles and read-only unattended capture/staging; seven bounded phases require approved app-only access and isolated live acceptance. No user credential cache or automatic apply/reporting.

- [Installable acornd background service](acornd-service.md) - Build on the landed portable job and strict daemon configuration contracts to serve existing APIs and configured check/gather jobs from a persistent local or SSH-managed service; SQLite history and composable server lifecycle are ready, and release requires signed paired binaries plus live host acceptance.
- [Automated JSONL management with the Vergil GitLab bot](vergil-gitlab-jsonl-automation/index.md) - PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to their prerequisites. Service deployment, callback integration, reporting setup and authority decisions remain planned/blocked. This GitLab-only track is independent of SharePoint manual reporting and adds no bridge or list cleanup. Implemented runtime contracts await live deployment and end-to-end acceptance: protected runtime, callbacks, durable admission/cleanup, human review/merge and scheduled reporting.

- [Sponsor highlights clarity](sponsor-highlights-clarity.md) - Add evidence-backed mission value, a consistent project synopsis, reviewed ASPECT graduation across schema, local CLI and other adapters, accessibility checks, and an approved distribution pilot.
- [Grouped report separators](grouped-report-separators.md) - Extend the existing local --combine PDF with ordered project groups and full-page office dividers; keep individual report publication unchanged.
- [Swift SLM capability profile](swift-slm-profile.md) - Release Tongue, Redact, and Title after licensing, host, artifact, and quality gates.
- [Julia decision sidecar profile](julia-decision-sidecar.md) - Release advisory Choice and Noul decisions after independent parity and quality gates.
- [Release-gated Beam inference integration](beam-inference-integration.md) - Verify released serving contracts, then evaluate an explicitly configured hosted or operator-owned endpoint independently of local SLM sidecars.

- [Database-backed artifact generation](database-backed-artifact-generation.md) - Generate reproducible current or historical artifacts from canonical database state through existing typed renderers.
- [Rust browser and Tauri desktop workspace for ACORN tools](acorn-browser-workspace/index.md) - Build one accessible Dioxus workspace with a loaded Research Activity portfolio, hot-reloadable isolated Resium/CesiumJS ROR map, and least-privilege Tauri shell while preserving the existing portable, `acorn-wasm`, `acorn-host`, and `acorn-lib` ownership boundaries.
- [CEL metadata validation-profile pilot](cel-validation-profile-pilot.md) - Evaluate bounded, optional institutional rules against existing Rust/Rhai behavior before adopting a CEL runtime; see the [OPA/CEL review](../_assets/opa-cel-review.md).
- [dPID research-object support](dpid-research-object-support.md) - Recognize and resolve versioned dPIDs, import resolver RO-Crate metadata into RAD, and preserve manifest provenance through a read-only first release.
- [Deterministic and assisted gather extraction](gather-extraction.md) - Add bounded CSS extraction first, then an explicitly selected typed inference fallback for unresolved required fields.
- [Azure Cosmos DB bucket downloads](azure-cosmos-buckets.md) - After the provider-neutral source prerequisite, add a Cosmos DB for NoSQL container or logical partition as a read-only bucket source.

## Waiting on validation

- [Retained GitLab intake fallback hardening](powerapps-jsonl-intake/gitlab-fallback-hardening/index.md) - Complete isolated byte-boundary, concurrent-write and native-restore acceptance for the retained Off GitLab route; every remaining phase is blocked on authorized disposable systems or connector access, and passing evidence does not authorize restoration.

## Planned with no ready phase

- [Generic ask executor adapters](ask-executor-adapters.md) - Reusable local sidecar registration and configured CLI composition are landed with MCP, ACP, and endpoint bindings; all remaining phases wait for PAR-16 native parity acceptance.

## Epics

- [PowerApp and Power Automate intake portfolio](powerapps-jsonl-intake/index.md) - Coordinates the accepted SharePoint baseline, report-first approval plans, retained GitLab fallback, shared deployment and boundaries with downloader, Teams, Vergil, editorial and enrichment work.

## Coordination

- [Ask routing and sidecar coordination](slm-sidecars.md) - Orders the four independently executable plans without owning phases.
