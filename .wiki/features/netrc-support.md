---
type: Plan
title: Opt-in netrc authentication for native downloads
description: Add explicit netrc credential loading and origin-scoped HTTP Basic authentication to native source reads and remote BagIt downloads.
difficulty: medium
impact: medium
package: [acorn-host, acorn-lib, acorn-cli]
tags: [authentication, downloads, netrc, os:linux, os:macos, os:windows]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome and boundaries

Users can reuse a netrc file for HTTPS Basic authentication through explicitly configured native host clients and `acorn download` remote BagIt archive inputs. Existing callers remain anonymous unless they supply credentials. The CLI accepts `--netrc` or `--netrc-file PATH`; neither option exposes a password in command arguments or persists it in ACRE, RAD, logs, provenance, or activity records.

The first release covers remote archive acquisition, including content inference and forced archive formats, plus reusable `acorn-host` HTTP/source/download APIs. Repository enumeration, GitLab/Hugging Face provider requests, model/tool installation, authenticated link checking, OAuth sign-in, proxy authentication, FTP, cookie persistence, browser/WASM access and credential-file editing are outside this outcome. Unsupported CLI source/subcommand combinations must reject the flags rather than silently ignore them. Provider-specific credentials retain their current behavior.

# Current evidence

Reviewed November 11, 2026 against `43284095` and the working tree. Unrelated Makefile, browser documentation and intake evidence changes are outside this plan. The November 7 portfolio-wide review remains current; this is a targeted addition. Code inspection establishes structure, not a fresh passing test run.

