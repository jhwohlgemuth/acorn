---
type: Plan
title: Install and operate the acornd background service
description: Run ACORN request endpoints and configured local check/gather jobs as a persistent service on local or SSH-managed hosts.
difficulty: high
impact: high
package: [acorn-service, acorn-server, acorn-daemon, acorn-store, acorn-cli]
tags: [feature, daemon, jobs, service, ssh, release, os:linux, os:macos, os:windows]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Ship `acornd` as a long-running ACORN host application. It serves the existing authenticated JSON-RPC and typed gRPC operations while it watches configured local project paths and runs `acorn check` or `acorn gather` on file changes and fixed intervals. A matching `acorn` CLI installs and manages it locally or over SSH on Linux, macOS, and Windows. The process runs as the chosen unprivileged user and survives logout and reboot.

Deliver a native Linux service MVP before starting Windows service implementation or publication. The MVP uses same-version source-built `acorn`/`acornd` binaries, a protected local config and token, and an owner-scoped systemd unit to prove local control, jobs, and persistence on a named Linux host. Signed bundles, SSH installation, and multi-OS release acceptance follow in separate phases.

# Current evidence

- `acorn-service` owns transport-neutral operations and portable daemon job values. The distinct `Gather` action now has its own serialized spelling; supplied-content repository processing rejects it because that path has no gather capability. `acorn-store` implements the existing `DurableOperations` status/replay port using run IDs.
- `acorn-server` has authenticated `/rpc`, health/readiness endpoints, typed gRPC, loopback defaults, non-loopback TLS validation, and graceful listener shutdown. `ServerConfig::resolve_with_token` accepts a caller-supplied secret and redacts it from Debug output. `serve_with_registry` accepts a caller-owned operation registry and shutdown future while the standalone `serve`/`serve_with_durable` entry points retain their signal handling and mutation-adapter guard. The 14 `acorn-server` tests pass, including injected operation, external shutdown, listener failure, and in-flight drain cases; package clippy passes with warnings denied. The standalone binary is not a background-service installer.
- `acorn check` and `acorn gather` already support watch mode and have mature host behavior. Their command modules remain inside the CLI, so a typed, fixed subprocess invocation preserves current behavior without first moving broad host workflows across crate boundaries. `check` can emit JSON to stdout; `gather` accepts `--output`.
- `standard_project_folder(namespace, None)` creates a fresh path under the per-user ACORN project cache. It does not mean the watched repository. The daemon must index those paths in durable run history; `gather` needs an explicit output path.
- The current release targets package the CLI and SHA-256 files, not a paired signed `acorn`/`acornd` bundle. Local Linux user-service registration exists; signed SSH installation remains planned. `acorn-store` now owns only daemon SQLite history schema version 1; existing `acorn-lib` tables are untouched.
- `acorn-service::job` exports validated job IDs, kinds, trigger schedules and causes, durable run states, and pure run coalescing. Module tests pin the serialized spellings, validation boundaries, terminal states, and complete transition matrix. All 27 `acorn-service` tests and the alloc-only `wasm32-unknown-unknown` check pass without adding host I/O or dependencies.
- November 24 source and focused execution at `4fc72c19` plus this worktree confirm that `acornd run [CONFIG]` opens per-user daemon history, starts one worker, watches configured inputs, and schedules startup, debounced file-change, and interval runs. Fixed check/gather subprocesses use the paired binary, a project-root working directory, process-group timeout/cancellation, bounded captured streams, and indexed reports. Pause state, claims, recovery, replay, and 100-run/30-day per-job retention are covered by store tests. Focused `acorn-store` (7) and `acorn-service` (27) tests, package formatting, daemon/store rustdoc, and the alloc-only service WASM check passed in an isolated target directory.
- ACD-08 composes the accepted server with the scheduler only after watches and startup queues initialize. It loads the per-user daemon token from `<data>/daemon/token`, requires an owner-only regular file on Unix, and injects it without environment mutation. Authenticated JSON-RPC reads are `acorn.jobs.list` → `{jobs:[{id,kind,paused}]}`, `acorn.jobs.status` → `{job,latest}`, and `acorn.jobs.history` → `{runs}` with an optional 1–100 `limit`; mutations are `acorn.jobs.run` → `{accepted}` and `acorn.jobs.pause|resume` → `{paused}`. Run records expose `id`, `jobId`, `trigger`, `state`, timestamps, `exitCode`, `reportPath`, and bounded `detail`. The daemon allows mutation on its loopback JSON-RPC endpoint; the typed gRPC service keeps its existing methods. The worker and server drain together on shutdown or failure. Focused `acorn-daemon` (31) and `acorn-server` (14) tests, targeted Clippy with warnings denied, and package formatting passed.
- The reload prerequisite repair added `DaemonStore::activate_jobs`: one SQLite transaction replaces active job IDs, interrupts superseded queued records, and inserts startup runs only for enabled jobs. Running and completed records keep their IDs, and pause state survives; rollback on duplicate IDs is tested. All 11 focused `acorn-store` tests pass.
- ACD-08A applies a complete replacement ACRE job snapshot through authenticated mutation-gated `acorn.jobs.reload` and returns its generation. Invalid or partial documents, unsupported actions, watcher preparation failure, and restart-only listener/TLS changes leave the active generation intact. Unchanged jobs retain scheduling state and pause/history; changed in-flight jobs finish on their old snapshot and receive one new run; removed in-flight jobs finish without requeue. Watcher epochs discard stale callbacks, and the scheduler serializes control events while keeping one worker. A live HTTP test confirms process identity and non-overlapping old/new runs. Serial focused tests passed (43 daemon, 11 store), as did package Clippy with warnings denied and formatting.
- The distinct Gather action exposed one missing `acorn-server` gRPC client conversion arm. It now maps to the existing unspecified repository action, which the typed server rejects; all 14 focused `acorn-server` tests pass. No protobuf action was added because supplied-content repository processing does not support gather.
- ACD-09 added `acorn daemon` lifecycle dispatch and authenticated JSON-RPC job commands. The CLI reads a private regular token from the daemon data directory, accepts an explicit config path for local job control, and never passes the token to `acornd` or logs it. Three focused parser/token/error tests, package Clippy with warnings denied, and formatting passed on November 24.
- ACD-09R made daemon result output visible at default verbosity while preserving explicit quiet/log overrides. On Ubuntu 22.04 x86_64, ACD-10's owner-scoped systemd 249 unit passed `systemd-analyze --user verify`, was active/enabled with mode 0600, and survived an idempotent uninstall/reinstall. Two Linux installer tests and native CLI/daemon Clippy with warnings denied passed. ACD-25R excluded access events from file watchers: 14 scheduler tests passed on macOS and Linux, and a stable native check input kept run ID 101 until an edit produced successful file-change run ID 103. ACD-25V temporarily removed one invalid cached proselint rule after Vale sync; restored bad input was removed and native check startup/manual runs 99/101 succeeded. That source workaround was removed during Hunk review, so those check results do not validate the current managed Vale path.
- ACD-25J corrected the fixed check runner's JSON report by passing the CLI's global quiet flag. Nine focused runner tests passed on macOS and Linux; native startup run 125 succeeded and its stored report parsed as JSON with `jq`. This addressed Vale sync progress bytes previously prefixed to successful check reports.
- ACD-10R escaped dollar signs in generated systemd `ExecStart` paths and rejects empty or malformed private tokens before registration. Three Linux installer tests, native daemon Clippy with warnings denied, and a native build passed. The existing unit passed `systemd-analyze --user verify`.
- ACD-25 accepted the source-built local Linux MVP on `oklahoma` (Ubuntu 22.04.5 x86_64, systemd 249). The unit started seven seconds after reboot and before the first observed SSH login, retained paused gather state and prior check/gather history, and produced valid new check and gather reports. Authentication, file/interval/manual jobs, restart, reload, stop, and idempotent reinstall passed. Final uninstall removed the unit while preserving protected config, token, and history. A direct last-logout transition remained unavailable because other sessions were active; [dated native evidence](../../crates/acorn-daemon/acceptance/linux-mvp.md) records that row as incomplete.
- ACD-09S fixed the local CLI token disclosure risk from a forged loopback listener. The CLI verifies an owner-only daemon directory with non-writable, trusted-owner ancestors, then sends the bearer only through its Unix socket; the daemon binds that socket with mode 0600 and safely recovers stale paths. TCP JSON-RPC remains available for other bearer-authenticated clients. On `oklahoma`, the rebuilt CLI listed jobs through a mode-0600 socket in a mode-0700 directory, and uninstall removed it. The occupied-port test received no TCP connection. Local tests passed for 17 server, 55 daemon, and four focused CLI cases; native Linux tests and Clippy also passed. Windows local job control fails closed until its platform adapter supplies an owner-verified transport.
- November 25 Hunk review removed the temporary ACD-25V managed Vale rule cleanup and the associated default disabled-rule entry. A rebuilt native check initially exited 70 under Vale 3.9.4 because sync restored `LexicalIllusions.yml`. ACORN now pins Vale 3.28.1 in a versioned managed cache and parses checksums independently of version-string length. On `oklahoma`, the rebuilt CLI downloaded Vale 3.28.1 and `acorn --quiet check check.json --output json` exited 0 with no checks. The native service check-job itself was not rerun after this upgrade.

