# Feature Plans

> **Sanitized copy:** Account labels and related assets have been redacted. Historical checksums and import results describe the originals; see [privacy and reuse limits](../privacy.md).

## In progress

- [Rust browser and Tauri desktop workspace for ACORN tools](acorn-browser-workspace/index.md) - The Dioxus browser has consolidated Review tabs, saved Portfolio tile order/visibility controls, and ASPECT/S3A construction routes. ABW-03 is ready for metadata acceptance; ABW-04/05/32 follow its shared result contract, then ABW-06/16 accept cross-engine effects/folders. ABW-19 needs a hosted new-build private-provider recheck before ABW-63–65 multiple-bucket loading and ABW-44/18 project-map work; ABW-72 then adds bounded local CZML drop/picker overlays before final browser acceptance. ABW-69 defines the later official React/shadcn/ui migration handoff. Desktop ABW-45–59 cover the host capability matrix after ABW-11/12/17 and before ABW-56 and native OS acceptance. ABW-70/71 add desktop and hosted-web bot status after the authenticated feed contract. See the [execution handoff](acorn-browser-workspace/execution-preparation.md).

## Planned

- [Browser RADS project templates and editor URL intake](browser-rads-templates.md) - The shared frontend now has three validated starters, a new-tab editor dropdown, project ZIP export, and bounded URL entry. TPL-06 awaits ABW-11 for packaged desktop acceptance.
- [Live bot observability over SSE](sse-bot-observability.md) - Coordination epic for the GitLab bot feed, later observation authentication, desktop dashboard, and authorized hosted-web view.
- [Live GitLab bot status feed](gitlab-bot-live-status.md) - Add a bounded SSE view of bot state first, then authenticate bot observation before desktop or hosted browser connections.
- [Browser and WebMCP metadata formatting parity](browser-format-parity.md) - Ship RAD JSON formatting through one portable WASM function, a tenth read-only WebMCP tool, and `/format`; expand only tested standard/syntax pairs, then reconcile the page with ABW-50 desktop local format.
- [Claude Code ACP agent profile](claude-acp-agent.md) - Add Claude Code beside Codex and OpenCode for `infer run --agent claude`, trusted executor bindings, `acorn doctor`, and the opt-in ACP interoperability test; CLA-01 first records the `claude-agent-acp` adapter's read-only behavior.
- [Optional USWDS browser theme](uswds-browser-theme.md) - The static ACORN component catalog and USWDS appearance are implemented; finish manual accessibility acceptance and retire the plan.
- [Conversational RAD assistance and reviewed fixes](assisted-rad-workspace.md) - After ABW-11 creates the desktop shell, prove CopilotKit/A2UI compatibility and build a small allowlisted assistant display catalog if it passes; add opt-in Ask ACORN, grounded read-only tools, ACORN-backed fix previews, per-fix review, and explicit browser download or native save.
- [Opt-in netrc authentication](netrc-support.md) - Add explicit native credential loading, origin-scoped HTTPS Basic authentication and remote BagIt download flags; preserve provider/OAuth precedence and anonymous defaults. Earthdata compatibility is unproven and outside the initial release claim.
- [Interactive metadata chat and terminal prompts](interactive-metadata-chat.md) - Inline multi-turn `ask`, durable transcripts, reusable terminal controls, and interactive format review are in place; the TUI consumer seam, multi-select control, and release validation remain.
- [SharePoint JSONL download for ACORN intake](sharepoint-jsonl-download/index.md) - Complete reusable OAuth 2.0/Entra browser/device authorization with ACRE auth profiles, verified account identity, scoped cache inspection/reset, cross-platform protected caching, existing Confirm controls and Graph file capture, consumed by SharePoint direct-URL intake and offline review; approved CLI tenant access is unverified, and it requires no operational app/flow changes.
- [Model acquisition and metadata hardening](model-acquisition-hardening.md) - Complete identity-safe Hugging Face requests, exact GGUF metadata, and ModelKit/ModelPack interoperability evidence.

- [Power Automate Teams workflow bot](powerautomate-teams-bot/index.md) - Package a Workflows-authored Teams assistant as solution-aware flows with Dataverse sessions, requests, jobs, idempotent effects and receipts. The owner-only summarize request/reply slice is repaired and accepted; remaining phases decide the supported Teams interaction surface, then add ALM, durable routing, centralized responses, bounded capabilities, recovery and governed pilot release. The Sponsor Highlights intake and Vergil automation plans retain their own systems.

- [Intake approval pilot acceptance and cutover](powerapps-jsonl-intake/approval-release/index.md) - Finish PA-40B/C participant retests and PA-40A usability/fidelity, plus PA-40D/E/F service authorization and fault acceptance, before PA-41/41A operational cutover and handoff. The isolated technical chain is accepted.
- [Shared Power Platform intake deployment](powerapps-jsonl-intake/shared-deployment/index.md) - After PA-41A and target readiness, import the accepted solution into a named shared environment, accept it in isolation, then switch future links and notifications with rollback.

- [Headless CI SharePoint JSONL download](sharepoint-headless-ci-download/index.md) - Follow SPD-26 shared functionality and acceptance on any one native OS with GitLab OIDC federation, ACRE workload profiles and read-only unattended capture/staging; seven bounded phases require approved app-only access and isolated live acceptance. No user credential cache or automatic apply/reporting.