| Verdict | Inspected source and observation | Implication |
|---|---|---|
| verified | `crates/acorn-host/src/http.rs`, `HttpClient::response`, `probe`, and `build_native_client`: requests and redirects use immutable host policy; the GET probe fallback builds its own request. | Integrate credentials into every opted-in request path, including the fallback, and preserve authority restrictions. |
| verified | `crates/acorn-host/src/lib.rs` and `Cargo.toml`: HTTP, source, download and OAuth auth are separately feature-gated; `secrecy` is currently optional under auth. | A netrc capability must not require the OAuth stack or enter portable crates. |
| verified | `crates/acorn-lib/src/io/source.rs`, `Source::read_context`: native source reads construct a host HTTP client. | Add an explicit client/context path without changing default readers. |
| verified | `crates/acorn-lib/src/io/config/bucket/mod.rs`, `Bucket::archive` and `Sources`: remote BagIt bytes go through `download_with_progress` before verification. `crates/acorn-cli/src/commands/download/mod.rs::run` builds the transfer options. | Loading a netrc file only in the host client would leave the CLI archive path uncovered. |
| verified | `crates/acorn-lib/src/io/http/mod.rs`: request, streaming, range-probe and anonymous-fallback paths exist. `io/download.rs::request_headers_with_huggingface_fallback` and `io/config/bucket/source.rs::download_item` already resolve provider credentials. | Use explicit per-transfer context; preserve provider precedence and avoid process-global credential state. |
| verified | A targeted search of crates, manifests, lockfile, docs and wiki found no existing netrc integration or plan. | This is new functionality, not a completed capability to document. |
| verified | [curl netrc documentation](https://everything.curl.dev/usingcurl/netrc.html), sections “The .netrc file format” and “Enable netrc,” inspected November 11: machine/login/password entries, quoted values, opt-in flags and Windows filename fallback are documented. | Use the documented syntax as a compatibility reference, with the deliberate restrictions below. |
| verified | [Earthdata cURL/Wget guide](https://urs.earthdata.nasa.gov/documentation/for_users/data_access/curl_and_wget), steps 2–4, inspected November 11: netrc setup is combined with cookies and redirect following. | Research-data interoperability motivates the feature, but parsing alone does not prove Earthdata compatibility. |
| unresolved | No live authenticated data-source run or user credential file has been inspected. | Synthetic acceptance gates this release; do not advertise Earthdata compatibility. |

# Contract decisions

- Activation is per operation. `--netrc-file PATH` selects exactly that file; `--netrc` discovers the home `.netrc`, falling back to `_netrc` on Windows only when `.netrc` is absent. The flags conflict. Neither current-directory discovery nor `NETRC` environment discovery is part of this release. Missing, unreadable or malformed selected files fail clearly before network effects. Offline mode prevents both credential loading and requests.
- Parse whitespace-separated `machine`, `login` and `password` records, comments, quoted values and documented escapes with explicit file/token limits. Match normalized URL hostnames exactly, never by suffix. Reject ambiguous duplicate machine records and incomplete selected credentials. Ignore `default` records for lookup and skip `macdef` bodies without execution; document the restricted dialect. Passwords may be empty only when explicitly present; do not trim secret values.
- Use a separate optional host `netrc` feature and redacted secret types. Reuse a maintained parser if it satisfies the conformance fixtures; otherwise keep the restricted parser local. NTR-01 must record the dependency choice and freeze its exact supported grammar before consumers land. No credential contents may appear in parse errors or Debug output.
- Open once, bound the read, and inspect the opened file's metadata. On Unix reject group/other permission bits and non-regular files; never change permissions automatically. Windows has no Unix mode-bit guarantee: document the user's ACL responsibility and validate native discovery/loading independently. Tests use temporary synthetic files, never the user's home credentials.
- Explicit request authentication, provider token selection and selected OAuth profiles take precedence; an explicit authentication failure must not trigger netrc fallback. Non-authentication headers must not suppress netrc. Credentials from netrc become HTTP Basic only; never infer Bearer or provider-specific token headers from a password.
- Require HTTPS for sending netrc credentials. Scope credentials to an explicitly authorized origin (scheme, host and port), not just the machine lookup key. Initial source origins are authorized by the caller; a broad network allowlist alone does not authorize sending secrets to redirect targets. On an origin change strip authentication; resolve a destination's own entry only if that destination origin was explicitly authorized for credentials. CLI v1 authorizes initial archive origins only. Never send credentials on HTTPS-to-HTTP downgrade.
- Credential resolution applies consistently to GET, HEAD, probe fallback, retries and range/resume. Preserve existing timeout, redirect-count, response-size, cancellation, checksum and atomic destination guarantees. An anonymous fallback must stay anonymous, without middleware silently reattaching credentials. Preserve anonymous APIs through additive methods/context parameters rather than adding secret fields to serialized options.

# Dependencies and ownership

NTR-01 → NTR-02 → NTR-03 → NTR-04 → NTR-05 → NTR-06 → NTR-07. Each phase yields an independently usable library capability, integration or verified handoff. Native Windows acceptance is a prerequisite for that platform's release claim, not for parser work.

The [shared API contract](../refactors/api/shared-client-contract.md), [SharePoint OAuth](sharepoint-jsonl-download/index.md), [CLI composition](../refactors/core/cli-composition-boundary.md), [model acquisition](model-acquisition-hardening.md) and [Cosmos buckets](azure-cosmos-buckets.md) retain their existing outcomes. No entire adjacent plan blocks netrc. Serialize overlapping HTTP, module-root, manifest, bucket and CLI files with their owners. NTR-01 uses the workspace-graph lock for feature/dependency wiring; NTR-03/04 own the HTTP transport lock; NTR-05 owns transfer/archive; NTR-06 owns CLI contract. Final broad checks and central wiki edits have one integration owner.

# Acceptance criteria

- Without opt-in, a populated or malformed home netrc produces no credential-file reads or authenticated requests.
- Explicit path and home discovery, syntax boundaries, duplicates, missing files, Unix permissions, Windows fallback and redaction are covered by synthetic tests.
- Authenticated host source reads and remote BagIt CLI downloads succeed against controlled endpoints; archive integrity and safe materialization still gate publication.
- Tests capture received headers to prove explicit-auth precedence, Basic encoding, exact machine matching, separate ports, unauthorized redirect origins, downgrade refusal, retry consistency and truly anonymous fallback.
- Offline and unsupported-source cases fail before credential/network effects; no credentials escape through verbose diagnostics, errors, provenance or persisted configuration.
- Default/native/portable feature checks pass. Native acceptance is recorded only for platforms actually exercised. Documentation states the supported subset and excludes unproven Earthdata/cookie workflows.

# Execution phases

## NTR-01 — Add the host credential parser and feature

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: workspace-graph and host-root owners have released the exact files.
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-host/Cargo.toml`, `crates/acorn-lib/Cargo.toml`, `crates/acorn-host/src/lib.rs`, `crates/acorn-host/src/netrc.rs`, `crates/acorn-host/src/netrc/tests.rs`.
- Primary integration lock: Workspace graph
- Outcome: An optional pure parser with bounded input, deterministic exact-host lookup, redacted credentials and documented dialect; no filesystem or network effects. Wire native library feature access without changing portable dependencies. Pin a parser dependency only after checking the required fixtures and license.
- Focused validation: `cargo test -p acorn-host --features netrc netrc`; cover quoting, comments, escapes, duplicate machines, incomplete records, ignored default, skipped macros, empty values and size limits; `cargo check -p acorn-host --no-default-features`.
- Handoff: Record the parser/dependency contract and passing fixtures; NTR-02 consumes the stable credential type.

## NTR-02 — Load explicitly selected credential files

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-01.
- Exact write set: `crates/acorn-host/src/netrc.rs`, `crates/acorn-host/src/netrc/tests.rs`, `crates/acorn-host/src/netrc/load.rs`.
- Primary integration lock: Host netrc loader
- Outcome: Explicit path/home selection, bounded open-once reads, Unix permission enforcement, Windows filename fallback and no implicit loading.
- Focused validation: `cargo test -p acorn-host --features netrc netrc`; use injected home paths and temporary files for absent/invalid files, non-regular files, permission rejection, fallback order and zero credential reads when disabled/offline.
- Handoff: Stable operation-owned loader result with redacted diagnostics; NTR-03 can attach it to a host client.

## NTR-03 — Apply origin-scoped credentials in the host HTTP client

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-02; HTTP transport owner available.
- Exact write set: `crates/acorn-host/src/http.rs`, `crates/acorn-host/src/http/netrc.rs`, `crates/acorn-host/src/tests/http.rs`, `crates/acorn-host/src/tests/source.rs`, `crates/acorn-host/src/tests/download.rs`.
- Primary integration lock: HTTP transport
- Outcome: Additive opted-in client construction applies Basic credentials consistently to host reads, downloads and HEAD/GET probes with per-origin redirect authorization; defaults retain existing behavior.
- Focused validation: `cargo test -p acorn-host --all-features`; synthetic transport/header assertions for initial requests, same-origin and cross-origin redirects, different ports, downgrade, probe fallback and redaction. Never disable TLS verification in production to simplify fixtures.
- Handoff: Reusable request credential/origin policy for NTR-04; record which host paths and negative cases are covered.

## NTR-04 — Carry explicit credentials through library HTTP paths

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-03.
- Exact write set: `crates/acorn-lib/src/io/http/mod.rs`, `crates/acorn-lib/src/io/http/tests/mod.rs`, `crates/acorn-lib/src/io/source.rs`.
- Primary integration lock: HTTP transport
- Outcome: Add explicit operation-context variants for source reads and streaming downloads that reuse host credential policy, preserve existing APIs, and handle retries/range probes without replacing provider tokens or reattaching credentials to anonymous fallback.
- Focused validation: `cargo nextest run -p acorn-lib io::http`; focused source tests and feature compilation; assert header precedence and request isolation across concurrent contexts.
- Handoff: Archive acquisition can receive credentials without global state or changed model/provider defaults; NTR-05 consumes this API.

## NTR-05 — Enable authenticated remote archive acquisition

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-04; transfer/archive owner available.
- Exact write set: `crates/acorn-lib/src/io/config/bucket/mod.rs`, `crates/acorn-lib/src/io/config/bucket/netrc_tests.rs`.
- Primary integration lock: Transfer/archive
- Outcome: Add opt-in archive/source batch methods passing an operation-owned context through remote archive classification and verified BagIt acquisition; existing methods remain anonymous and serialized bucket options contain no secrets.
- Focused validation: `cargo nextest run -p acorn-lib netrc`; synthetic authenticated archive fixtures verify inference, forced format, integrity failure, unchanged destination on failure, offline refusal and mixed unsupported-source rejection before effects.
- Handoff: A usable library download path with explicit eligibility rules for NTR-06; no repository enumeration changes.

## NTR-06 — Expose opt-in download flags

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-05; CLI and generated-help owners available.
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/commands/download/tests/mod.rs`, `CommandLineHelp.md`.
- Primary integration lock: CLI contract
- Outcome: Wire conflicting `--netrc`/`--netrc-file PATH` flags into remote BagIt downloads, with early offline/unsupported-source rejection, operation-local loading and safe verbose errors.
- Focused validation: `cargo nextest run -p acorn-cli commands::download`; parser tests for flag conflicts and unsupported subcommands; synthetic CLI success/failure and `-vvvv` redaction checks; `make help`.
- Handoff: User-facing behavior is complete and help matches syntax; NTR-07 validates release claims and documents actual coverage.

## NTR-07 — Validate platforms and publish the supported contract

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: NTR-06; authorized native Linux, macOS and Windows test environments for platform claims, and a stable tree for serialized broad gates. Host availability is external waiting.
- Exact write set: `README.md`, `docs/src/packages/acorn-host.md`, `docs/src/cli-reference/configuration.md`, `.wiki/features/netrc-support.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`.
- Primary integration lock: Documentation generation
- Outcome: Document opt-in examples using fake credentials, file handling, precedence, supported syntax, redirect limits and provider/Earthdata exclusions; record native acceptance and retire the completed plan.
- Focused validation: Run loader/CLI fixtures on each named native OS; serialized `make lint`, `make test`, `make check-wasm`; run the wiki validator and check help/documentation consistency. Unexpected product defects become bounded repair phases with exact files; do not expand this documentation phase.
- Handoff: Preserve passing evidence and limitations in durable documentation; remove this plan and all index/network/lane entries once acceptance passes.
