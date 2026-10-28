---
type: Plan
title: "SharePoint JSONL download for ACORN intake"
description: "Add browser and device OAuth authorization, cross-platform protected credential caching with testing reset commands, and Graph file capture for SharePoint intake."
difficulty: high
impact: high
package: [acorn-cli, acorn-host, acorn-lib]
tags: [sharepoint, microsoft-graph, oauth2, entra, jsonl, intake, authentication, credentials, cross-platform, download]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Let an authorized operator give ACORN the SharePoint combined `logbook.jsonl` URL, sign in through their organization, and stage an exact local snapshot for review. Remove the manual browser-download step while preserving the existing local and GitLab source behavior. Build authentication as a reusable host capability and Graph file capture as a byte-oriented service; SharePoint JSONL intake is their first production consumer. Downloading must not apply, graduate, publish, acknowledge, or delete submissions.

Proposed use of the existing command shape, **not implemented yet**:

```sh
acorn intake stage --batch --config intake.json \
  --source "https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/logbook.jsonl"
```

The command should report the saved snapshot and provenance paths, capture time, source modification time, digest, and normal staging result. Preserve the existing machine-readable batch result on stdout; authentication prompts and capture diagnostics belong on stderr, with full provenance in the receipt. Here `--batch` means a multi-project review batch, not permission for unattended authentication. Subsequent review uses the saved local file with `--offline`. A separate generic `acorn download file` command is outside this first release: today's `download` command downloads bucket repositories and models, and its semantics must remain intact.

# Downstream operator flow