- [Installable acornd background service](acornd-service.md) - Build on the landed portable job and strict daemon configuration contracts to serve existing APIs and configured check/gather jobs from a persistent local or SSH-managed service; SQLite history and composable server lifecycle are ready, and ACD-08A owns atomic job-config reload, coordinated with the incremental development loop; release requires signed paired binaries plus live host acceptance.
- [Automated JSONL management with the Vergil GitLab bot](vergil-gitlab-jsonl-automation/index.md) - PA-05/06/07/09 and BOT-02 are Waiting on validation, subject to their prerequisites. Service deployment, callback integration, reporting setup and authority decisions remain planned/blocked. This GitLab-only track is independent of SharePoint manual reporting and adds no bridge or list cleanup. Implemented runtime contracts await live deployment and end-to-end acceptance: protected runtime, callbacks, durable admission/cleanup, human review/merge and scheduled reporting.

- [Sponsor highlights clarity](sponsor-highlights-clarity.md) - Add evidence-backed mission value, a consistent project synopsis, reviewed ASPECT graduation across schema, local CLI and other adapters, accessibility checks, and an approved distribution pilot.
- [Grouped report separators](grouped-report-separators.md) - Extend the existing local --combine PDF with ordered project groups and full-page office dividers; keep individual report publication unchanged.
- [Swift SLM capability profile](swift-slm-profile.md) - Release Tongue, Redact, and Title after licensing, host, artifact, and quality gates.
- [Julia decision sidecar profile](julia-decision-sidecar.md) - Release advisory Choice and Noul decisions after independent parity and quality gates.
- [Release-gated Beam inference integration](beam-inference-integration.md) - Verify released serving contracts, then evaluate an explicitly configured hosted or operator-owned endpoint independently of local SLM sidecars.

- [Database-backed artifact generation](database-backed-artifact-generation.md) - Generate reproducible current or historical artifacts from canonical database state through existing typed renderers.
- [CEL metadata validation-profile pilot](cel-validation-profile-pilot.md) - Evaluate bounded, optional institutional rules against existing Rust/Rhai behavior before adopting a CEL runtime; see the [OPA/CEL review](../_assets/opa-cel-review.md).
- [dPID research-object support](dpid-research-object-support.md) - Recognize and resolve versioned dPIDs, import resolver RO-Crate metadata into RAD, and preserve manifest provenance through a read-only first release.
- [Deterministic and assisted gather extraction](gather-extraction.md) - Add bounded CSS extraction first, then an explicitly selected typed inference fallback for unresolved required fields.
- [Azure Cosmos DB bucket downloads](azure-cosmos-buckets.md) - After the provider-neutral source prerequisite, add a Cosmos DB for NoSQL container or logical partition as a read-only bucket source.

## Waiting on validation

- [Scheduled GitLab and SharePoint sync release](powerapps-jsonl-intake/scheduled-sync/index.md) - Catalog and approved-snapshot definitions are hardened and tested offline. SGS-01/02 need owner-selected isolated targets; SGS-03 activates an owned, monitored deployment after acceptance and PA-41A for operational mirroring.

- [Needle routing and native capability parity](needle-router-native-parity.md) - Native routes cover 46/46 bounded rows; PAR-34's synthetic safety matrix is accepted and PAR-16 is ready for final explicit-dispatch release checks and documentation. Automatic Needle selection remains preview-only after failed calibration.
- [Generic ask executor adapters](ask-executor-adapters.md) - Reusable local sidecar registration and configured CLI composition are landed with MCP, ACP, and endpoint bindings; remaining heterogeneous matrix and release evidence wait for PAR-16 native parity acceptance.
- [Retained GitLab intake fallback hardening](powerapps-jsonl-intake/gitlab-fallback-hardening/index.md) - Complete isolated byte-boundary, concurrent-write and native-restore acceptance for the retained Off GitLab route; every remaining phase is blocked on authorized disposable systems or connector access, and passing evidence does not authorize restoration.

## Epics

- [Research Activity Index capability](research-activity-index/index.md) - ABW-40 private GitLab loading, ABW-42/43 projection/catalog, and ABW-60–62 catalog media are accepted; coordinates ABW-19 initial-index acceptance and ABW-63–65 multiple-bucket loading.
- [PowerApp and Power Automate intake portfolio](powerapps-jsonl-intake/index.md) - Coordinates the accepted SharePoint baseline, report-first approval plans, retained GitLab fallback, shared deployment and boundaries with downloader, Teams, Vergil, editorial and enrichment work.

## Coordination

- [Ask routing and sidecar coordination](slm-sidecars.md) - Orders the four independently executable plans without owning phases.

Application publishing now has checked browser/desktop coverage: browser preview ZIP/checksum build, upload and release-link integration is implemented; desktop remains planned until ABW-11 adds its native crate and ABW-23–30 establish signed artifacts and channels. Both are application artifacts, outside crates.io publishing. GitHub Pages is enabled and a mirror-triggered build/smoke/deploy workflow is configured locally; root and `/acorn/` Chromium checks pass. First hosted deployment and HTTPS enforcement await mirroring the workflow and certificate provisioning. GitLab archive public access and native publication remain unverified.
