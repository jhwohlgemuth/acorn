---
type: Plan
title: "Headless CI SharePoint JSONL download"
description: "After SPD shared-functionality and single-platform acceptance, add reusable workload authentication and a read-only GitLab CI handoff for SharePoint JSONL snapshots."
difficulty: high
impact: high
package: [acorn-cli, acorn-host, acorn-lib]
tags: [sharepoint, microsoft-graph, oauth2, entra, oidc, ci, gitlab, jsonl, authentication]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Let an approved unattended GitLab CI job authenticate as an application, capture the configured SharePoint combined `logbook.jsonl`, and stage an immutable local review batch without a browser, terminal prompt, user refresh token, or desktop credential store. Reuse the authentication capability, Graph transfer, provenance, and intake behavior delivered by [SharePoint JSONL download (SPD)](../sharepoint-jsonl-download/index.md).

**SPD’s shared functionality and acceptance on one native platform are prerequisites for this plan.** Require the SPD-26 handoff before starting HCI-01: shared implementation, live download/offline handoff (SPD-10), live device/reset acceptance (SPD-22), and any one of native macOS SPD-16, Windows SPD-17 or Linux SPD-18. This is an OR condition, not a three-platform gate. Linux is a natural choice for the first CI target, but accepted macOS or Windows evidence also qualifies. HCI does not wait for the other two native validations, all-platform documentation SPD-11 or final retirement SPD-12. When SPD-26 is pruned, retain its accepted commit and durable evidence reference here; eventual SPD plan deletion is not itself acceptance. HCI-06 still separately proves actual headless operation on the approved Linux runner without a desktop vault.

The first supported deployment is one approved Linux GitLab runner and one Entra tenant, using GitLab OIDC federation. Keep token acquisition reusable for other workload providers, but do not claim their live acceptance. Application secrets, certificate signing/rotation, managed identity, other CI services, and Windows/macOS runner acceptance are separate future scope. If ORNL federation is unavailable, record the blocker and prepare a separately bounded alternative plan instead of silently switching credentials.

# Current evidence

Reviewed October 26, 2026 against `58c22da4` and the existing working tree. CodeGraph inspection of `crates/acorn-cli/src/commands/intake/source.rs` confirms local and GitLab capture; no SharePoint source is implemented there. The proposed `crates/acorn-host/src/auth/` and CLI auth command directories do not yet exist. SPD has 26 remaining phases, including the new single-platform handoff SPD-26, and explicitly excludes unattended application credentials. Its planned token-capability boundary is a design prerequisite, not an API already available to extend.

The owner SharePoint download/local-report demonstration proves the manual path, not CI identity or unattended capture. The [PA plan](../powerapps-jsonl-intake/index.md) retains intended-user access and exporter acceptance independently. The current `.gitlab-ci.yml` includes external reporting/release templates; those schedules and existing jobs must remain operational. No runner, Entra registration, app-only permission, federation trust, or CI acceptance was verified during this planning task.

The earlier 15–25-hour estimate covered a single CI mechanism on top of a finished downloader. The seven phases below reserve at most 35 active hours including feasibility, live acceptance, documentation and final gates; they are ceilings, not a delivery promise. Approval, tenant access and runner availability are external elapsed time and excluded. Re-estimate at HCI-01 using the actual accepted SPD interfaces.

# Authentication and permission decisions

## Workload identity

Use the OAuth client-credentials grant with a GitLab-issued assertion and an Entra federated credential. Microsoft authorizes the application rather than a signed-in user, and this grant does not issue refresh tokens. Acquire short-lived access tokens as needed within the job. Do not run the SPD user-ID-token/account-validation path or invent a user identity for the application. See [Microsoft client credentials](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-client-creds-grant-flow).

Bind trust to an exact issuer, audience, project and protected ref/subject. The initial rollout has no wildcard subject and no merge-request/fork access. Entra must be able to retrieve GitLab's OIDC discovery and signing keys; local runner reachability alone is insufficient. Validate the actual `code.ornl.gov` issuer, GitLab version, tenant cloud, supported federation audience and endpoints before implementation. The Power Apps government URL does not establish the SharePoint/Entra cloud. See [GitLab federation setup and endpoint requirements](https://docs.gitlab.com/ci/cloud_services/azure/).

