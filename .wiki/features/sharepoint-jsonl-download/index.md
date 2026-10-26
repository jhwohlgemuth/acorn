---
type: Plan
title: "SharePoint JSONL download for ACORN intake"
description: "Capture authenticated SharePoint logbook exports as verified local snapshots through the existing intake source URI interface."
difficulty: high
impact: high
package: [acorn-cli, acorn-host, acorn-lib]
tags: [sharepoint, microsoft-graph, jsonl, intake, authentication, download]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Let an authorized operator give ACORN the SharePoint combined `logbook.jsonl` URL, sign in through their organization, and stage an exact local snapshot for review. Remove the manual browser-download step while preserving the existing local and GitLab source behavior. Downloading must not apply, graduate, publish, acknowledge, or delete submissions.

Proposed use of the existing command shape, **not implemented yet**:

```sh
acorn intake stage --batch --config intake.json \
  --source "https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/logbook.jsonl"
```

The command should report the saved snapshot and provenance paths, capture time, source modification time, digest, and normal staging result. Preserve the existing machine-readable batch result on stdout; authentication prompts and capture diagnostics belong on stderr, with full provenance in the receipt. Here `--batch` means a multi-project review batch, not permission for unattended authentication. Subsequent review uses the saved local file with `--offline`. A separate generic `acorn download file` command is outside this first release: today's `download` command downloads bucket repositories and models, and its semantics must remain intact.

# Current evidence

Reviewed October 26, 2026 against `ca774b01` plus the current working tree. CodeGraph inspection of `crates/acorn-cli/src/commands/intake/source.rs` confirms local files and GitLab HTTP(S) file URLs are supported; HTTP(S) inputs currently enter the GitLab parser. Remote capture rejects offline mode, bounds the source to 10 MiB, pins GitLab commit provenance, and stores snapshots beneath the intake configuration's `.acorn/intake/sources/` directory. There is no implemented SharePoint/Graph authentication or download adapter in the searched crates.