# Boundaries and dependencies

- `acorn-service` owns portable job identifiers, trigger/state values, and coalescing policy; host paths remain in daemon config. `acorn-store` initially owns only the SQLite daemon-history adapter; migration of the existing `acorn-lib` database remains separate. `acorn-daemon` owns filesystem watching, clock, process execution, host control, installation, and service-manager integration. It calls the existing `acorn-lib` cache-path helper rather than copying its path convention. `acorn-server` remains the protocol boundary.
- The daemon ships the existing portable request methods unchanged. Host job-control methods are registered by the daemon on JSON-RPC only; the typed gRPC service stays limited to its present supplied-content operations. Neither path exposes arbitrary process execution or config mutation.
- The portable service contract, composable server lifecycle, versioned history store, fixed runners, foreground scheduler, authenticated listener composition, job-control methods, atomic job reload, and local CLI management are available. ACD-09 released the CLI syntax lock for [CLI composition](../refactors/core/cli-composition-boundary.md) CLI-01. ACD-18/19 later claim their CLI command leaves under the same contract lock. Release wiring serializes with [Podman](../migrations/podman-makefile.md), runner-pool, and other CI/build owners. Windows live validation needs a native host; the [shell-runner plan](../testing/gitlab-shell-runner-readiness.md) may supply one but is not a code prerequisite.
- ACD-10 Linux systemd registration and ACD-25 native Linux MVP acceptance are complete. ACD-12 Windows SCM work and its dependent registration, bundle, and live-host phases may now proceed on their own locks. ACD-20 remains the later signed SSH installation and upgrade acceptance on Linux; the MVP does not imply that release gate passed.
- Service registration requires explicit elevation when the host requires it. Linux user lingering, macOS system LaunchDaemon registration, and Windows Service registration are platform-specific gates; the daemon process still runs as the chosen user. Do not claim persistent operation on a host that cannot pass the registration and reboot/logout smoke test. See [systemd lingering](https://www.freedesktop.org/software/systemd/man/257/loginctl.html), [Apple launch daemons](https://developer.apple.com/documentation/servicemanagement/updating-helper-executables-from-earlier-versions-of-macos), and [Windows service creation](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/sc-create).

# Decisions and public contract

- Add a `crates/acorn-daemon` package with an `acornd` binary. Keep the current `acorn-server` binary and APIs available. Add narrow `acorn-server` entry points for an explicitly supplied bearer secret, injected operation registry, and external shutdown signal; preserve the existing `serve`/`serve_with_durable` behavior. Load the daemon token from its protected file before threads start, without mutating process-global environment.
- Reuse one ACRE application document and its `daemon` section; do not introduce a second daemon-only config schema. Preserve the current `acornd run [CONFIG]` foreground syntax unless a separately scoped CLI change adds an alias. Validate the complete replacement daemon configuration before listener/worker activation. Initial runners support only check/gather: explicitly reject other shared `WorkflowAction` values at runtime admission until their runner is implemented. Resolve relative inputs against each canonical job root, never the service working directory.
- A daemon job invokes its co-installed, same-version `acorn` binary with a constructed argument vector, fixed working directory, and no shell. It supplies `check <path> --output json` (the current positional CLI form) or `gather <path> --output <run-path> --format json`; it never passes `--watch`. No free-form command or flag list is accepted in config. Preserve normal CLI validation findings and command errors in the stored report and exit record.
- Schedule one enabled run per job at startup, then at each fixed interval and after debounced file changes. Use one global worker so jobs do not overlap; while a job runs, collapse further triggers for that job into one pending rerun. On restart mark in-flight runs interrupted and schedule one fresh startup run, rather than replaying every missed interval. A failed run waits for the next trigger or manual run. Pause state survives restart.
- Store job pause state and run records in SQLite under the daemon's per-user data directory. Record job ID, trigger, timestamps, state (`queued`, `running`, `succeeded`, `failed`, `interrupted`), exit status, bounded diagnostic, and report path. Use `standard_project_folder("check" | "gather", None)` for each run's artifacts and index the path; only daemon-owned run paths are eligible for retention cleanup. Bound subprocess duration and captured stdout/stderr, and retain at most 100 runs or 30 days per job. Replay and manual run create a new run ID; prior history stays immutable.
- Register `acorn.jobs.list`, `acorn.jobs.status`, and `acorn.jobs.history` as authenticated reads; register `acorn.jobs.run`, `acorn.jobs.pause`, `acorn.jobs.resume`, and `acorn.jobs.reload` as mutation-gated operations. Inputs use job IDs and an optional bounded history limit; results return stable states and report paths, not report bodies or secrets. Reload validates the whole replacement config before swapping jobs; removed jobs stop scheduling while their history remains queryable. The daemon enables the mutation policy for its loopback control endpoint and supplies a durable adapter that maps existing `acorn.operations.status`/`replay` keys to run IDs; it never weakens the standalone server's adapter requirement. The local CLI uses an owner-protected Unix socket and private token file; `--host ssh://…` executes the corresponding fixed management command on the host so the token never leaves it. Windows needs an owner-verified local transport before its job commands are enabled.
- The portable service contract supplies validated `JobId`, `JobSchedule`, `JobTrigger`, `RunState` and coalescing policy. The current ACRE job kind uses `WorkflowAction`, not a separate `JobKind`. Job IDs are 1–64 lowercase ASCII letters/digits/interior hyphens; schedules serialize `onChange` and optional positive `everySeconds`, requiring a trigger. Host paths and process settings stay in daemon config. Initial check/gather runtime admission must be tested independently of the broader shared action enum.
- Model coalescing as one pure state transition over `idle`, `queued`, `running`, and `running-pending`. A trigger changes idle to queued, leaves queued unchanged, changes running to running-pending, and leaves running-pending unchanged. Starting is valid only from queued and produces running. Finishing running produces idle; finishing running-pending produces queued. Invalid start/finish transitions return a typed error. Pause is a persisted scheduler gate, not another coalescing state; while paused the scheduler does not submit trigger events. Unit tests pin identifier bounds, deserialization rejection, exact serialized spellings, schedule validation, terminal-state classification, every transition, and invalid transitions.
- The daemon's `acornd run [CONFIG]` uses the platform `.acorn.jsonc` when no path is supplied. It loads and validates the ACRE daemon section, opens the local store, reads the protected token, starts the scheduler and worker, and serves the configured listeners until Ctrl-C or failure. Linux registration commands dispatch from `acornd`; native service behavior passed the local Linux MVP. Installed-target resolution does not search PATH.
- Preserve the current ACRE and host dependencies in `crates/acorn-daemon/Cargo.toml`; do not remove the existing library/schema dependencies to recover an obsolete standalone-YAML design. Add only dependencies consumed by the next bounded runtime phase under the workspace-graph lock.
- ACRE owns `DaemonConfiguration` with `bind`, `jobs` and optional paired `tls` paths. RPC defaults to `127.0.0.1:8001`, gRPC to `127.0.0.1:50051`; explicit null disables a listener and both disabled is rejected by daemon validation. Compose with the existing `ServerConfig` validation before binding non-loopback endpoints. Listener/TLS changes are restart-only for the initial job reload contract.
- Each shared ACRE daemon job has `id`, `root`, `input`, `kind` and `triggers`, with unknown fields rejected. Host validation requires an existing absolute root and an existing portable relative input beneath its canonical root, rejecting traversal, remote/stdio input and symlink escape. Shared schedules use `onChange`/`everySeconds`; preserve shared schema validation and duplicate-ID checks.
- The local source-built CLI provides `acorn daemon install|uninstall|start|stop|restart|service-status|list|status|history|run|pause|resume|reload`; `--config` is required for install and optional for job commands. It invokes fixed sibling `acornd` lifecycle commands and does not depend on the daemon crate as a Rust library. ACD-18/19 add signed bundle staging, remote `--host` management, versioned upgrades, and rollback. Uninstall stops registration and preserves config, token, history, and reports unless a separate explicit purge operation is later designed.
- Default to loopback listeners and SSH tunneling for remote API use. Reuse the server's certificate/key check for explicit non-loopback listeners. Generate a random bearer token at install, write it with owner-only permissions or ACL, and never print or put it in process arguments, unit files, or logs. A third-party client using a tunnel needs a separately provisioned bearer credential; `acorn daemon --host` management reads it only on the host.
- Release paired `acorn`/`acornd` bundles for Linux x86_64/ARM64, macOS x86_64/ARM64, and Windows x86_64. Sign the complete versioned bundle with a maintainer-controlled Minisign key and pin the public key in the installer. A release-built CLI uses its embedded versioned ACORN package-registry base URL; a source build without that metadata requires `--release-url URL` or `--bundle PATH`. The SSH installer detects target OS/architecture, downloads and verifies the exact bundle locally, uploads it, verifies bytes on the host, and activates it. `--bundle` supplies an already downloaded signed bundle for offline targets. Provisioning the private release key is an external prerequisite, never a repo file. [Minisign signing and verification](https://github.com/jedisct1/minisign).
- Coordinate incremental compilation, rebuild/restart and optional function hot reload through the [development-loop contract](../refactors/blaze-development-pilot/development-loop.md). The prior Hotcode proposal becomes BLZ-13: compare Cargo + Hotcode with Blaze-native live-code support in an isolated pure synchronous harness. No production Hotcode dependency, crate-type change or daemon-state patching is planned. Production config reload is owned here by ACD-08A; development build supervision is owned by BLZ-11/12. Daemon shipping does not depend on experimental compiler or hot-patch acceptance.

# Validation and acceptance criteria

- Config reload passes the atomic-generation matrix in ACD-08A without recompiling or restarting. Rebuild/restart development acceptance is coordinated through BLZ-12 and is not a production release gate. BLZ-13 function-patching results cannot substitute for scheduler, history or service-manager acceptance.

- Config tests cover valid defaults, duplicate jobs, path escape, no trigger, malformed intervals, and no listener; token and error diagnostics are redacted. Service tests prove the existing JSON-RPC/gRPC methods and health behavior still work while the worker is active.
- Deterministic clock/watcher and fake executable tests prove startup runs, interval and file-event coalescing, one-at-a-time execution, pause/resume/reload, timeout/cancellation, crash recovery, report paths, expected check findings, failed commands, and retention. No test executes arbitrary configured commands.
- API/CLI tests cover authentication, mutation denial, bounded history, unknown jobs, SSH argument safety, no token transfer, installer preflight, idempotent reinstall, failed upgrade rollback, wrong architecture, and corrupted or unsigned bundles.
- Native host acceptance proves installation, request handling, scheduled and watched jobs, logout/reboot persistence, service-manager restart, status/history, and uninstall on each supported OS/architecture family. Record the exact tested host and version. Windows service-account credentials or service-logon rights and the release signing key must be available before their dependent live/release phases.
- Final integration runs focused crate tests, relevant feature/portable checks, `make lint`, `make test`, generated CLI help, and documentation build. Do not mark the plan complete from simulated platform tests alone.

# Execution phases

## ACD-11 — Register persistent macOS service

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: accepted local CLI management and foreground daemon composition
- Exact write set: `crates/acorn-daemon/src/install/macos.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Register a system LaunchDaemon that runs as the chosen user, with correct ownership, boot startup, status, and uninstall behavior.
- Focused validation: Plist/preflight tests and launchd syntax check; live logout/reboot acceptance belongs to ACD-21.
- Handoff: Remove ACD-11 after recording plist syntax and preflight behavior; ACD-21 owns live host evidence.

## ACD-12 — Run acornd under Windows Service Control Manager

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: accepted foreground daemon composition and native Linux service MVP
- Exact write set: `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/main.rs`, `crates/acorn-daemon/src/windows_service.rs`, `Cargo.lock`
- Primary integration lock: Workspace graph
- Outcome: Use a safe Windows Service wrapper to accept SCM start/stop events and drain worker/listeners within the service timeout while retaining foreground development mode; preserve the workspace ban on unsafe code.
- Focused validation: Cross-target compile and fake SCM start/stop/forced-termination tests; native service acceptance belongs to ACD-22.
- Handoff: Remove ACD-12 after recording the SCM event contract; release the workspace graph lock.

## ACD-13 — Register persistent Windows service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted local CLI management and ACD-12
- Exact write set: `crates/acorn-daemon/src/install/windows.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Install/start/stop/uninstall a Windows Service under the selected user. Registration obtains credentials through a protected target-side interactive prompt or a preprovisioned managed identity; no credential enters arguments, config, or logs.
- Focused validation: Registration/ACL tests with a fake SCM adapter; live logout/reboot acceptance belongs to ACD-22.
- Handoff: Remove ACD-13 after recording rights preflight and fake-SCM evidence; ACD-22 owns live host evidence and no credentials are retained.

## ACD-26 — Add owner-verified Windows local control

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-12 and ACD-13; Windows service identity and ACL contract available
- Exact write set: `crates/acorn-cli/Cargo.toml`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/windows.rs`, `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/lib.rs`, `crates/acorn-daemon/src/local/windows.rs`, `crates/acorn-server/Cargo.toml`, `crates/acorn-server/src/client/mod.rs`, `crates/acorn-server/src/runtime.rs`, `Cargo.lock`
- Primary integration lock: Local daemon control transport
- Outcome: Give the Windows CLI an owner-verified local control channel to the selected service identity, with no bearer token sent to an unverified loopback listener.
- Focused validation: Fake peer and ACL denial tests, stopped/restarting service behavior, local job-control round trips, and a Windows target compile; ACD-22 owns native host acceptance.
- Handoff: Remove ACD-26 after the Windows control channel and fail-closed behavior pass; release the local transport lock.

## ACD-14 — Define signed bundle verification

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted local CLI management; maintainer provisions the release signing key and corresponding public key
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-cli/Cargo.toml`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/bundle.rs`, `crates/acorn-cli/assets/acornd-release.pub`
- Primary integration lock: Workspace graph
- Outcome: Pin the release public key, define version/target/binary metadata, and verify a complete Minisign-signed bundle before extraction or execution.
- Focused validation: Valid signature, wrong key, altered manifest/binary, missing member, version mismatch, and archive path traversal fixtures.
- Handoff: Remove ACD-14 after recording the public-key fingerprint and bundle format; release the workspace graph lock.

## ACD-15 — Produce Linux release bundles

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted Linux registration and ACD-14; Linux x86_64 and ARM64 builders available
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `scripts/upload-binaries.sh`, `scripts/package-acornd.sh`
- Primary integration lock: CI/build
- Outcome: Build paired Linux x86_64/ARM64 binaries, sign and upload each bundle, and publish only after local verification.
- Focused validation: Native Linux build jobs, signature round trips, target metadata, and downloaded artifact inspection.
- Handoff: Remove ACD-15 after recording artifact URLs and builder evidence; release the CI/build lock.

## ACD-16 — Produce macOS release bundles

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-11, ACD-14; native macOS x86_64 and ARM64 builders available
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `scripts/upload-binaries.sh`, `scripts/package-acornd.sh`
- Primary integration lock: CI/build
- Outcome: Build, sign, and publish matching macOS x86_64/ARM64 bundles under the same verified format.
- Focused validation: Native macOS build jobs, signature round trips, target metadata, and downloaded artifact inspection.
- Handoff: Remove ACD-16 after recording artifacts and builder evidence; release the CI/build lock.

## ACD-17 — Produce Windows release bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-13, ACD-14; native Windows x86_64 builder available
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `scripts/upload-binaries.sh`, `scripts/package-acornd.ps1`
- Primary integration lock: CI/build
- Outcome: Build, sign, and publish a paired Windows x86_64 bundle, including the SCM-capable daemon.
- Focused validation: Native Windows build/runtime checks, signature round trip, target metadata, and downloaded artifact inspection.
- Handoff: Remove ACD-17 after recording artifact and builder evidence; release the CI/build lock.

## ACD-18 — Stage and activate a local signed bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted Linux registration, ACD-14, and ACD-15; other platform adapters are exercised after their respective registration and bundle phases
- Exact write set: `crates/acorn-cli/src/commands/daemon/bundle.rs`, `crates/acorn-cli/src/commands/daemon/install.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Stage a verified paired bundle and config, generate a protected token, invoke platform registration, verify readiness, and roll back a failed upgrade while preserving user data.
- Focused validation: Local install/reinstall/upgrade/uninstall fixtures for all platform adapters, plus one native host activation; corrupt signature, unsupported target, and failed-readiness rollback.
- Handoff: Remove ACD-18 and freeze the installer transaction contract.

## ACD-19 — Implement SSH transfer and remote control

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted local CLI management, ACD-14, and ACD-18
- Exact write set: `crates/acorn-cli/src/commands/daemon/remote.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/tests.rs`
- Primary integration lock: CLI contract
- Outcome: Detect remote target, verify and upload a signed bundle, invoke fixed host commands, check remote bytes/readiness, and support `--bundle` for offline hosts without transferring the token.
- Focused validation: Fake SSH tests for quoting, target mismatch, interrupted transfer, elevation denial, secret redaction, idempotence, and rollback.
- Handoff: Remove ACD-19 after freezing remote command and transfer behavior; release the CLI contract lock.

## ACD-20 — Accept SSH installation on Linux

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted Linux registration, ACD-15, and ACD-19; named Linux SSH host and approved lingering/elevation
- Exact write set: named Linux SSH test host and its user service manager
- Primary integration lock: External Linux service host
- Outcome: Prove signed remote install, update, control, logout/reboot persistence, rollback, and uninstall on Linux.
- Focused validation: Capture service status, health, job report/history, before/after version, and post-reboot results on the host.
- Handoff: Remove ACD-20 after preserving host/version evidence in durable deployment documentation.

## ACD-21 — Accept SSH installation on macOS

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-11, ACD-16, ACD-19; named macOS SSH host and administrator approval
- Exact write set: named macOS SSH test host and its system launchd domain
- Primary integration lock: External macOS service host
- Outcome: Prove signed remote install, update, control, logout/reboot persistence, rollback, and uninstall on macOS.
- Focused validation: Capture launchd status, health, job report/history, before/after version, and post-reboot results on the host.
- Handoff: Remove ACD-21 after preserving host/version evidence in durable deployment documentation.

## ACD-22 — Accept SSH installation on Windows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-13, ACD-17, ACD-19, and ACD-26; named Windows SSH host, administrator approval, and provisioned service account
- Exact write set: named Windows SSH test host and its Service Control Manager
- Primary integration lock: External Windows service host
- Outcome: Prove signed remote install, update, control, logout/reboot persistence, rollback, and uninstall on Windows.
- Focused validation: Capture SCM status, health, job report/history, before/after version, and post-reboot results on the host without exposing credentials.
- Handoff: Remove ACD-22 after preserving host/version evidence in durable deployment documentation.

## ACD-23 — Publish daemon documentation and generated help

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-19; platform adapters and release formats stable
- Exact write set: `README.md`, `ARCHITECTURE.md`, `docs/src/packages/acornd.md`, `docs/src/SUMMARY.md`, `CommandLineHelp.md`
- Primary integration lock: Documentation generation
- Outcome: Document config, local/SSH installation, service identities, SSH tunnels, cache/report paths, security, retention, upgrade, and recovery; regenerate CLI help.
- Focused validation: `make help`, `make book`, link checks, and example config parsing on each supported OS.
- Handoff: Remove ACD-23 after durable docs replace plan-only operating details.

## ACD-24 — Close workspace and deployment acceptance

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: remaining ACD-11 through ACD-23 phases and ACD-26; five bundles and all three OS acceptance records available
- Exact write set: repository CI and named Linux, macOS, and Windows acceptance records
- Primary integration lock: CI/build
- Outcome: Run final workspace gates, review native host evidence against every acceptance criterion, and retire this plan only when no required work remains.
- Focused validation: `make lint`, `make test`, `make check-portable`, bundle signature verification, and clean documentation build.
- Handoff: Move any remaining durable facts to code/docs, delete this concept, and remove all wiki index, network, and lane references.