Request the configured Graph resource's `/.default` scope through the already trusted tenant/token endpoint. Do not reuse SPD delegated scopes or assume delegated consent grants application access. Prefer the narrowest selected application permission that supports the exact download operations. Application consent and an explicit resource-level read assignment are both required. Pin approved site/drive/item identity to avoid broad directory discovery; prove all required metadata/content requests under the actual grant. No automatic escalation to tenant-wide read/write permissions. See [Microsoft selected permissions](https://learn.microsoft.com/en-us/graph/permissions-selected-overview).

## Configuration and credential input

- Extend the accepted ACRE `auth.profiles` contract with an explicit workload authentication mode, preserving existing public-client profiles, declaration order and source mapping behavior. Store only non-secret tenant/client/resource identifiers and an assertion environment-variable reference. HCI-02 freezes the exact serialized discriminator/field names against landed SPD; proposed fields are not a supported configuration example yet.
- Use one named CI assertion environment variable supplied by GitLab `id_tokens`. Read it once at the composition boundary into SPD's secret wrapper and pass a typed assertion to the host capability. Do not add arbitrary command execution, a token-printing command, token CLI arguments, a user-level config file or automatic `.env` discovery for this credential. Never serialize the resolved assertion back into ACRE, debug output or receipts.
- Resolve the workload profile through the source mapping. Missing or mismatched mappings fail before reading credentials or making requests. Preserve SPD's first-declared-profile default for its existing auth commands; `--device`, interactive login and user-cache options are incompatible with the workload mode and must fail clearly rather than switching mode.
- Treat repository configuration as a credential-routing trust boundary. Only the protected, reviewed ref supplies the production profile. Reuse SPD's authority/cloud/host validation; configuration edits must not send an assertion to an arbitrary token endpoint. Keep tenant, client, grant kind and Graph resource bound to token acquisition and in-memory reuse.
- Never read, write or unlock the user's macOS/Windows/Linux OAuth cache in workload mode. Do not fall back to personal credentials or a device-code prompt. Cache status/clear must report that this mode has no persistent credential cache without deleting another profile's credentials.

## Token lifetime and transport

Use SPD's maintained OAuth/HTTP libraries for client-credentials assertion exchange and its existing secret/redaction types. Expected additional runtime dependencies: **none**, subject to verifying the accepted SPD library supports the standards-based assertion parameters. No new keyring backend, Azure CLI runtime, JWT-signing library or certificate store is required for this first mode. If a library limitation requires an additional dependency, stop and add a bounded manifest/API/license review phase; do not write a custom cryptographic implementation.

The job assertion is a secret credential; local decoding is not proof of trust. Entra validates its signature and federated subject. Graph access tokens remain opaque. Use response expiry and a bounded clock-skew margin for in-memory reuse; reject missing/empty tokens and unusable lifetime responses. An expired assertion cannot be renewed by pretending to refresh a user token. A replacement pipeline obtains a new assertion. Bound retries/deadlines, distinguish denied trust/permission from transient transport failures, and never retry authorization denial indefinitely. At most one bounded re-acquisition after Graph 401; 403 does not widen permissions. Token exchange must not follow redirects to another origin.

Reuse SPD Graph capture unchanged: allowed origins, no bearer token forwarded to signed content URLs, bounded bytes/lines, optimistic revision checks, exact-byte digest, immutable receipt, interrupted-download safety and offline review. No parallel transfer implementation or separate SharePoint REST downloader.

# CI behavior and operational boundaries

The eventual command shape remains the SPD intake operation, with a CI-specific ACRE profile/source mapping:

```sh
acorn intake stage --batch --config ci-intake.json --source "$SHAREPOINT_LOGBOOK_URL"
```