The [October 26 live demonstration](../powerapps-jsonl-intake/sharepoint-pilot.md#october-26-live-demonstration-and-local-report) downloaded six rows (3,413 bytes) through the browser. Three selected submissions then passed local ACORN staging/application and generated a PDF. The original six-row bytes have SHA-256 `84eef36810338f5ebabb074d4e9993a09b9a910e38da82dc458f4344a18a09c7`; the [evidence packet](../powerapps-jsonl-intake/_assets/sharepoint/demo-20261026/evidence.json) records the files and runs. This proves the export and manual handoff, **not CLI authentication or remote capture**.

The combined exporter runs every five minutes and is distinct from per-submission receipt files. Its current bounds and remaining cloud acceptance belong to [PA-23](../powerapps-jsonl-intake/index.md). Capturing the exported file does not force a fresh list export or prove that every just-submitted receipt has reached it. Both operational pilots remain enabled; this work requires no app, flow, connector, credential, or SharePoint schema changes.

# Product and security contract

## Source selection and configuration

- Accept a direct HTTPS SharePoint document URL from an explicitly configured tenant/host. Initially support the combined export file; reject sharing links, library view pages, query strings, fragments, embedded credentials, malformed escaping, traversal, and unsupported schemes with actionable errors. Do not guess a provider by an arbitrary `.jsonl` suffix.
- Resolve the site, document library, and item through Graph. Match decoded path segments correctly, including spaces and Unicode; never double-decode. Freeze a bounded discovery strategy before implementation. If the least-privilege grant cannot support discovery, configure verified site/drive/item IDs rather than silently widening permission.
- Resolve non-secret tenant ID, public-client application ID, cloud, allowed SharePoint host and optional pinned resource IDs once at the CLI boundary. SPD-02 defines their exact names and precedence; do not add Microsoft secrets to an intake config, `.env`, command line or log.
- Keep GitLab URL parsing, origin checks, credential selection and commit receipts unchanged. A SharePoint URL must never receive GitLab credentials, and an unknown HTTPS origin must fail before sign-in or network access.

## Authentication

Use an organization-approved desktop/public-client Entra registration and delegated read access. Prefer system-browser authorization code with PKCE, validated state, and an exact loopback redirect. Do not collect a password, reuse browser cookies, embed a client secret, or assume the user's Chrome session grants ACORN a token. Microsoft documents desktop browser sign-in and public-client registration in its [desktop sign-in flow](https://learn.microsoft.com/en-us/entra/identity-platform/app-sign-in-flow) and [registration guidance](https://learn.microsoft.com/en-us/azure/active-directory/develop/scenario-desktop-app-registration).

The actual tenant and cloud must be verified. Microsoft 365 GCC uses worldwide Graph, while GCC High and DoD use different deployments; a Power Apps `.gov` URL alone does not select the Graph authority. Token audience and cloud endpoints must agree. See [Graph national cloud deployments](https://learn.microsoft.com/en-us/graph/deployments).

The [file download API](https://learn.microsoft.com/en-us/graph/api/driveitem-get-content?view=graph-rest-1.0) lists delegated `Files.Read` as its least-privileged work/school permission. That does not establish that this grant alone can discover and read this shared site's file. SPD-01 must demonstrate the actual resolution/read path and record its granted scopes and consent requirements. Do not request write permissions. Conditional Access and tenant consent are external gates; device code is not an automatic workaround for a denied desktop flow.

Keep access tokens in memory. Persist refresh credentials only through an approved OS credential store, keyed by tenant, application, cloud, account and granted scopes. Initial live acceptance targets the user's macOS host. Other hosts must retain a safe memory-only interactive path until their native secure-store adapter is tested; do not fall back to plaintext storage. Document credential removal and sign-out, handle a locked/unavailable store explicitly, and verify no secret appears in debug output or panic/error diagnostics. Noninteractive invocations must fail promptly when interaction is needed, rather than hang or launch a hidden sign-in session.

## Download and snapshot integrity

1. Read item metadata, recording site/drive/item identity, size, modification time and revision/ETag. Prefer a verifiable immutable version endpoint where available; otherwise compare metadata before and after content capture. An ETag is an opaque revision validator, not a Git commit or a promise of historical retrievability.
2. Fetch content using bounded time, redirects, response size and retry limits. Graph's content API redirects to a short-lived preauthenticated URL. Follow only the validated provider redirect policy over HTTPS; do not forward Graph authorization to the destination, log the signed URL, or accept arbitrary redirects into loopback/private services. See the [download API redirect contract](https://learn.microsoft.com/en-us/graph/api/driveitem-get-content?view=graph-rest-1.0).
3. Preserve the original UTF-8 bytes, including Unicode and LF delimiters. Enforce the existing 10 MiB source and 64 KiB record limits. Reject invalid UTF-8, login HTML, malformed/truncated JSONL, and size/integrity mismatches with no ready snapshot. Use the existing intake validator for envelope/schema acceptance; do not invent a second LogbookEntry model.
4. If the item changes during capture, discard the temporary result and retry a bounded number of times; report an unstable export after exhaustion. Do not combine rows from different revisions. Handle 401 refresh at most once per request sequence, distinguish permission denial from missing items, and bound 429/503 retries with `Retry-After` and an overall deadline.
5. Materialize `logbook.jsonl` and a versioned `source.json` receipt atomically beneath `.acorn/intake/sources/`. Retain provider, canonical source identity, revision evidence, byte/record counts, source modification time, capture time and SHA-256. Never put tokens or preauthenticated URLs in the receipt. Preserve the existing GitLab receipt format. Do not overwrite an existing verified snapshot; reuse identical content only after verifying its stored receipt and digest. Capture time must not make every identical download a new content identity.
6. Distinguish downloaded, verified and staged states. Failed validation or interrupted persistence must not leave a ready receipt, mutate the intake ledger, or erase a prior good snapshot. Local project/actor/authority checks still apply after download: read access to a SharePoint file does not confer intake authority.

`--offline` must reject a remote URL before authentication, directory creation or network calls. A captured local snapshot must remain usable offline after token expiry. A cached file must never be silently substituted for a failed fresh download. Display export age without calling it the latest list state; a missing modification time is unknown, not current.

# Boundaries and dependencies

- This plan owns the CLI source adapter, host authentication and read-only Graph transfer. [Power Platform intake](../powerapps-jsonl-intake/index.md) owns exporter correctness, standard-user access/licensing (PA-22), aggregate boundaries (PA-23), backups and any cutover. A successful CLI read does not close those phases or establish Power Apps licensing.
- [Manual ingestion documentation](../manual-jsonl-ingestion-graduation/walkthrough.md) remains the downstream workflow. No new graduation API, callbacks, Teams messages, email, report publication, or automatic source handback is introduced.
- [Vergil automation](../vergil-gitlab-jsonl-automation/index.md) remains independent. Unattended application credentials, scheduler integration, SharePoint writeback, direct list-to-JSONL construction, generic file downloads and cross-tenant discovery are excluded.
- Consume landed `acorn-host` HTTP services and `acorn-lib` provider seams. Coordinate exact shared files with the [shared API contract](../../refactors/api/shared-client-contract.md), [CLI composition](../../refactors/core/cli-composition-boundary.md), and [parallel execution locks](../../parallel-execution.md). Their entire plans need not finish first. Do not extract `acorn-api`, create a generic authentication framework, or rewrite bucket downloads for this adapter.
- The immediate external uncertainty is an approved CLI identity that can read this file. SPD-01 records that evidence or the precise denial. Fixture/contract work may proceed without a live tenant grant; live acceptance and release may not. No administrator ticket is assumed to exist, and no policy bypass is part of the plan.

# Acceptance matrix

| Area | Required evidence |
|---|---|
| Identity and permissions | Owner signs in with the approved public client; scopes/cloud are recorded without tokens; a separately authorized intended operator reads the same file; a denied account fails clearly. Owner success alone is labeled as such. |
| Source routing | Local and GitLab regressions pass; SharePoint direct URLs resolve spaces/Unicode; unconfigured origins, malformed URLs and sharing/view links fail before credential use. |
| Credentials | PKCE/state/redirect validation; cancellation and expired/revoked tokens; bounded refresh; unavailable secure store; account/tenant separation; redacted errors at maximum verbosity. |
| Transport | 401/403/404/429/503, redirect loops and unsafe targets, no bearer forwarding, size limits, interrupted transfer, and HTML masquerading as successful content. |
| Consistency | Stable exact download; append/replacement/deletion during capture; bounded unstable-revision failure; altered size/ETag; no mixed-version or partial ready artifact. |
| JSONL | Empty logbook has explicit zero-record behavior; malformed line, invalid UTF-8, duplicate/conflicting IDs, CRLF input policy, Unicode, 64 KiB record and 10 MiB total boundaries are covered without rewriting bytes. |
| Local handoff | Original bytes/digest match; receipt provenance is complete; repeated capture/staging does not duplicate accepted intake; interrupted persistence preserves prior evidence; offline staging succeeds without contacting Microsoft. |
| Operations | Capture reads the current export without invoking flows or modifying source files; snapshot age and limits are visible; both existing routes remain operational. |

# Execution phases

New files below are proposed ownership boundaries, not files claimed to exist. Each phase gets at most five active hours; external consent/host waiting is not active work. Register leaf modules before their implementation so each slice compiles and tests independently. If dependency evaluation or a security defect needs more time, create a new bounded phase before expanding the write set. Every handoff updates this plan and its portfolio links; evidence belongs in durable documentation, with sanitized supporting files in this plan's `_assets/` only when needed.

## SPD-01 — Prove approved read-only tenant access

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: Operator sign-in through the approved ORNL setup; permission to use an existing public-client registration or obtain one. Availability is unverified.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md` (new feasibility/operations record); this plan's `_assets/auth-evidence.json` (sanitized); authorized Entra desktop-app registration and read-only Graph access to the ResearchEnablement export, if permitted.
- Primary integration lock: SharePoint CLI identity
- Outcome: Verify tenant/cloud, registration, redirect, delegated scopes, and a metadata/content read using an approved client. Record the minimum sufficient route and whether site/drive IDs must be configured; make no operational Power Platform changes.
- Focused validation: Compare downloaded bytes/digest to the browser export at the same revision; demonstrate an authenticated Graph request rather than browser-cookie reuse; record exact denial if blocked.
- Handoff: Non-secret identifiers and consent/access evidence, or a precise external prerequisite. No live-read or licensing claim without evidence.

## SPD-02 — Freeze source, authentication and receipt contracts

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: Current source behavior verified; SPD-01 evidence may remain pending, with tenant-specific values explicitly unresolved.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md`; `tests/fixtures/sharepoint-intake/` (new synthetic fixtures); this plan.
- Primary integration lock: SharePoint source contract
- Outcome: Specify configuration names/precedence, provider routing, bounded path/ID resolution, callback and credential-cache lifecycle, snapshot identity/receipt version, empty/CRLF behavior and retry/deadline limits. Select maintained OAuth and secure-store dependencies after API/license/platform review; document the read-only service interface and initial macOS support boundary.
- Focused validation: Walk fixture cases through the contract; confirm no credentials in fixtures and no changes to GitLab receipts, intake authority, or downstream commands. Resolve every implementation-affecting choice before SPD-03.
- Handoff: Frozen contract and fixture expectations; future library owners can implement without inventing tenant policy.

## SPD-03 — Register optional host/provider dependencies and seams

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-02; exclusive manifest/root ownership window.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/lib.rs`, `crates/acorn-host/src/microsoft/{mod.rs,login.rs,cache.rs}` (new), `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/api/sharepoint/{mod.rs,resolve.rs,content.rs}` (new).
- Primary integration lock: Workspace graph
- Outcome: Land narrow request/credential/result types, feature-gated registration, and selected dependencies. Unimplemented operations return explicit unsupported errors; no route is advertised or enabled prematurely.
- Focused validation: Build affected crates and portable/no-default-feature paths; dependency/license review; no Microsoft host-only dependency leaks into WASM.
- Handoff: Compiling, testable seams for independent leaf implementation. Reserve shared API/library roots in this same serialized integration window.

## SPD-04 — Implement interactive PKCE sign-in

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-03; SPD-01 for live sign-in, fixture tests may proceed earlier.
- Exact write set: `crates/acorn-host/src/microsoft/login.rs`; `tests/fixtures/sharepoint-intake/auth/`.
- Primary integration lock: Microsoft authentication leaf
- Outcome: Browser sign-in and in-memory token acquisition with state/PKCE/redirect validation, bounded listener lifetime, cancellation, tenant/cloud binding and explicit noninteractive failure.
- Focused validation: Mock authorization/token responses, wrong state, callback errors, timeout/cancel, wrong tenant/audience configuration, and redaction; live approved sign-in when available.
- Handoff: Safe memory-only authentication usable without a persistent cache.

## SPD-05 — Add protected credential reuse on macOS

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-04 and a macOS credential-store test host.
- Exact write set: `crates/acorn-host/src/microsoft/cache.rs`; `crates/acorn-host/src/microsoft/login.rs`; `tests/fixtures/sharepoint-intake/auth/`; credential lifecycle section of `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Microsoft authentication leaf
- Outcome: Account/tenant/cloud-scoped secure refresh credential storage, refresh/revocation behavior and documented removal. Other hosts and unavailable stores retain explicit memory-only behavior without plaintext fallback.
- Focused validation: Mock locked/missing store, restart/reuse, expiry, revoked refresh token, account switch and secret redaction; native macOS store round trip with test credentials.
- Handoff: Credential lifecycle evidence and precise supported-platform limitations.

## SPD-06 — Resolve the configured SharePoint document

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-03 and frozen SPD-02 path/identity contract; SPD-01 for real resource mapping.
- Exact write set: `crates/acorn-lib/src/io/api/sharepoint/resolve.rs`; `tests/fixtures/sharepoint-intake/resolve/`.
- Primary integration lock: SharePoint provider leaf
- Outcome: Bounded Graph metadata resolution returning stable site/drive/item identity and revision evidence from a permitted direct URL or configured IDs.
- Focused validation: Spaces/Unicode, path ambiguity, pagination overflow, deleted/moved files, insufficient discovery permission, pinned-ID mismatch and hostile origins using a mock server.
- Handoff: Typed metadata and actionable resolution errors without widening scopes.

## SPD-07 — Capture bounded content with revision proof

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-04 and SPD-06; SPD-02 transfer contract.
- Exact write set: `crates/acorn-lib/src/io/api/sharepoint/content.rs`; `tests/fixtures/sharepoint-intake/content/`.
- Primary integration lock: SharePoint provider leaf
- Outcome: Exact bounded bytes and consistent revision evidence using existing HTTP policy. Refresh/retry/redirect handling stays bounded and cannot leak credentials; a changing export fails safely after the configured retry budget.
- Focused validation: Mock redirect and bearer isolation, 401/403/404/429/503, unsafe destinations, oversized/interrupted content, changed revision and repeated instability. If shared HTTP changes are required, schedule a separate transport phase instead of silently widening this leaf.
- Handoff: Read-only verified transfer result; no local staging side effects yet.

## SPD-08 — Persist complete immutable local snapshots

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-07 and frozen receipt contract.
- Exact write set: `crates/acorn-cli/src/commands/intake/sharepoint.rs` (new); `crates/acorn-cli/src/commands/intake/mod.rs` (private module registration only); `tests/fixtures/sharepoint-intake/snapshots/`.
- Primary integration lock: Intake snapshot persistence
- Outcome: Source validation and atomic byte/receipt materialization with deterministic content identity, protected local files, no overwrite and no partial ready states. Preserve GitLab persistence behavior.
- Focused validation: Empty/malformed/Unicode/boundary records, duplicate/conflicting identities through existing validators, digest mismatch, existing snapshot verification, concurrent local captures and failures between file/receipt writes.
- Handoff: A local captured path and versioned provenance available to existing intake stages.

## SPD-09 — Connect SharePoint to the intake source URI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-05, SPD-08; exclusive intake source window.
- Exact write set: `crates/acorn-cli/src/commands/intake/source.rs`, `crates/acorn-cli/src/commands/intake/sharepoint.rs`; `tests/fixtures/sharepoint-intake/cli/`.
- Primary integration lock: Intake source routing
- Outcome: Existing `intake stage --source` selects SharePoint, resolves non-secret configuration once, authenticates when needed, captures and stages locally. Output names the snapshot and its freshness/provenance. No new top-level command or automatic application is introduced.
- Focused validation: Local/GitLab source regressions, offline failure before network/storage, unknown host before auth, fresh-versus-cached semantics, repeat staging and local replay after token expiry. Follow existing `Label` output conventions.
- Handoff: Fixture-tested CLI behavior with live acceptance still explicitly pending.

## SPD-10 — Accept the real read-only handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-01 and SPD-09; approved owner and intended-operator access; available export. External access waiting is separate from the phase budget.
- Exact write set: Read-only requests to the named ResearchEnablement export; isolated local acceptance workspace; this plan's `_assets/live-acceptance.json`; `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: SharePoint CLI acceptance
- Outcome: Owner and intended operator download, verify and stage the export, then reproduce review offline. Record identity context, CLI version, resource IDs, revision, digest and record counts without secrets or unnecessary personal information.
- Focused validation: Same-revision browser/CLI digest comparison; repeat capture, offline replay, denied access and safe cancellation. Test concurrent exporter writes with mocks or an authorized isolated file; never alter the operational export to inject failure. Confirm no source modification or operational flow change.
- Handoff: Dated acceptance matrix with remaining limitations. An unavailable second identity remains an unaccepted matrix row, not an implied pass.

## SPD-11 — Publish operator instructions and demo handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-10; accepted command/error behavior.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md`, `docs/src/cli-reference/commands/intake.md`, `.wiki/features/manual-jsonl-ingestion-graduation/walkthrough.md`, `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`, `README.md`.
- Primary integration lock: Documentation generation
- Outcome: Document setup, approved sign-in, exact download/stage/offline commands, freshness, credential removal, access troubleshooting and browser-download fallback. Link the guide from existing intake documentation; describe platform and consent limitations accurately.
- Focused validation: Follow examples against sanitized fixtures and the accepted live setup; verify links and absence of tokens/signed URLs; do not claim CLI download is available in previously generated PDFs without regenerating them in a separately scoped documentation task.
- Handoff: Durable instructions sufficient to use the feature independently of this plan.

## SPD-12 — Complete integration gates and retire the plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-11; stable implementation and exclusive final-check window.
- Exact write set: This plan and any sanitized supporting evidence; `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/features/powerapps-jsonl-intake/index.md`; final validation record in `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Workspace validation
- Outcome: Accepted feature with accurate limitations, durable evidence and no stale planning entries. Preserve evidence outside the active plan before removing it.
- Focused validation: Serialize `make lint`, `make test`, `make check-wasm`, relevant no-default-feature checks and wiki validation after focused tests pass. Check application/API behavior against the frozen contract; unexpected defects get new bounded repair phases rather than being waived.
- Handoff: Remove this plan and its index/network/lane references only after all acceptance criteria pass; retain any genuinely unfinished phase with its stable ID.