Preserve the documented **stage -> inspect -> apply -> report** workflow in the [intake guide](../../../docs/src/cli-reference/commands/intake.md#downloaded-jsonl-to-a-local-pdf) and [three-row SharePoint demonstration](../manual-jsonl-ingestion-graduation/walkthrough.md#sharepoint-download-to-a-combined-local-pdf). OAuth replaces browser download at the initial capture/stage step; it does not bypass inspection or confirmation, add graduation as a report prerequisite, or publish/send callbacks. Remote staging runs online; subsequent inspect, apply and local reporting may run offline. Examples use `acorn`, normal apply confirmation, the current-directory report root, all configured projects and `.acorn/reports/report.pdf` unless an override is necessary. Keep the explicit reporting window and local-only flags. Do not invent defaults for currently required intake configuration/source/batch arguments.

# Current evidence

SharePoint-aware planning review: October 26, 2026 against `58c22da4` plus the current working tree. Earlier capture evidence was recorded against `ca774b01`. CodeGraph inspection of `crates/acorn-cli/src/commands/intake/source.rs` confirms local files and GitLab HTTP(S) file URLs are supported; HTTP(S) inputs currently enter the GitLab parser. Remote capture rejects offline mode, bounds the source to 10 MiB, pins GitLab commit provenance, and stores snapshots beneath the intake configuration's `.acorn/intake/sources/` directory. There is no live SharePoint/Graph authentication or download adapter; SPD-03 has only inactive, compiling module seams. The existing global `--clear-cache` clears application/database/download cache and must retain that separate meaning.

October 27 implementation evidence: the frozen contract and synthetic fixtures are in the [SharePoint intake capture guide](../../../docs/src/cli-reference/commands/sharepoint-intake.md). Feature-gated host auth and Graph/SharePoint seams compile on macOS without exposing a CLI route. Focused host tests, the SharePoint library check, portable WASM check and existing GitLab capture/offline-review regression passed. The lockfile advisory count remains six, matching the prior baseline. Linux cross-compilation on this macOS host stopped before Rust code checking because the cross C compiler is absent. The current working tree removes the four pre-staged keyring dependencies and three empty platform modules after `cargo-shear` rejected them; `make check-dependencies` and all 24 `acorn-host --features auth` tests pass. The selected versions and features remain requirements for SPD-24/05/14/15, to be restored with code that uses them. [SPD-01's sanitized evidence](_assets/auth-evidence.json) records the missing approved public-client registration; no Graph request was made.

October 27 fixture implementation against `bde5df3f` plus the current working tree: ACRE now parses and validates ordered `auth.profiles` and exact SharePoint `auth.sources` in JSON, JSONC and YAML. The test-only CLI resolver exercises selection of a validated host profile without invoking project hooks or database setup; SPD-09 or SPD-21 must make it a production module when adding its first caller. The host profile pins Entra authority and grant endpoints; its OIDC verifier checks signed ID tokens against trusted discovery/JWKS with bounded metadata requests. The device grant uses library polling with a private code handle, typed redacted failures, cancellation and an outer monotonic deadline. `cargo test -p acorn-host --features auth auth::tests --offline` passed 16 tests covering grant success, pending, slow-down, denial, expiry, provider errors, rate limiting, URL safety, signature and claim failures. `cargo test -p acorn-lib --features auth test_auth_ --offline` passed focused ACRE tests, and `cargo test -p acorn-cli --no-default-features --features auth io::tests::auth --offline` passed two resolver tests. Focused Clippy checks for all three crates and the planning wiki validator passed. No CLI auth route, protected cache adapter, live tenant sign-in or Graph capture is claimed; those remain separate phases.

The [October 26 live demonstration](../powerapps-jsonl-intake/sharepoint-pilot.md#october-26-live-demonstration-and-local-report) downloaded six rows (3,413 bytes) through the browser. Three selected submissions then passed local ACORN staging/application and generated a PDF. The original six-row bytes have SHA-256 `84eef36810338f5ebabb074d4e9993a09b9a910e38da82dc458f4344a18a09c7`; the [evidence packet](../powerapps-jsonl-intake/_assets/sharepoint/demo-20261026/evidence.json) records the files and runs. This proves the export and manual handoff, **not CLI authentication or remote capture**.

The combined exporter runs every five minutes and is distinct from per-submission receipt files. Its 5,000-row/10 MiB bounds and repaired isolated revision passed [collected-export acceptance](../powerapps-jsonl-intake/sharepoint-pilot.md#october-27-collected-export-acceptance) on October 27. PA-24 native backup/restore and PA-25 operational installation are accepted. The repaired exporter is deployed; later editing acceptance independently verified original/correction bytes in an 18-row collected snapshot. Capturing the exported file does not force a fresh list export or prove that every just-submitted receipt has reached it. Graph revision consistency proves captured file identity/bytes, not list completeness or successful exporter readback. The SharePoint receipt history is retained across local apply; repeated snapshots need replay-safe local staging, not remote deletion or acknowledgement. SharePoint is the active default; the retained GitLab intake and catalog flows are Off; this work requires no app, flow, connector, credential, or SharePoint schema changes.

# Reusable architecture

OAuth 2.0 authorization code with PKCE is a standard protocol; Entra is the identity provider used by this integration. OpenID Connect adds identity information when needed. These mechanisms are not specific to SharePoint. Microsoft documents the [authorization-code protocol](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow) and [resource-specific scopes](https://learn.microsoft.com/en-us/entra/identity-platform/scopes-oidc).

| Layer | Responsibility | Must not depend on |
|---|---|---|
| Host OAuth 2.0 capability | Typed authorization/token requests, PKCE/state, bounded browser callback, device-code request/polling, refresh, cancellation, protected cache and redacted failures, using maintained libraries | SharePoint URLs, Graph paths, JSONL, intake commands or ORNL constants |
| Entra profile | Explicit tenant/authority/cloud configuration, approved endpoints, public-client registration, Entra-specific scope and identity policy | Site/library names, export cadence or logbook schema |
| Graph file service | Metadata and bounded byte capture by drive/item identity, revisions, redirects and provider error handling; accepts a scoped token capability | OAuth browser mechanics, OS credential storage, JSONL or intake persistence |
| SharePoint source adapter | Allowed site/document URL resolution to Graph identity and the required read scopes | Token exchange, refresh or cache implementations |
| Intake adapter | JSONL bounds/validation, immutable local receipt, review staging, project authority and user-facing output | Entra protocol details or credential-store internals |

The proposed host module is `acorn-host::auth`, with an OAuth 2.0 implementation, an Entra profile and a secure-cache adapter. Use a small typed request/result interface, not a registry of every possible authentication scheme. Callers supply validated authority/client/redirect information, target resource, requested scopes and interaction policy. Existing provider operations consume an injected, resource-scoped token acquisition capability so refresh stays outside their business logic. Preserve secret wrappers and dependency direction: `acorn-host` must not depend on `acorn-lib` to acquire credentials. Adapt at the existing provider/composition boundary without replacing GitLab PAT authentication.

The proposed optional ACRE `auth` attribute separates a reusable named authentication profile from the source profile's allowed host, site/drive/item identity and resource scopes. The ORNL URL is an acceptance fixture, never a compiled default. Another approved tenant, site, document or Entra-protected API must not require changes to the OAuth implementation. Standard protocol behavior is verified with two synthetic issuers; real support for additional providers, grant types and APIs still needs provider-specific acceptance.

Authentication reuse does not make tokens interchangeable. Token requests and cached access-token selection must include authority/tenant, application, account, resource audience and normalized scopes; only compatible grants may be reused. Refresh credentials follow the library/provider's documented cache rules and retain account/authority/client isolation. A Graph token must not be sent to SharePoint REST, Azure Key Vault or an arbitrary URL; another resource requires its own correctly scoped token. Consent is requested only for the operation selected by the caller. OIDC ID tokens are never used as API bearer tokens. The Entra profile uses library-validated OIDC identity for account binding.

Keep the initial implementation bounded: two explicit interactive grants (browser authorization code with PKCE and device authorization), one live Entra profile and one SharePoint intake consumer. Prove reusable boundaries with fixtures rather than adding unrelated live service integrations, an unrestricted token-management CLI, other OAuth grant types, or a new crate. A narrow `auth login` and `auth cache status/clear` CLI is in scope for reusable sign-in and repeatable testing. A later OneDrive/Graph consumer should reuse byte capture; a different Entra API should reuse sign-in with its own permissions and API adapter.

The existing Confirm control is implemented in `crates/acorn-cli/src/io/interactive/mod.rs` (`Viewport::confirm`, `PromptOutcome`) and already used by manual intake. It renders filled Yes/No buttons, defaults to No, supports keyboard cancellation and no-color display. Its viewport currently requires stdin/stdout TTYs and writes to stdout, so using it for OAuth requires a tested stderr rendering path that preserves existing callers. See the [interactive controls plan](../interactive-metadata-chat.md).

# Product and security contract

## Source selection and configuration

- Accept a direct HTTPS SharePoint document URL from an explicitly configured tenant/host. Initially support the combined export file; reject sharing links, library view pages, query strings, fragments, embedded credentials, malformed escaping, traversal, and unsupported schemes with actionable errors. Do not guess a provider by an arbitrary `.jsonl` suffix.
- Resolve the site, document library, and item through Graph. Match decoded path segments correctly, including spaces and Unicode; never double-decode. Freeze a bounded discovery strategy before implementation. If the least-privilege grant cannot support discovery, configure verified site/drive/item IDs rather than silently widening permission.
- Resolve a named authentication profile (non-secret tenant ID, public-client application ID and cloud) separately from the SharePoint source profile (allowed host, resource scopes and optional pinned resource IDs), once at the CLI boundary. The frozen contract defines their exact names and precedence; do not add Microsoft secrets to an intake config, `.env`, command line or log.
- Keep GitLab URL parsing, origin checks, credential selection and commit receipts unchanged. A SharePoint URL must never receive GitLab credentials, and an unknown HTTPS origin must fail before sign-in or network access.

## Profile loading and trust boundary

The optional top-level ACRE `auth` attribute (`ApplicationConfiguration`) contains `auth.profiles` (named reusable non-secret authentication profiles) and `auth.sources` (SharePoint source mappings referencing those names). It uses existing ACRE discovery and explicit `--config` selection, including JSON/JSONC and YAML support. ACRE's existing `config` attribute remains reserved for synchronization targets. Existing documents without `auth` retain their behavior; requesting OAuth without a matching profile fails clearly.

For example, `acorn auth login --config .acorn.jsonc --device` selects the first declared entry in `auth.profiles` from that ACRE document. Add `--profile ornl` to select `auth.profiles.ornl` explicitly. Without `--config`, use normal ACRE discovery. The intake command's existing `--config` selects the same ACRE document for folders, authority rules and authentication/source mappings; it does not select a second OAuth file. `--profile` selects a name, not a path. Show the resolved configuration path and selected profile in interactive context so operators know which repository's settings they are using. The attribute contains public client/tenant identifiers, resource/scope limits, grant preference and cache policy only; passwords, client secrets, access/refresh tokens and cache contents are forbidden and remain outside ACRE.

Resolve the selected ACRE document once into immutable inputs; provider operations never rediscover files or read environment variables/configuration. Reuse the ACRE parser and filename precedence without loading unrelated project data, running hooks, initializing the database or requiring GitLab credentials for `auth` commands. Downloaded JSONL, remote metadata and nested project files cannot replace that resolved configuration. Preserve the host/library boundary: ACRE types live in `acorn-lib` and translate into provider-neutral host inputs; `acorn-host` never loads ACRE or depends on `acorn-lib`.

CLI overrides are limited to the existing configuration path, named profile/account selectors and explicit browser/device choice. Select source mappings by an unambiguous canonical host/site/library match; fail on zero or multiple matches before accessing credentials. Reject conflicting pinned IDs, duplicate names, insecure endpoints, embedded secrets, unknown fields and unsupported ACRE versions. Each auth profile defines allowed resource/scope bounds and one explicit default resource/scope set for `auth login`; source mappings may select narrower permitted requests but cannot extend those bounds or endpoints. An unconfigured or ambiguous login target fails before sign-in. For `auth login` and `auth cache status/clear`, an omitted `--profile` selects the first declared profile. An explicit unknown name fails instead of falling back; an empty or absent profile collection fails before credentials/network. Preserve declaration order in JSON, JSONC and YAML parsing and formatting; do not use alphabetic ordering or unordered-map iteration to choose the default. Schema export and round-trips must preserve this order-sensitive behavior. Ordinary intake continues to use its matching source's referenced profile, so a default cannot redirect a configured source.

Repository configuration is shareable policy, not proof that an endpoint may receive existing credentials. Validate provider authority/cloud/discovery relationships and bind cached credentials to the full validated issuer, tenant, client and token endpoint, with resource/scope checks at acquisition. A profile name alone is never a credential lookup key. Reusing the name `ornl` in another repository, or changing its endpoints, must not redirect an existing refresh credential or authorize sending a bearer token to a different resource. Keep the existing Confirm sign-in interaction and display the effective authority/resource; cache aliases share entries only when their validated identities and endpoint bindings match. Coordinate ACRE roots and boundary types with API-06/API-14 and the ACRE format owner; preserve existing GitLab credential precedence.

## Verified account identity

The account used for caching and `--account` selection must be established by validated provider identity, never a caller-supplied email or decoded API access token. For the Entra profile, request `openid` for identity and `offline_access` when persistent refresh is requested; use a maintained OIDC library to validate returned ID tokens. Use issuer plus subject as the opaque stable account key, bound to the configured client and tenant. Email/display name is optional presentation only. Browser flow requires its generated nonce; device flow validates signature, issuer, client audience and time constraints under the documented grant rules without inventing a nonce requirement the provider does not support. If validated identity or refresh material is absent, report that limitation and do not silently persist credentials under a guessed account.

Treat Microsoft API access tokens as opaque. Enforce resource binding through validated configuration, the token request and response metadata; validation of the API token belongs to its resource server. Reject wrong-account/tenant results before replacing cached credentials. Pin discovery/JWKS to the selected authority and validate allowed HTTPS destinations before fetching; bound metadata size, redirects and key refresh, including unknown-key rotation. See [Microsoft access-token guidance](https://learn.microsoft.com/en-us/entra/identity-platform/access-tokens). The landed identity adapter is shared by both grant implementations.

## Authentication

Use an organization-approved desktop/public-client Entra registration and delegated read access. Prefer system-browser authorization code with PKCE, validated state, and an exact loopback redirect. Do not collect a password, reuse browser cookies, embed a client secret, or assume the user's Chrome session grants ACORN a token. Microsoft documents desktop browser sign-in and public-client registration in its [desktop sign-in flow](https://learn.microsoft.com/en-us/entra/identity-platform/app-sign-in-flow) and [registration guidance](https://learn.microsoft.com/en-us/azure/active-directory/develop/scenario-desktop-app-registration).

The actual tenant and cloud must be verified. Microsoft 365 GCC uses worldwide Graph, while GCC High and DoD use different deployments; a Power Apps `.gov` URL alone does not select the Graph authority. Token audience and cloud endpoints must agree. See [Graph national cloud deployments](https://learn.microsoft.com/en-us/graph/deployments).

The [file download API](https://learn.microsoft.com/en-us/graph/api/driveitem-get-content?view=graph-rest-1.0) lists delegated `Files.Read` as its least-privileged work/school permission. That does not establish that this grant alone can discover and read this shared site's file. SPD-01 must demonstrate the actual resolution/read path and record its granted scopes and consent requirements. Do not request write permissions. Conditional Access and tenant consent are external gates; device code is not an automatic workaround for a denied desktop flow.

Keep access tokens in memory. Persist refresh credentials only through an approved OS credential store, isolated by authority, tenant, application, cloud and account under the library/provider cache rules; access-token lookup also checks resource audience and compatible granted scopes. Protected caching is a release requirement on macOS, Windows and Linux with an available supported store. Native acceptance is required on each platform; memory-only operation on an unavailable store is a documented fallback, not evidence that persistent caching passed. Do not fall back to plaintext storage. Document credential removal and sign-out, handle a locked/unavailable store explicitly, and verify no secret appears in debug output or panic/error diagnostics. Noninteractive invocations must fail promptly when interaction is needed, rather than hang or launch a hidden sign-in session.

## Explicit device authorization

Browser authorization with PKCE remains the default. Keep `flow: browser | device` in the authentication profile, defaulting to `browser` when omitted. `auth login --device` overrides the selected profile to use device authorization; without the flag, honor the profile's flow. The CLI exposes the boolean `--device` flag rather than a `--flow` option. Never switch grants automatically after a policy denial, callback failure or timeout. Device authorization uses the same authority/client/resource/scopes and protected-cache interfaces as browser authorization; the library returns the granted identity and permissions without guessing them from a SharePoint URL.

After the shared Confirm control asks **“Start device-code sign-in?”**, request the user/device codes, render the titled panel and begin bounded polling. A second automatic browser launch is not required: the displayed verification URL works in a browser on the same or another computer. No/Cancel before starting makes no device-authorization request. Keep the private polling `device_code` out of all UI and diagnostics.

Honor the server's expiry and minimum polling interval, a monotonic deadline and RFC 8628 `slow_down` behavior; handle `authorization_pending`, provider-specific denial/expiry names, unsupported grants, transport timeouts and throttling separately. Backoff must never exceed the overall expiry or create an unbounded loop. Escape/Ctrl-C stops polling and prevents late responses from being persisted. Use the selected OAuth library's device-flow support; no new grant-specific dependency is assumed. See [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628) and [Microsoft's device flow](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code).

Entra registration and tenant policy must actually permit device authorization. SPD-01 records registration capability and known constraints; SPD-22 records a successful read-only device sign-in or the exact policy denial. A denied tenant does not justify automatic browser/device fallback or changing organizational policy. The browser track may progress independently, but device-flow live acceptance remains open until demonstrated on an approved configured tenant. Headless means the CLI has no local browser, not that no human is involved: a terminal user still enters the code in a browser elsewhere. Without an interactive terminal, fresh authorization fails clearly; no unattended polling or code emission to CI logs.

Keep the chosen grant/origin in protected cache metadata when needed for provider policy. Do not relabel a device-origin refresh credential as browser-authorized to evade grant-specific controls. Explicit `auth login --device` starts that flow even when usable credentials exist, making it testable; ordinary intake may reuse compatible cached credentials from either accepted flow.

## Cross-platform credential cache

Keep one reusable cache contract with target-specific store adapters. The selected dependencies and target feature wiring are recorded in the SharePoint intake capture contract. The current [Rust Keyring documentation](https://docs.rs/keyring/latest/keyring/) recommends `keyring-core` plus selected store implementations when applications need explicit backend control; evaluate that approach rather than assuming an older crate API or enabling every backend. No backend dependency may leak into the portable/WASM path, and ACORN adds no unsafe native bindings of its own.

| Platform | Protected persistence target | Native acceptance required |
|---|---|---|
| macOS | User Keychain, with non-synchronizing application entries | Locked/unlocked access, OS prompt denial, process restart and sign-out/re-login persistence, replace/delete and secret redaction |
| Windows | User Credential Manager generic credential, local-machine persistence for the current user; no enterprise roaming | Native credential read/write/update/delete, unloaded/unavailable profile, process restart and sign-out/re-login persistence, isolation and supported payload-size limits |
| Linux | A supported persistent Secret Service collection over the user's session D-Bus, initially GNOME Keyring | Available/unavailable bus/service, locked collection and cancelled unlock, process restart and session re-login persistence, replace/delete and headless behavior |

These are platform-specific implementations of one contract. See [Apple Keychain services](https://developer.apple.com/documentation/security/keychain-services), [Windows credential persistence](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw), and the [Secret Service API](https://specifications.freedesktop.org/secret-service/latest/). Linux acceptance must name the tested service/version and its persistent collection; the API alone does not establish the implementation's at-rest protection. A session-only kernel keyring does not satisfy restart/re-login persistence. Additional Linux store implementations need their own evidence before being advertised.

The frozen contract requires:

- Non-secret profile policy offers **preferred**, **required**, and **disabled** persistence. Preferred uses the native store when available and reports session-only operation if unavailable; required fails clearly when storage cannot be read or updated; disabled neither reads nor writes persistent credentials. Preferred is the default. No fallback writes plaintext, creates a homemade encrypted file, or switches to another OS user's store.
- Store only the minimal refresh material and associated protected metadata needed by the selected OAuth library. Keep access tokens in memory. Version and namespace entries under ACORN; bind them to authority/tenant/client/account, with resource/scope compatibility enforced during token acquisition. Entry labels and lookup attributes contain no secrets or unnecessary personal details.
- Distinguish missing, locked, unavailable, denied, malformed, unsupported-version and oversized entries. Bound caller wait and cancellation, and account for blocking OS APIs that may not abort when an async task is cancelled. Do not equate a timeout with confirmed deletion or grant permission for late writes; mutations must remain serialized through completion or report an unresolved outcome before a retry. Native acceptance verifies actual prompt/call behavior. Native unlock/consent dialogs are allowed only in an interactive session after the Confirm flow has explained the action; noninteractive execution never waits for a desktop dialog. Never truncate credentials to fit a backend's limits.
- Use the store/library's supported replace semantics and a bounded per-identity cross-process refresh coordination strategy. Re-read under coordination before refresh; prevent stale writers from replacing newer refresh material. Profile-scoped removal must not race an in-flight refresh into recreating a signed-out cache entry. Freeze an implementable locking/generation design before backend work; retain no secrets in coordination files.
- Removal deletes only the selected ACORN identity/profile records and never an entire user's vault. Explain that local removal does not revoke remote sessions. Do not delete malformed/unrecognized records automatically; provide a clear recovery path. Tests use isolated ACORN test namespaces and clean up only their own entries.
- Native success requires retrieving a previously stored refresh credential in a new process and successfully obtaining a new access token, not merely retrieving a dummy string or reusing a still-live in-memory token. Separately test OS store lifecycle with synthetic secrets. Never publish real credential material as evidence.

Headless Linux, SSH sessions, containers, WSL and service accounts are not assumed to have a usable desktop vault. Detect capabilities and honor the selected persistence/interaction policy. Native Windows and Linux desktop acceptance are release gates; untested WSL/container/service deployments remain explicit limitations. Cross-platform support means the same behavior on each supported OS, not moving or synchronizing credentials between machines. Unattended bot authentication remains outside this plan.

## Credential-cache inspection and reset for testing

Proposed commands, **not implemented yet** (profile and account names are examples):

```sh
acorn auth login --device
acorn auth login --profile ornl --device
acorn auth cache status
acorn auth cache clear --profile ornl --account ACCOUNT_ID --dry-run
acorn auth cache clear --profile ornl --account ACCOUNT_ID
acorn auth cache clear --profile ornl --account ACCOUNT_ID --yes
```

`login` resolves the first declared profile unless `--profile NAME` is provided, then runs explicit authorization using that profile's flow (browser when omitted) or device authorization when `--device` is present. It uses Confirm, saves credentials only under the profile's cache policy and reports success without printing tokens. It does not download, ingest or mutate a SharePoint document. `status` performs a bounded local inspection without refreshing tokens or contacting the identity provider: report backend, policy, stable account selector, matching entry count and missing/locked/unavailable states. Never show tokens, codes, secret lengths or raw vault contents.

`clear` uses the explicit `--profile` or the first declared profile, just like login and status; always show the resolved profile/configuration path in the deletion preview. An omitted account is allowed only if exactly one identity matches; otherwise report account selectors and require a choice. Compute an exact deletion set of ACORN-owned credential entries and local identity metadata, show its profile/account/backend and count, then use the existing Confirm control with No focused. `--dry-run` reports that set without deleting, invalidating generations or starting sign-in. `--yes` explicitly authorizes the already-scoped local removal for automated tests; it cannot override an invalid/missing profile or account ambiguity or authorize other applications' entries. Without a TTY, require `--yes` for mutation and never trigger vault-unlock dialogs. No broad `--all` or whole-vault clearing is included.

Clearing covers the selected identity's ACORN OAuth credentials across both grant types; report when other profile aliases reference the same cached identity. Hold the shared identity coordination lock, advance its non-secret cache generation, remove only matching credential/account records and verify their absence. Retain the minimum generation tombstone needed to reject stale writers; clearing must not delete its own race protection. In-flight refresh or sign-in attempts from an older generation must not recreate the cleared cache or reuse stale credentials in subsequent ACORN operations. The calling process drops its memory cache; other cooperating processes must check generation before using or writing credentials. Clearing cannot recall an API request already sent or revoke a token already held by another application. Missing entries are idempotent success; locked/denied stores and partial deletion return an explicit failure with remaining scoped records, without credential contents.

The existing global `--clear-cache`, `--no-clear-cache` and cache TTL flags do not clear or expire OAuth credentials. Keep their current behavior and test that they cannot trigger credential removal. `auth login --offline` fails before Confirm/network/cache mutation; `auth cache status/clear --offline` remain supported without application-cache initialization or unsolicited stdout banners.

The operation works offline and must leave JSONL snapshots, intake ledgers, application configuration, GitLab PATs, other profiles/accounts and other applications' credentials unchanged. It does not revoke Microsoft consent, invalidate remote sessions or remove browser cookies. A fresh OAuth exchange after clearing may still use browser SSO; do not promise a password/MFA prompt. Test wrong-account and fresh-consent behavior with a separate test browser/account when needed.

For repeatable tests: use an isolated ACORN test profile/namespace; inspect -> dry-run -> clear -> verify missing -> explicitly log in with the chosen flow -> start a new process and verify cached refresh -> clear again. The profile's disabled persistence policy supports tests that must never read/write persistent credentials. Disabled ordinary auth must not prevent an explicit scoped `cache clear` from removing older persisted entries; that command uses its own explicit removal authorization. Native acceptance on all three platforms exercises the public clear command and concurrent refresh/removal, preserving unrelated seeded test entries. Do not clear the user's operational credentials just to execute acceptance tests.

## Dependency selection and wiring

The selected OAuth/OIDC, keyring and target-specific store versions, features, licenses, MSRVs, runtime costs and rejected HTTP adapters are recorded in the [SharePoint intake capture contract](../../../docs/src/cli-reference/commands/sharepoint-intake.md#dependency-decision-and-validation). `oauth2` and `openidconnect` remain in use. The native-store dependencies are deferred until their implementation phases: SPD-24 restores `keyring-core` 1.0.1 for the shared store interface; SPD-05 restores `apple-native-keyring-store` 1.0.2 with only `keychain` on macOS; SPD-14 restores `windows-native-keyring-store` 1.1.0 with defaults disabled on Windows; SPD-15 restores `zbus-secret-service-keyring-store` 1.0.1 with only `rt-tokio-crypto-rust` on Linux. Add each optional dependency to the host `auth` feature and update `Cargo.lock` only when production code uses it. Keep target-specific dependencies out of the portable/WASM path and run `make check-dependencies` after each manifest change. Linux and Windows still require target/native acceptance in their later phases. No all-backends keyring facade or second OAuth HTTP stack was added.

## Interactive sign-in and code presentation

The first delegated sign-in, explicit `auth login`, additional consent, or an expired/revoked session can require interaction. During ordinary intake, a usable cached token or successful silent refresh should proceed without another Confirm prompt. Host authentication exposes typed interaction-needed events and a continuation/cancellation result; terminal UI remains in `acorn-cli`, with no dependency from the host library back to the CLI.

When interaction is required, show the application/profile, identity provider, target resource and a short permission explanation, followed by the existing Confirm control: **“Open your browser to sign in?”** Use the shared filled Yes/No buttons with No focused initially and existing arrows/Tab, y/n, Enter and Escape behavior. No, Escape or interruption cancels without opening the browser or starting the interactive authorization exchange. Microsoft performs actual sign-in and consent in its browser page; the local Confirm button authorizes launching that interaction, not granting API permissions.

For the browser flow, on Yes launch the configured browser flow and show a bounded “Waiting for sign-in…” state using the existing Spin control. Restore the terminal on success, denial, timeout, cancellation or browser-launch failure. If opening fails, offer the validated authorization link for manual opening in the interactive terminal; keep authorization URLs out of logs and receipts. Noninteractive or unavailable-TTY runs must return a clear sign-in-required error when silent acquisition fails. Do not hang, auto-approve, or treat `--batch` as consent.

The browser authorization-code/PKCE flow receives its code through the browser callback; **do not print that authorization code, a PKCE verifier, tokens, or an internal device code**. The device authorization option supplies `user_code`, `verification_uri` and expiry to the same reusable presenter; display those intended user instructions in the titled panel. This grant is part of the planned release and requires both protocol and live acceptance. Microsoft's [device-code documentation](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code) distinguishes the displayable `user_code` from the private polling `device_code`. Device code remains explicitly selected and policy-approved, never an automatic response to a Conditional Access denial.

Illustrative layout with a fake code (shown only when a supported flow supplies a user code):

```text
╭─ Sign in to Microsoft ─────────────────────────╮
│ Open: https://microsoft.com/devicelogin         │
│                                               │
│ Code: ABCD-EFGH                                │
│                                               │
│ Enter this code in your browser.               │
│ Expires in 10 minutes.                         │
╰───────────────────────────────────────────────╯
```

Use the actual validated provider URL, exact user code and server-provided expiry; the example URL/time are not defaults. Keep the code prominent and copyable, with padding, readable contrast and a compact title border. Support Unicode and ASCII borders, no-color mode and terminal resizing. At narrow widths, switch to labeled unboxed text rather than truncating the code or verification URL. Sanitize provider text/control sequences. Show waiting/success/expired/cancelled status outside the copyable code; expiry/cancellation must stop device-grant polling immediately, and any late response must be discarded without caching credentials. Do not copy codes to the clipboard automatically or persist them in logs, transcripts or receipts.

All interactive controls and instructions use stderr/the interactive terminal; stdout remains the existing machine-readable batch result. Add a selectable output stream to the shared renderer instead of duplicating Confirm. Preserve its current stdout behavior for existing callers, and test cleanup against the selected stream. When stdin/stderr are TTYs, redirected stdout must remain usable for JSON output; if interaction is impossible, fail clearly without emitting terminal escape sequences into redirected output.

## Download and snapshot integrity

1. Read item metadata, recording site/drive/item identity, size, modification time and revision/ETag. Prefer a verifiable immutable version endpoint where available; otherwise compare metadata before and after content capture. An ETag is an opaque revision validator, not a Git commit or a promise of historical retrievability. Record whether evidence is a pinned version or a consistent before/after observation; the local byte snapshot is immutable even when the remote observation is optimistic. Require stable item identity, size and revision across capture; missing revision evidence is an explicit unsupported-verification error, not a verified snapshot.
2. Fetch content using bounded time, redirects, response size and retry limits. Graph's content API redirects to a short-lived preauthenticated URL. Follow only the validated provider redirect policy over HTTPS; do not forward Graph authorization to the destination, log the signed URL, or accept arbitrary redirects into loopback/private services. See the [download API redirect contract](https://learn.microsoft.com/en-us/graph/api/driveitem-get-content?view=graph-rest-1.0).
3. The Graph service returns bounded exact bytes and revision metadata without knowing their file format; the intake adapter then validates UTF-8/JSONL. Preserve the original UTF-8 bytes, including Unicode and LF delimiters. Enforce the existing 10 MiB source and 64 KiB record limits. Reject invalid UTF-8, login HTML, malformed/truncated JSONL, and size/integrity mismatches with no ready snapshot. Use the existing intake validator for envelope/schema acceptance; do not invent a second LogbookEntry model.
4. If the item changes during capture, discard the temporary result and retry a bounded number of times; report an unstable export after exhaustion. Do not combine rows from different revisions. A Graph 401 may refresh at most once per request sequence; an expired signed-download URL instead reacquires content metadata through Graph and restarts the consistency check, without sending OAuth credentials to that URL. Distinguish permission denial from missing items, and bound 429/503 retries with `Retry-After` and an overall deadline.
5. Materialize `logbook.jsonl` and a versioned `source.json` receipt atomically beneath `.acorn/intake/sources/`. Retain provider, canonical source identity, revision evidence, byte/record counts, source modification time, capture time and SHA-256. Never put tokens or preauthenticated URLs in the receipt. Preserve the existing GitLab receipt format. Do not overwrite an existing verified snapshot; reuse identical content only after verifying its stored receipt and digest. Snapshot identity includes provider/tenant/source identity, remote revision and byte digest, excluding capture time. Identical bytes from different sources must not share provenance. Write both files into a restricted temporary directory on the same filesystem, then publish it atomically without replacing an existing final directory; reject symlink/reparse-point redirection and verify existing contents before reuse. Use the frozen Windows/POSIX permission and crash-recovery behavior. Keep first-capture time unchanged on reuse and report the latest verification time separately.
6. Distinguish downloaded, verified and staged states. Failed byte/framing validation or interrupted persistence must not leave a ready receipt, mutate the intake ledger, or erase a prior good snapshot. A verified capture is not an accepted submission: retain the existing staging outcomes for schema errors, duplicate/conflicting IDs, routing and authority failures instead of deleting a valid captured file or silently accepting its rows. Local project/actor/authority checks still apply after download: read access to a SharePoint file does not confer intake authority.

`--offline` must reject a remote URL before authentication, directory creation or network calls. A captured local snapshot must remain usable offline after token expiry. A cached file must never be silently substituted for a failed fresh download. Display export age without calling it the latest list state; a missing modification time is unknown, not current.

# Boundaries and dependencies

- This plan owns reusable host OAuth 2.0 mechanics, the Entra profile, read-only Graph file capture and the SharePoint CLI source adapter. [Power Platform intake](../powerapps-jsonl-intake/index.md) owns exporter correctness, recorded standard-user access/licensing acceptance, accepted aggregate boundaries/failure handling, backups and any cutover. A successful CLI read does not replace producer acceptance or close deployment phases or establish Power Apps licensing.
- [Manual ingestion documentation](../manual-jsonl-ingestion-graduation/walkthrough.md) remains the downstream workflow. No new graduation API, callbacks, Teams messages, email, report publication, or automatic source handback is introduced.
- [Vergil automation](../vergil-gitlab-jsonl-automation/index.md) remains independent. Unattended application credentials, scheduler integration, SharePoint writeback, direct list-to-JSONL construction, generic file downloads and cross-tenant discovery are excluded.
- The separate [headless CI SharePoint download plan](../sharepoint-headless-ci-download/index.md) depends on SPD-26: shared functionality, SPD-10 live handoff, SPD-22 device/reset acceptance and any one of SPD-16, SPD-17 or SPD-18 native validation. It does not wait for the other two native validations or SPD-11/12. It subsequently adds GitLab OIDC federation/application authentication and opt-in CI staging; none of its phases is a prerequisite for SPD. Preserve the accepted SPD-26 commit/evidence in that plan when pruning the phase, and replace the plan link at eventual SPD retirement.
- Consume landed `acorn-host` HTTP services and `acorn-lib` provider seams. Coordinate exact shared files with the [shared API contract](../../refactors/api/shared-client-contract.md), [CLI composition](../../refactors/core/cli-composition-boundary.md), and [parallel execution locks](../../parallel-execution.md). Their entire plans need not finish first. Keep the OAuth capability reusable through typed configuration and injected token acquisition. Do not extract `acorn-api`, build a universal provider/grant framework, or rewrite bucket downloads for this adapter. Coordinate authentication and configuration ownership with API-06/API-14 before shared-root edits; preserve their existing contracts rather than introducing a competing configuration system.
- The immediate external uncertainty is an approved CLI identity that can read this file. SPD-01 records that evidence or the precise denial. Fixture/contract work may proceed without a live tenant grant; live acceptance and release may not. No administrator ticket is assumed to exist, and no policy bypass is part of the plan.

# Acceptance matrix

| Area | Required evidence |
|---|---|
| Identity and permissions | Validated issuer/subject identity, client/tenant/nonce checks as appropriate, expired/wrong-key/wrong-account tests, and opaque API access-token handling. Owner signs in with the approved public client; scopes/cloud are recorded without tokens; a separately authorized intended operator reads the same file; a denied account fails clearly. Owner success alone is labeled as such. |
| Profile loading | ACRE `auth` parsing, discovery/explicit-file precedence, first-declared-profile defaults and named overrides, schema export and order-preserving JSON/JSONC/YAML round-trips; documents without `auth` retain behavior. Same-name profiles in different repositories and endpoint edits cannot redirect cached credentials; missing/ambiguous mapping fails before credentials/network. |
| Source routing | Local and GitLab regressions pass; SharePoint direct URLs resolve spaces/Unicode; unconfigured origins, malformed URLs and sharing/view links fail before credential use. |
| Reuse boundaries | Two synthetic OAuth issuers and two distinct resource audiences/scopes use the same authentication implementation; incompatible grants never collide in the cache; Entra configuration contains no ORNL/SharePoint constants. A non-JSONL byte fixture passes the Graph file service while intake rejects it. No extra live service permission is requested for these tests. |
| Device grant | Confirm before code request; exact user-code/URL display; pending/slow-down/denial/expiry, bounded network retry and cancellation; no private code/token leakage or cache writes from late responses; live approved-tenant proof. |
| Cache reset | Scoped status/dry-run/clear, No/Escape/yes behavior, noninteractive `--yes`, offline operation, ambiguity and shared aliases, idempotence, partial failures, concurrent refresh/sign-in invalidation, generation tombstone persistence, existing global cache-flag independence and preservation of unrelated credentials/snapshots on each native OS. |
| Sign-in UX | Existing Confirm reused; default No, Yes/browser launch, Escape/Ctrl-C, silent reuse without prompts, Spin cleanup, denied/expired flow and launch failure. PTY tests verify clean JSON stdout with interactive stderr, no-color/ASCII/narrow layouts, exact synthetic user-code display and no secret output. |
| HCI prerequisite handoff | SPD-26 pins shared contracts and validation, live handoff/device evidence and one accepted native platform. Unvalidated OSes remain explicit; no all-platform completion claim or dependency on SPD-11/12. |
| Native cache platforms | macOS Keychain, Windows Credential Manager and Linux Secret Service each pass create/read/replace/remove, separate-process and re-login persistence, actual OAuth refresh, locked/denied/unavailable states, cross-account isolation and bounded calls. Synthetic tests cover payload limits, corruption, concurrent refresh and removal races. Memory-only fallback and cross-compilation do not count as native persistence acceptance. |
| Credentials | PKCE/state/redirect validation; cancellation and expired/revoked tokens; bounded refresh; unavailable secure store; account/tenant separation; redacted errors at maximum verbosity. |
| Transport | 401/403/404/429/503, redirect loops and unsafe targets, no bearer forwarding, size limits, interrupted transfer, and HTML masquerading as successful content. |
| Consistency | Stable exact download; append/replacement/deletion during capture; bounded unstable-revision failure; altered size/ETag; no mixed-version or partial ready artifact. |
| JSONL | Empty logbook has explicit zero-record behavior; malformed line, invalid UTF-8, duplicate/conflicting IDs, CRLF input policy, Unicode, 64 KiB record and 10 MiB total boundaries are covered without rewriting bytes. |
| Local handoff | Original bytes/digest match; receipt provenance is complete; repeated capture/staging does not duplicate accepted intake; interrupted persistence preserves prior evidence; offline staging succeeds without contacting Microsoft. |
| Operations | Capture reads the current export without invoking flows or modifying source files; snapshot age and limits are visible; both existing routes remain operational. |

# Execution order and release gates

The contract, synthetic fixtures, feature-gated seams, ACRE auth profiles, verified identity adapter and fixture-tested device grant are landed. SPD-01 remains blocked on an approved public-client registration and delegated Graph read consent; this does not block synthetic implementation. SPD-24 next implements cache coordination. The native cache leaves then share that contract; neither Windows nor Linux implementation needs the macOS leaf to finish. The browser grant remains a separate implementation. SPD-20/21 compose explicit cache clearing and auth commands after those leaves; native and device live acceptance remain external gates.

The full cross-platform CLI download release and SPD retirement require all three native cache acceptances, real SharePoint handoff and a successful approved device-grant test. The earlier SPD-26 handoff requires the shared implementation, SPD-10, SPD-22 and any one of SPD-16, SPD-17 or SPD-18. That gate unlocks HCI independently of SPD-11/12 and the remaining native acceptance phases; it does not claim those platforms are accepted. The platform providing SPD evidence need not match the later Linux CI runner, whose headless workload behavior is separately verified by HCI-06. These gates do not block the existing browser-download/manual-report workflow; the accepted PA-22 evidence independently proves standard participant access and the recorded isolated producer acceptance covers completeness/failure behavior; PA-25 operational installation is also accepted. A tenant denial is recorded accurately, never counted as successful device authorization; it does not prohibit merging fixture-tested work or demonstrating an accepted browser path with its limitations labeled. Neither path may change operational Power Platform components. No full-workspace test result or live OAuth acceptance is claimed by this planning review.

# Execution phases

New files below are proposed ownership boundaries, not files claimed to exist. Each phase gets at most five active hours; external consent/host waiting is not active work. Register each leaf module and its dependency with the implementation that uses them so each slice compiles and passes the dependency gate independently. If dependency evaluation or a security defect needs more time, create a new bounded phase before expanding the write set. Every handoff updates this plan and its portfolio links; evidence belongs in durable documentation, with sanitized supporting files in this plan's `_assets/` only when needed.

## SPD-01 — Prove approved read-only tenant access

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: An approved Entra public-client registration and delegated read consent for the operator and export. The operator has an enterprise SharePoint account but does not know of such a registration; see the [sanitized auth evidence](_assets/auth-evidence.json).
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md` (new feasibility/operations record); this plan's `_assets/auth-evidence.json` (sanitized); authorized Entra desktop-app registration and read-only Graph access to the ResearchEnablement export, if permitted.
- Primary integration lock: Entra CLI identity
- Outcome: Verify tenant/cloud, registration, redirect, delegated scopes, and a metadata/content read using an approved client. Record browser and device-grant registration/policy availability, the minimum sufficient route and whether site/drive IDs must be configured; make no operational Power Platform changes.
- Focused validation: Compare downloaded bytes/digest to the browser export at the same revision; demonstrate an authenticated Graph request rather than browser-cookie reuse; record exact denial if blocked.
- Handoff: Non-secret identifiers and consent/access evidence, or a precise external prerequisite. No live-read or licensing claim without evidence.

## SPD-24 — Implement shared cache policy and refresh coordination

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: The resolved profile policy and verified identity contract are landed; cache seams are available.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/auth/cache/mod.rs`, `crates/acorn-host/src/auth/cache/coordination.rs`; `tests/fixtures/oauth2/cache/lifecycle/`.
- Primary integration lock: Host credential cache lifecycle
- Outcome: A cache-aware token acquisition service using injected grant/refresh and store interfaces: preferred/required/disabled policy, identity namespacing, minimal non-secret identity index, refresh rotation, per-identity coordination and generation tombstones shared by all stores/grants. Restore and use `keyring-core` 1.0.1 in the shared store interface. No native backend or grant protocol implementation in this phase. Define late-completion behavior for blocking store calls and safe metadata writes with protected paths.
- Focused validation: Fake-store tests for concurrent refreshes, cancellation, lock timeout/recovery, stale generation, shared profile aliases, no plaintext fallback, disabled read/write behavior, malformed metadata, path redirection and bounded index enumeration; `make check-dependencies`.
- Handoff: Stable cache lifecycle for SPD-05/14/15; SPD-20 later composes explicit clear and binds both grant paths. Any additional locking dependency returns to the manifest owner.

## SPD-04 — Implement reusable interactive OAuth sign-in

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: The ACRE configuration and identity verifier are landed; fixture implementation does not require live sign-in.
- Exact write set: `crates/acorn-host/src/auth/oauth2.rs`; `tests/fixtures/oauth2/browser/`.
- Primary integration lock: Host OAuth authentication
- Outcome: Provider-configured browser sign-in, reusable refresh-token exchange and in-memory token acquisition with state/PKCE/redirect validation, bounded listener lifetime, cancellation and explicit noninteractive failure. Entra authority/cloud/tenant policy is a separate profile; callers provide resource/scopes, never SharePoint-specific auth logic. Return a typed interaction-needed event before launching a browser; resume only after caller confirmation, and propagate cancellation without embedding terminal controls.
- Focused validation: Mock two issuers and resource/scope requests, wrong state, callback errors, timeout/cancel, wrong tenant/audience configuration, OIDC ID-token misuse and redaction; live approved sign-in when available.
- Handoff: Reusable memory-only authentication usable without a persistent cache or SharePoint dependency.

## SPD-05 — Implement the macOS Keychain adapter

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-24 shared cache contract; restore target wiring with the adapter.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/auth/cache/mod.rs`, `crates/acorn-host/src/auth/cache/macos.rs`; `tests/fixtures/oauth2/cache/macos/`; macOS cache section of `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Host OAuth authentication
- Outcome: Restore and use `apple-native-keyring-store` 1.0.2 with only `keychain` in a macOS-targeted dependency and host `auth` feature. Implement non-synchronizing macOS Keychain entries against the shared lifecycle and typed failure contract. Preserve profile/account isolation, supported replace/remove behavior and explicit prompt/locked-store handling. No plaintext fallback or shared coordination rewrite.
- Focused validation: Mock locked/missing/denied store, account separation, replacement/removal, payload bounds, malformed entries, prompt cancellation, secret redaction and target gating; `make check-dependencies`. Native Keychain and real OAuth acceptance belong to SPD-16.
- Handoff: Focused macOS adapter evidence for SPD-16; Windows/Linux leaves independently consume SPD-24.

## SPD-14 — Implement Windows Credential Manager caching

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-24 shared cache contract; restore target wiring with the adapter.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/auth/cache/mod.rs`, `crates/acorn-host/src/auth/cache/windows.rs`; `tests/fixtures/oauth2/cache/windows/`.
- Primary integration lock: Windows credential cache leaf
- Outcome: Restore and use `windows-native-keyring-store` 1.1.0 with defaults disabled in a Windows-targeted dependency and host `auth` feature. Implement user-scoped generic credentials with non-roaming local persistence, versioned namespacing, replace/remove semantics and typed unavailable/profile/permission/size failures. Consume shared coordination and policy behavior without changing OAuth protocol code.
- Focused validation: Mock store tests and Windows-target compilation; missing and corrupt entries, credential size boundaries without truncation, account/identity collisions, update/delete and error redaction; `make check-dependencies`.
- Handoff: Testable Windows adapter; SPD-17 owns native proof, and compilation alone is not acceptance.

## SPD-15 — Implement Linux Secret Service caching

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-24 shared cache contract; restore target wiring with the adapter.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-host/src/auth/cache/mod.rs`, `crates/acorn-host/src/auth/cache/linux.rs`; `tests/fixtures/oauth2/cache/linux/`.
- Primary integration lock: Linux credential cache leaf
- Outcome: Restore and use `zbus-secret-service-keyring-store` 1.0.1 with only `rt-tokio-crypto-rust` in a Linux-targeted dependency and host `auth` feature. Implement persistent user Secret Service collection support with bounded D-Bus access, explicit unlock/denial behavior and the shared persistence policy. Missing service, locked collection, SSH/headless context and session-only storage produce accurate capabilities or errors.
- Focused validation: Mock service/collection/prompt responses, unavailable bus, cancelled unlock, timeout, malformed entry, replace/remove and redaction; Linux-target compilation, no-default-feature checks and `make check-dependencies`.
- Handoff: Testable Linux adapter; SPD-18 owns native persistence proof. Do not auto-install or unlock a user's service or claim support for untested store implementations.

## SPD-06 — Resolve the configured SharePoint document

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: The ACRE source configuration and frozen path/identity contract are landed; fixture implementation does not require real resource mapping.
- Exact write set: `crates/acorn-lib/src/io/api/sharepoint/resolve.rs`; `tests/fixtures/sharepoint-intake/resolve/`.
- Primary integration lock: SharePoint provider leaf
- Outcome: Bounded Graph metadata resolution returning stable site/drive/item identity and revision evidence from a permitted direct URL or configured IDs.
- Focused validation: Spaces/Unicode, path ambiguity, pagination overflow, deleted/moved files, insufficient discovery permission, pinned-ID mismatch and hostile origins using a mock server.
- Handoff: Typed metadata and actionable resolution errors without widening scopes.

## SPD-07 — Capture bounded content with revision proof

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-04 and SPD-06; use the frozen transfer contract.
- Exact write set: `crates/acorn-lib/src/io/api/graph/drive.rs`; `tests/fixtures/microsoft-graph/`.
- Primary integration lock: Graph file transfer
- Outcome: Format-independent exact bytes and consistent revision evidence by drive/item identity using existing HTTP policy and an injected resource-scoped token capability. Refresh/retry/redirect handling stays bounded and cannot leak credentials; a changing export fails safely after the configured retry budget.
- Focused validation: Mock both JSONL and unrelated binary content with identical transfer mechanics, redirect and bearer isolation, 401/403/404/429/503, unsafe destinations, oversized/interrupted content, changed revision and repeated instability. If shared HTTP changes are required, schedule a separate transport phase instead of silently widening this leaf.
- Handoff: Reusable Graph file result; no JSONL parsing, auth implementation or local staging side effects in this service.

## SPD-08 — Persist complete immutable local snapshots

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-07 and frozen receipt contract.
- Exact write set: `crates/acorn-cli/src/commands/intake/sharepoint.rs` (new); `crates/acorn-cli/src/commands/intake/mod.rs` (private module registration only); `tests/fixtures/sharepoint-intake/snapshots/`.
- Primary integration lock: Intake snapshot persistence
- Outcome: Source validation and atomic byte/receipt materialization with deterministic content identity, protected local files, no overwrite and no partial ready states. Preserve GitLab persistence behavior.
- Focused validation: Empty/malformed/Unicode/boundary records, duplicate/conflicting identities through existing validators, digest mismatch, existing snapshot verification, concurrent local captures and failures between file/receipt writes.
- Handoff: A local captured path and versioned provenance available to existing intake stages.

## SPD-13 — Reuse Confirm for OAuth and add a titled sign-in panel

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-04 host interaction events and an exclusive shared terminal-controls window; use the frozen interaction contract.
- Exact write set: `crates/acorn-cli/src/io/mod.rs`, `crates/acorn-cli/src/io/auth.rs` (new presenter), `crates/acorn-cli/src/io/interactive/mod.rs`, `crates/acorn-cli/src/io/interactive/tests.rs`, `tests/fixtures/oauth2/presentation/` (new synthetic fixtures).
- Primary integration lock: CLI interactive controls
- Outcome: Reuse Confirm/Spin through a tested selectable output stream and a provider-neutral OAuth presenter. Show profile/resource context, ask before interactive sign-in, and render an optional user code in the titled panel with accessible fallbacks. Existing commands retain their current behavior. Provide grant-specific Confirm text and the device-code panel through typed events from the landed device grant; the presenter never requests extra permissions.
- Focused validation: Renderer and PTY tests for Yes/No/default Enter/Escape/Ctrl-C, no browser launch on decline, cached-token bypass, waiting-state cleanup, stdout JSON with stderr TTY, no TTY, color/no-color/ASCII, resize/narrow widths, exact code/URL display, provider control-character sanitization and absence of credentials from logs. Use fake code data only; run existing Confirm tests for regressions.
- Handoff: Reusable CLI presentation functions ready for SPD-09 and SPD-21; the device grant supplies typed events and SPD-22 proves the live interaction.

## SPD-20 — Implement scoped cache inspection and clearing

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-24 shared coordination contract, SPD-05/14/15 adapters and SPD-04 browser grant; the device grant is landed.
- Exact write set: `crates/acorn-host/src/auth/cache/manage.rs`, `crates/acorn-host/src/auth/cache/mod.rs`, `crates/acorn-host/src/auth/cache/coordination.rs`, `crates/acorn-host/src/auth/oauth2.rs`, `crates/acorn-host/src/auth/device.rs`; `tests/fixtures/oauth2/cache/manage/`.
- Primary integration lock: Host credential cache lifecycle
- Outcome: Compose both real grants and native adapters with the SPD-24 acquisition service, then add bounded local inspection, deterministic scoped deletion plans, dry-run support and generation-aware removal that prevents in-flight sign-in/refresh from restoring cleared entries. Extend both grant paths to honor the generation boundary; no remote revocation or whole-vault operations.
- Focused validation: Mock multiple accounts/profiles/aliases, ambiguous/missing/corrupt records, partial deletion, locked/unavailable stores, offline operation, idempotence, refresh/sign-in races and preserved unrelated entries. Assert dry-run changes no generations and network requests remain zero.
- Handoff: Reusable management service ready for CLI binding and all three native cache acceptance phases.

## SPD-21 — Expose login and testing cache controls in the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-13 presenter and SPD-20 management service; the device grant and ACRE configuration are landed. Requires an exclusive CLI parser/dispatch window.
- Exact write set: `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/main.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/auth/{mod.rs,tests.rs}` (new), `crates/acorn-cli/src/io/mod.rs`, `crates/acorn-cli/src/io/auth/{mod.rs,config.rs}`, `CommandLineHelp.md`, `docs/src/cli-reference/commands/auth.md` (new).
- Primary integration lock: CLI contract
- Outcome: `auth login [--profile NAME] [--device]`, `auth cache status` and scoped `auth cache clear --dry-run/--yes` follow the frozen contract, reuse Confirm/panels and report metadata only. Register the commands using the repository's current dispatch mechanism and make the tested CLI auth resolver a production module when this is its first caller; no token output command.
- Focused validation: Parser/dispatch tests and fake-service PTY scenarios for default/explicit profile selection, `--device` precedence over profile flow, browser default, rejection of the superseded `--flow` syntax, account ambiguity, default No/cancel, explicit unattended clear, dry-run/no-network behavior, secret-free stdout, actual device events in the titled panel and help regeneration via `make help`. Verify auth commands are routed before unrelated project/database initialization, offline status/clear produce clean output, and existing global cache flags never delete credentials.
- Handoff: Reviewable login/reset CLI for real acceptance; local clear is explicitly distinguished from browser logout or remote revocation.

## SPD-09 — Connect SharePoint to the intake source URI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-05, SPD-08, SPD-13, SPD-14, SPD-15 and SPD-20; the device grant and ACRE configuration are landed. Requires an exclusive intake source window.
- Exact write set: `crates/acorn-cli/src/commands/intake/source.rs`, `crates/acorn-cli/src/commands/intake/sharepoint.rs`, `crates/acorn-cli/src/io/mod.rs`, `crates/acorn-cli/src/io/auth/{mod.rs,config.rs}`; `tests/fixtures/sharepoint-intake/cli/`.
- Primary integration lock: Intake source routing
- Outcome: Existing `intake stage --source` selects SharePoint, resolves non-secret configuration once, uses silent authentication when available or the explicitly configured browser/device flow through the SPD-13 Confirm presenter when interaction is required, then captures and stages locally. Make the tested CLI auth resolver a production module when this is its first caller. Output names the snapshot and its freshness/provenance. Auth command registration belongs to SPD-21; no automatic intake application is introduced.
- Focused validation: Local/GitLab source regressions, offline failure before network/storage, unknown host before auth, fresh-versus-cached semantics, decline/cancel without starting sign-in, clean JSON stdout during interactive stderr rendering, repeat staging of a retained full snapshot, a later snapshot containing old and new UUIDs, same-ID changed-payload conflict, and local replay after token expiry; no list deletion or acknowledgement. Follow existing `Label` output conventions.
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

## SPD-22 — Accept live device authorization and a clean testing cycle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-01, SPD-09 and SPD-21; an approved test identity/client permitting device authorization, isolated cache namespace and interactive terminal with a separate browser available. Tenant permission is not assumed.
- Exact write set: Read-only approved Entra/Graph access for the test identity and named SharePoint export; isolated ACORN test credential namespace/workspace; this plan's `_assets/device-acceptance.json`; `docs/src/cli-reference/commands/auth.md`.
- Primary integration lock: Device authorization live acceptance
- Outcome: Prove Confirm -> titled user code -> browser authorization -> verified file read -> new-process credential reuse -> dry-run/clear -> fresh explicit device sign-in. Verify the browser can run separately from the CLI and no callback listener is required for device authorization.
- Focused validation: Success, user cancellation and expiration where practical; inspect denied/policy outcomes without trying to bypass them. Record registration, flow, code-free timestamps and sanitized results; verify no operational credentials or source bytes change. Native clear semantics on all OSes remain SPD-16/17/18.
- Handoff: Live evidence or precise unresolved policy prerequisite. A denial proves error handling, not successful device-grant acceptance; browser progress is not blocked, but this phase and device release acceptance remain open.

## SPD-16 — Accept native macOS caching and restart recovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-05, SPD-09, SPD-21 and SPD-01; an approved native macOS host/account with Keychain and tenant sign-in access. Host availability and user-driven session restart are external prerequisites, not assumed evidence.
- Exact write set: Isolated ACORN test namespace in the native macOS user's Keychain; temporary local test workspace; this plan's `_assets/cache-macos-acceptance.json`; macOS evidence section of `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Native macOS credential acceptance
- Outcome: Prove store create/read/replace/remove and persistence across a new process and sign-out/re-login; then retrieve a real refresh credential after process restart and acquire a fresh access token without unnecessary Confirm interaction. Record OS/architecture, backend version, persistence policy and sanitized results.
- Focused validation: Locked/denied/unavailable store, preferred versus required versus disabled behavior, cancellation/terminal cleanup, refresh expiry/revocation, identity isolation, public cache status/dry-run/clear, preserved unrelated entries and fresh authorization after clear. Cover Keychain access prompts and non-synchronizing entries. Use an isolated test user/session where needed; do not lock, log out or disrupt the operational browser/user session without coordination.
- Handoff: Dated native acceptance and tested-platform boundaries. Any missing host, denied tenant access or untested matrix row stays open; preserve zero secrets in evidence and remove only test-owned cache entries.

## SPD-17 — Accept native Windows caching and restart recovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-14, SPD-09, SPD-21 and SPD-01; an approved native Windows host/account with Credential Manager and tenant sign-in access. Host availability and user-driven session restart are external prerequisites, not assumed evidence.
- Exact write set: Isolated ACORN test namespace in the native Windows user's Credential Manager; temporary local test workspace; this plan's `_assets/cache-windows-acceptance.json`; Windows evidence section of `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Native Windows credential acceptance
- Outcome: Prove store create/read/replace/remove and persistence across a new process and sign-out/re-login; then retrieve a real refresh credential after process restart and acquire a fresh access token without unnecessary Confirm interaction. Record OS/architecture, backend version, persistence policy and sanitized results.
- Focused validation: Locked/denied/unavailable store, preferred versus required versus disabled behavior, cancellation/terminal cleanup, refresh expiry/revocation, identity isolation, public cache status/dry-run/clear, preserved unrelated entries and fresh authorization after clear. Cover user-profile availability, generic credential payload limits and non-roaming persistence. Use an isolated test user/session where needed; do not lock, log out or disrupt the operational browser/user session without coordination.
- Handoff: Dated native acceptance and tested-platform boundaries. Any missing host, denied tenant access or untested matrix row stays open; preserve zero secrets in evidence and remove only test-owned cache entries.

## SPD-18 — Accept native Linux caching and restart recovery

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-15, SPD-09, SPD-21 and SPD-01; an approved native Linux host/account with Secret Service (initially GNOME Keyring) and tenant sign-in access. Host availability and user-driven session restart are external prerequisites, not assumed evidence.
- Exact write set: Isolated ACORN test namespace in the native Linux user's Secret Service (initially GNOME Keyring); temporary local test workspace; this plan's `_assets/cache-linux-acceptance.json`; Linux evidence section of `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Native Linux credential acceptance
- Outcome: Prove store create/read/replace/remove and persistence across a new process and sign-out/re-login; then retrieve a real refresh credential after process restart and acquire a fresh access token without unnecessary Confirm interaction. Record OS/architecture, backend version, persistence policy and sanitized results.
- Focused validation: Locked/denied/unavailable store, preferred versus required versus disabled behavior, cancellation/terminal cleanup, refresh expiry/revocation, identity isolation, public cache status/dry-run/clear, preserved unrelated entries and fresh authorization after clear. Cover missing session bus, locked persistent collection, cancelled unlock and headless no-dialog behavior. Use an isolated test user/session where needed; do not lock, log out or disrupt the operational browser/user session without coordination.
- Handoff: Dated native acceptance and tested-platform boundaries. Any missing host, denied tenant access or untested matrix row stays open; preserve zero secrets in evidence and remove only test-owned cache entries.

## SPD-26 — Hand off shared functionality after one native acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-10 and SPD-22 with their shared implementation prerequisites; **any one** of SPD-16 (macOS), SPD-17 (Windows) or SPD-18 (Linux). This is an OR dependency. Neither the other two native acceptance phases nor SPD-11/12 is a prerequisite. Stable baseline and exclusive focused integration-check window.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md`, `docs/src/cli-reference/commands/auth.md`; this plan's `_assets/single-platform-handoff.json` (sanitized); this plan, `.wiki/features/sharepoint-headless-ci-download/index.md`, `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`.
- Primary integration lock: Shared-auth handoff validation.
- Outcome: Pin the accepted commit, shared ACRE/token/Graph/snapshot contracts, exact download/offline commands, SPD-10/22 evidence and the qualifying native platform/version/cache result. Publish sufficient durable interface and operator guidance for HCI to consume the baseline while remaining OS validation stays open. SPD-11 later completes the cross-platform guide; SPD-12 still owns final retirement. No new product implementation belongs to this phase.
- Focused validation: Run focused auth, configuration, Graph, intake and CLI regression tests on the accepted platform, plus affected portability/no-default-feature checks. Verify live handoff/device/native evidence applies to the pinned baseline; rerun affected acceptance if intervening changes invalidate it. Review examples, redaction, links and explicit unvalidated-platform limitations. Do not require unavailable native hosts. Unexpected product defects get a bounded repair phase.
- Handoff: Record durable acceptance in the guides and HCI prerequisite before pruning SPD-26. HCI-01 becomes eligible subject to its separate external access requirements. Keep remaining native phases and SPD-11/12 open; concurrent HCI changes must preserve the pinned contract and coordinate shared files.

## SPD-11 — Publish operator instructions and demo handoff

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-10, SPD-16/17/18 and SPD-22; accepted command/error behavior and all three native cache evidence records.
- Exact write set: `docs/src/cli-reference/commands/sharepoint-intake.md`, `docs/src/cli-reference/commands/auth.md`, `docs/src/cli-reference/commands/intake.md`, `.wiki/features/manual-jsonl-ingestion-graduation/walkthrough.md`, `.wiki/features/powerapps-jsonl-intake/sharepoint-pilot.md`, `docs/src/SUMMARY.md`, `README.md`.
- Primary integration lock: Documentation generation
- Outcome: Document ACRE `auth.profiles`/`auth.sources`, discovery, `--config`, optional `--profile` with first-declared defaults and `--device` selection, reusable auth profiles versus resource-specific scopes, OS-specific cache setup/removal, preferred/required/disabled policies, tested platforms and headless limitations, explicit browser/device sign-in, scoped status/dry-run/clear testing cycles and browser SSO limitations, the stage -> inspect -> apply -> report sequence with maximum supported defaults, exact download/offline commands, freshness, credential removal, Confirm interaction, conditional user-code presentation, noninteractive behavior, access troubleshooting and browser-download fallback. Explain extension points and tested scope without claiming other identity providers or API integrations are accepted. Link the guide from existing intake documentation; describe platform and consent limitations accurately.
- Focused validation: Follow examples against sanitized fixtures and the accepted live setup; verify links and absence of tokens/signed URLs; do not claim CLI download is available in previously generated PDFs without regenerating them in a separately scoped documentation task.
- Handoff: Durable instructions sufficient to use the feature independently of this plan.

## SPD-12 — Complete integration gates and retire the plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SPD-11 and SPD-26; stable implementation and exclusive final-check window.
- Exact write set: This plan and any sanitized supporting evidence; `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/features/powerapps-jsonl-intake/index.md`, `.wiki/features/sharepoint-headless-ci-download/index.md` (replace the SPD prerequisite link with accepted durable evidence); final validation record in `docs/src/cli-reference/commands/sharepoint-intake.md`.
- Primary integration lock: Workspace validation
- Outcome: Accepted feature with accurate limitations, durable evidence and no stale planning entries. Preserve evidence outside the active plan before removing it.
- Focused validation: Serialize `make lint`, `make test`, `make check-wasm`, relevant no-default-feature checks and wiki validation after focused tests pass. Check application/API behavior against the frozen contract; unexpected defects get new bounded repair phases rather than being waived.
- Handoff: Remove this plan and its index/network/lane references only after all acceptance criteria pass; retain any genuinely unfinished phase with its stable ID.