This is proposed behavior, not a currently runnable SharePoint command. `ci-intake.json` is an ACRE document using the accepted SPD configuration structure. `--batch` still means multi-project staging; the selected workload profile establishes noninteractive authentication. Do not add a redundant `--ci` or `--headless` switch. Preserve SPD's machine-readable stdout; bounded redacted diagnostics go to stderr. Nonzero exit status prevents publishing a ready artifact. No Confirm, Spin requiring a terminal, code panel or browser launch occurs, even if a TTY happens to be available.

1. A manual, opt-in job on the approved protected ref receives a short-lived GitLab ID token with the configured federation audience. Live access is excluded from ordinary tests, fork/MR pipelines and release/report jobs.
2. ACORN exchanges the assertion for a Graph application token and captures/stages the current exported file in a fresh job-specific workspace. No developer home, shared token directory or cached user credentials are mounted.
3. Only a fully successful run makes the snapshot, provenance and staging output available as an access-controlled, short-lived review artifact. Use an explicit file allowlist, never the entire workspace/environment. Raw JSONL may contain sensitive data; the artifact audience/retention must be approved and enforced by the installed GitLab version. Sanitized evidence has no submission content, tokens or signed download URLs.
4. A reviewer downloads the accepted snapshot for the existing offline `stage -> inspect -> apply -> report` workflow. The CI job itself never applies, graduates, commits, publishes reports, sends callbacks/Teams/email, acknowledges rows or deletes list items. Re-staging follows SPD/local intake replay rules using the same local review state; independent fresh CI jobs do not constitute a durable duplicate-processing ledger.

Use isolated runner workspaces and the existing concurrency conventions. The first release has no recurring schedule. A restart begins a new capture; it never combines partial files from a failed run. Store no token/assertion in GitLab caches, dotenv artifacts, command traces or provenance. Disable shell/debug tracing before credential use and refuse a known debug-trace-enabled acceptance job. Test output redaction at maximum ACORN verbosity with synthetic secrets.

Reading an export does not trigger a fresh Power Automate export or prove list completeness. Report source modification/capture time and preserve SPD's distinction between file integrity and exporter freshness. The [October 27 isolated exporter acceptance](../powerapps-jsonl-intake/sharepoint-pilot.md#october-27-collected-export-acceptance) covers producer boundaries and failure handling; PA-24 backup/restore and PA-25 operational installation are accepted. SharePoint is active and the retained GitLab flows are Off. A successful CI read cannot substitute for that producer evidence or deploy the repair. Acceptance faults use isolated files/trust/configuration, never operational flows, token revocation or export mutation. No live Power Apps, Power Automate or existing GitLab connector changes are required.

# Dependencies and execution order

- **Hard predecessor:** SPD-26 shared-functionality/single-platform handoff, with durable evidence for SPD-02/23 configuration, SPD-03 token capability, SPD-07/08 Graph capture/snapshots, SPD-09/10 intake handoff, SPD-22 device/reset acceptance and at least one accepted native validation from SPD-16, SPD-17 or SPD-18. Shared implementation prerequisites remain required; only the additional native acceptance gates and SPD-11/12 closeout are removed from HCI’s critical path. No reverse dependency is added to SPD.
- **Sequence:** SPD-10 + SPD-22 + (SPD-16 OR SPD-17 OR SPD-18) -> SPD-26 -> HCI-01 feasibility -> HCI-02 profile contract -> HCI-03 assertion exchange -> HCI-04 intake integration -> HCI-05 opt-in CI template -> HCI-06 live acceptance -> HCI-07 documentation/final gates.
- **External gates:** an authorized application owner; exact approved app/tenant/cloud/subject; Entra-visible issuer metadata; selected Graph consent/resource assignment; approved Linux runner, protected ref, isolated acceptance document and artifact audience/retention. No existing administrator ticket or permission is presumed.
- **Shared locks:** coordinate ACRE/API configuration, host authentication, intake routing, CI/build and documentation with their owners in the [parallel guide](../../parallel-execution.md). Other complete API, CLI composition or runner-pool plans are not blanket prerequisites; the named runner must meet this plan's isolation requirements. Remaining SPD native validation may proceed alongside HCI on disjoint test systems, but shared auth/configuration files and operations documentation still serialize. Validate HCI changes against the pinned SPD-26 baseline and rerun affected SPD regressions; new changes do not inherit native acceptance automatically.
- **Independent work:** [Power Platform](../powerapps-jsonl-intake/index.md) and [Vergil](../vergil-gitlab-jsonl-automation/index.md) retain their existing scope. This plan neither deploys the Vergil bot nor creates a SharePoint-to-GitLab bridge. Browser download/manual reporting continues while this work is blocked.

# Acceptance matrix

| Area | Required evidence |
|---|---|
| SPD predecessor | Accepted SPD-26 commit and durable handoff: shared functionality, SPD-10/22 and one identified native platform result. Other native results and SPD-11/12 are not required; no premature claim that planned APIs exist or all OSes are accepted. |
| Configuration | Existing delegated configurations and JSON/JSONC/YAML round-trips pass; workload mode requires explicit valid source/profile/reference; wrong audience/authority/tenant and conflicting device/cache settings fail safely. |
| Assertions and tokens | Valid exchange; missing/empty/oversized assertion; invalid/expired/wrong-subject assertion; denial, malformed response, expiry and bounded timeout/retry behavior. Synthetic tests prove no secret/debug/error leakage or redirect forwarding. |
| Mode separation | Workload mode never invokes browser, Confirm, device polling, user identity validation or secure cache; delegated mode never consumes the CI assertion. Two clients/resources cannot reuse each other's in-memory token. |
| Graph capture | Pinned authorized item succeeds; unauthorized sibling fails under the selected grant; 401/403/404/429/5xx and unstable file tests preserve SPD's snapshot safety. An expired signed download URL triggers only the accepted bounded recovery path. |
| Headless process | No stdin/TTY, empty user home, no D-Bus/keyring and no browser still succeed with valid workload credentials; missing credentials fail promptly with clean stdout and actionable redacted stderr. |
| Repeatability | Two fresh successful jobs capture verifiable bytes; retry after interrupted capture exposes no partial ready artifact; offline review works without Microsoft credentials. Local repeated staging obeys stable submission IDs and does not mutate SharePoint. |
| Pipeline boundaries | Protected opt-in job only; fork/MR/unapproved refs cannot receive usable trust; debug trace is excluded; artifact allowlist, retention and reader access are verified; no existing job/schedule changes. |
| Operational isolation | Read-only export requests; isolated negative tests; current form, exporter, GitLab route and reporting schedules unaffected. No live tenant mutation without its authorized owner. |

# Execution phases

All proposed source paths below must be reconciled to SPD's accepted module layout before assignment. Do not implement stub copies of missing SPD modules. Each phase includes its plan/status handoff as planning maintenance; product writes are limited to its exact set. External waiting is not active work. Unexpected defects or an extra authentication mechanism require a new bounded phase.

## HCI-01 — Prove the approved federation path

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: SPD-26 accepted shared functionality and one native platform; authorized application/SharePoint owner and access to inspect the named GitLab issuer/runner. SPD is not yet implemented and these external grants are unverified.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-ci.md` (new feasibility record); this plan's `_assets/federation-readiness.json` (new sanitized evidence); authorized isolated Entra application/federated trust for the GitLab `research-enablement/experiments/power-automate` project (22464), selected read grant to an isolated acceptance file, and read-only inspection of `code.ornl.gov` OIDC/runner configuration. Record actual resource IDs before any mutation; no operational app registration or credential edits.
- Primary integration lock: isolated CI federation provisioning.
- Outcome: Freeze the actual cloud, client, exact issuer/audience/subject/ref, selected permission, resource IDs, runner identity and artifact policy. Prove an app-only read using approved provisioning tooling without printing credentials. Record precise denial if unavailable; do not substitute a user token. The developer's local ability to reach OIDC endpoints does not prove Entra can.
- Focused validation: Successful token exchange and metadata/content read of the isolated file; inaccessible sibling; exact trust-subject match; allowed network/runner/artifact controls. Review evidence for secrets and unrelated personal data.
- Handoff: Accepted sanitized deployment contract and SPD baseline make HCI-02 eligible; a policy/network denial leaves this phase blocked with the responsible external action.

## HCI-02 — Extend ACRE with a workload profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-01 and the accepted SPD-26 baseline; exclusive ACRE configuration window.
- Exact write set: `crates/acorn-host/src/auth/config.rs`, `crates/acorn-lib/src/io/config/application/auth.rs`, `crates/acorn-lib/src/io/config/tests/mod.rs`, `crates/acorn-cli/src/io/auth_config.rs`; `tests/fixtures/oauth2/workload/config/` (new); `docs/src/concepts/acre.md`, `docs/src/cli-reference/commands/sharepoint-ci.md`.
- Primary integration lock: API/config roots.
- Outcome: Freeze and implement workload discriminator, non-secret assertion-variable reference, cloud/resource constraints, source mapping and mode-conflict validation without changing SPD defaults. Verify SPD dependency support for assertion exchange. An unimplemented workload acquisition path returns explicit unsupported behavior until HCI-03/04; it cannot silently choose delegated login.
- Focused validation: Focused configuration tests, schema export and format round-trips; missing/unknown/invalid fields, changed authority, mixed modes and existing SPD fixtures. No credential value appears in serialized/debug config.
- Handoff: Document exact schema and resolved host input; HCI-03 consumes these types without reopening profile design.

## HCI-03 — Implement bounded workload token acquisition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-02, accepted SPD token/HTTP capability and dependency audit; exclusive host auth window.
- Exact write set: `crates/acorn-host/src/auth/mod.rs`, `crates/acorn-host/src/auth/entra.rs`, `crates/acorn-host/src/auth/workload.rs` (new); `tests/fixtures/oauth2/workload/token/` (new).
- Primary integration lock: host authentication capability.
- Outcome: Implement client-credentials assertion exchange with opaque API tokens, application identity context, in-memory expiry handling, endpoint binding, bounded retry/deadline and typed redacted failures. No user cache, refresh token, public token-dump API, custom JWT verification or signing implementation.
- Focused validation: Host fixture tests for two synthetic issuers/resources, malformed/expired/denied credentials and responses, cancellation, HTTP redirects, lifetime margins, bounded retries and synthetic-secret redaction. Compile affected host/portable feature paths.
- Handoff: Working injected workload capability with recorded library/feature evidence; HCI-04 can compose it into intake.

## HCI-04 — Connect workload authentication to headless intake

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-03 and exclusive intake routing window.
- Exact write set: `crates/acorn-cli/src/io/auth_config.rs`, `crates/acorn-cli/src/commands/intake/source.rs`, `crates/acorn-cli/src/commands/intake/sharepoint.rs`, `crates/acorn-cli/src/commands/auth/mod.rs`, `crates/acorn-cli/src/commands/auth/tests.rs`; `crates/acorn-cli/tests/sharepoint_ci.rs` and `tests/fixtures/sharepoint-intake/ci/` (new).
- Primary integration lock: intake source routing.
- Outcome: Select workload capability from the resolved source mapping, read the named assertion variable only at the trusted boundary, and reuse SPD capture/staging. Auth login/device options reject workload mode clearly; cache commands report no persistent workload cache. Preserve machine stdout and existing local/GitLab/delegated behavior.
- Focused validation: Subprocess tests without TTY/stdin, browser or credential stores; unknown-source/offline rejection before network; no UI/cache invocations; isolated app/delegated tokens; valid capture, partial failure and offline restaging. Reuse SPD transport/bounds tests rather than copying implementation.
- Handoff: Headless binary acceptance fixtures pass; HCI-05 may package the existing command without new CLI flags.

## HCI-05 — Prepare an opt-in GitLab CI template

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-04 and approved HCI-01 runner/ref/artifact contract; exclusive CI/build window.
- Exact write set: `scripts/ci/sharepoint-download.gitlab-ci.yml`, `scripts/ci/sharepoint-download.sh` (new); `tests/fixtures/sharepoint-intake/ci/pipeline/` (new); `docs/src/cli-reference/commands/sharepoint-ci.md`. Do not edit the active `.gitlab-ci.yml` or external reporting/release templates in this phase.
- Primary integration lock: CI/build.
- Outcome: Reusable manual job declares `id_tokens`, protects its exact ref/project, uses a pinned accepted ACORN binary/environment, rejects debug tracing, creates a fresh workspace, runs capture/stage and exports only approved successful artifacts with enforced audience/retention. Shell helper contains no embedded token, secret output or broad cache/artifact collection. No schedule or automatic apply/report action.
- Focused validation: GitLab CI lint using the installed server/version; shell syntax and synthetic pipeline tests for missing credentials, failure exits, paths with spaces, trace refusal and artifact allowlist. Verify rules exclude MR/fork/unapproved refs and existing pipeline files are unchanged.
- Handoff: Reviewable template and runbook for isolated pilot inclusion; HCI-06 owns actual deployment and live acceptance.

## HCI-06 — Accept fresh unattended jobs in the isolated pilot

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-05; HCI-01 grants still valid; authorized isolated manual job inclusion and available approved Linux runner. Live permission/runner waits are external.
- Exact write set: Isolated manual CI job/include in GitLab project 22464 on the approved protected ref; its dedicated job artifacts/workspace; the HCI-01 isolated test identity/file only for negative fixtures; this plan's `_assets/live-ci-acceptance.json`; `docs/src/cli-reference/commands/sharepoint-ci.md`. Existing operational jobs/schedules, Power Platform resources and production credentials are outside the write set.
- Primary integration lock: isolated SharePoint CI acceptance.
- Outcome: Two fresh headless jobs capture the same known isolated export bytes with distinct job provenance; a separate authorized read of the current operational combined export proves the real target if its read grant is approved. Clearly distinguish fixture success from production-file acceptance. Test interrupted capture/retry and isolated denied/expired/wrong-subject cases without revoking operational access.
- Focused validation: Verify digest, exact bytes, metadata/receipt and offline review; no credential store or user session; failed runs expose no ready artifacts; unauthorized sibling denied; approved artifact access/expiry enforced; protected rules and unaffected existing jobs evidenced. Record source age without claiming producer freshness/completeness.
- Handoff: Sanitized run IDs, commit/tool version, permission scope, byte/digest results and negative outcomes. Missing real-target permission leaves live acceptance incomplete; HCI-07 cannot claim deployment readiness from mocks alone.

## HCI-07 — Publish operations guidance and close integration gates

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: HCI-06 accepted for the intended target; stable implementation and exclusive final-check window.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-ci.md`, `docs/src/cli-reference/commands/sharepoint-intake.md`, `docs/src/cli-reference/commands/auth.md`, `docs/src/cli-reference/commands/intake.md`, `docs/src/SUMMARY.md`, `README.md`; this plan/evidence and its root/feature index, dependency-network and parallel-lane entries.
- Primary integration lock: documentation generation and final validation.
- Outcome: Document app-only vs delegated behavior, exact accepted ACRE example/job include, registration/resource grant prerequisites, no-prompt errors, assertion expiry/retry, artifact controls, offline review, isolated testing and rollback by removing the dedicated job/trust. Preserve provenance and sanitized acceptance outside the active plan, then retire the plan and all portfolio references. Removal of an already issued token's trust does not imply immediate invalidation; document expiry and resource-grant controls accurately.
- Focused validation: Run required `make lint`, `make test`, `make check-wasm`, dependency/feature checks and docs build once after changes stabilize; focused auth/intake/pipeline tests must pass. Validate examples against accepted CLI/schema; validate wiki and links. New product defects become separate bounded repair phases rather than expanding this docs phase.
- Handoff: Accepted commit, reproducible job/runbook and final results remain in durable documentation. Delete this plan only when its full acceptance matrix is met; retain precise external blockers otherwise.
