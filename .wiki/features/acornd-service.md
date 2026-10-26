---
type: Plan
title: Install and operate the acornd background service
description: Run ACORN request endpoints and configured local check/gather jobs as a persistent service on local or SSH-managed hosts.
difficulty: high
impact: high
package: [acorn-service, acorn-server, acorn-daemon, acorn-store, acorn-cli]
tags: [feature, daemon, jobs, service, ssh, release]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Ship `acornd` as a long-running ACORN host application. It serves the existing authenticated JSON-RPC and typed gRPC operations while it watches configured local project paths and runs `acorn check` or `acorn gather` on file changes and fixed intervals. A matching `acorn` CLI installs and manages it locally or over SSH on Linux, macOS, and Windows. The process runs as the chosen unprivileged user and survives logout and reboot.

# Current evidence

- `acorn-service` already owns transport-neutral validation, readability, repository processing, operation effects, and a JSON-RPC registry. Its `DurableOperations` port covers operation status/replay, but there is no job-definition, scheduler, or job-history implementation.
- `acorn-server` already has authenticated `/rpc`, health/readiness endpoints, typed gRPC, loopback defaults, non-loopback TLS validation, and graceful listener shutdown. `serve_with_durable` currently constructs its own registry and owns the process signal; daemon composition needs an injected registry and coordinated shutdown. The standalone `acorn-server` binary is not a background-service installer.
- `acorn check` and `acorn gather` already support watch mode and have mature host behavior. Their command modules remain inside the CLI, so a typed, fixed subprocess invocation preserves current behavior without first moving broad host workflows across crate boundaries. `check` can emit JSON to stdout; `gather` accepts `--output`.
- `standard_project_folder(namespace, None)` creates a fresh path under the per-user ACORN project cache. It does not mean the watched repository. The daemon must index those paths in durable run history; `gather` needs an explicit output path.
- The current release targets package the CLI and SHA-256 files, not a paired signed `acorn`/`acornd` bundle. There is no daemon service registration or SSH installer in the tree. `acorn-store` is a planned package in the [crate boundary record](../refactors/crate-boundary-architecture.md), with no implementation yet.

# Boundaries and dependencies

- `acorn-service` owns portable job identifiers, trigger/state values, and coalescing policy; host paths remain in daemon config. `acorn-store` initially owns only the SQLite daemon-history adapter; migration of the existing `acorn-lib` database remains separate. `acorn-daemon` owns filesystem watching, clock, process execution, host control, installation, and service-manager integration. It calls the existing `acorn-lib` cache-path helper rather than copying its path convention. `acorn-server` remains the protocol boundary.
- The daemon ships the existing portable request methods unchanged. Host job-control methods are registered by the daemon on JSON-RPC only; the typed gRPC service stays limited to its present supplied-content operations. Neither path exposes arbitrary process execution or config mutation.
- ACD-01 can start from current code; ACD-02 through ACD-06 follow their stated contract/store prerequisites. ACD-09 and ACD-19 claim the CLI contract only after the active [CLI composition plan](../refactors/core/cli-composition-boundary.md) releases the parser/dispatch roots. Release wiring serializes with [Podman](../migrations/podman-makefile.md), runner-pool, and other CI/build owners. Windows live validation needs a native host; the [shell-runner plan](../testing/gitlab-shell-runner-readiness.md) may supply one but is not a code prerequisite.
- Service registration requires explicit elevation when the host requires it. Linux user lingering, macOS system LaunchDaemon registration, and Windows Service registration are platform-specific gates; the daemon process still runs as the chosen user. Do not claim persistent operation on a host that cannot pass the registration and reboot/logout smoke test. See [systemd lingering](https://www.freedesktop.org/software/systemd/man/257/loginctl.html), [Apple launch daemons](https://developer.apple.com/documentation/servicemanagement/updating-helper-executables-from-earlier-versions-of-macos), and [Windows service creation](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/sc-create).

# Decisions and public contract

- Add a `crates/acorn-daemon` package with an `acornd` binary. Keep the current `acorn-server` binary and APIs available. Add narrow `acorn-server` entry points for an explicitly supplied bearer secret, injected operation registry, and external shutdown signal; preserve the existing `serve`/`serve_with_durable` behavior. Load the daemon token from its protected file before threads start, without mutating process-global environment.
- Use one versioned YAML daemon config at a platform user-configuration path, or `acornd run --config PATH`. Version 1 has `server` listener settings and an array of jobs. Each job has a unique ID, absolute project root, one local relative input path, `kind: check | gather`, and `triggers.on_change` and/or positive `triggers.every`. Reject duplicate IDs, missing roots, path escape, no trigger, unsupported fields, and remote/stdio inputs before starting listeners. Resolve relative paths against the job root, never the service process directory.
- A daemon job invokes its co-installed, same-version `acorn` binary with a constructed argument vector, fixed working directory, and no shell. It supplies `check --path ... --output json` or `gather ... --output <run-path>`; it never passes `--watch`. No free-form command or flag list is accepted in config. Preserve normal CLI validation findings and command errors in the stored report and exit record.
- Schedule one enabled run per job at startup, then at each fixed interval and after debounced file changes. Use one global worker so jobs do not overlap; while a job runs, collapse further triggers for that job into one pending rerun. On restart mark in-flight runs interrupted and schedule one fresh startup run, rather than replaying every missed interval. A failed run waits for the next trigger or manual run. Pause state survives restart.
- Store job pause state and run records in SQLite under the daemon's per-user data directory. Record job ID, trigger, timestamps, state (`queued`, `running`, `succeeded`, `failed`, `interrupted`), exit status, bounded diagnostic, and report path. Use `standard_project_folder("check" | "gather", None)` for each run's artifacts and index the path; only daemon-owned run paths are eligible for retention cleanup. Bound subprocess duration and captured stdout/stderr, and retain at most 100 runs or 30 days per job. Replay and manual run create a new run ID; prior history stays immutable.
- Register `acorn.jobs.list`, `acorn.jobs.status`, and `acorn.jobs.history` as authenticated reads; register `acorn.jobs.run`, `acorn.jobs.pause`, `acorn.jobs.resume`, and `acorn.jobs.reload` as mutation-gated operations. Inputs use job IDs and an optional bounded history limit; results return stable states and report paths, not report bodies or secrets. Reload validates the whole replacement config before swapping jobs; removed jobs stop scheduling while their history remains queryable. The daemon enables the mutation policy for its loopback control endpoint and supplies a durable adapter that maps existing `acorn.operations.status`/`replay` keys to run IDs; it never weakens the standalone server's adapter requirement. The local CLI uses loopback JSON-RPC and a protected token file; `--host ssh://…` executes the corresponding fixed management command on the host so the token never leaves it.
- Add `acorn daemon install|uninstall|start|stop|restart|status|history|run|pause|resume|reload`, with `--host` on each management command and `--config` required for install. The CLI verifies and stages signed bundles, then invokes fixed `acornd` registration commands; it does not depend on the daemon crate as a Rust library. `install` is idempotent for the same version/config; upgrades stage new binaries and restore the prior version if readiness fails. Uninstall stops registration and preserves config, token, history, and reports unless a separate explicit purge operation is later designed.
- Default to loopback listeners and SSH tunneling for remote API use. Reuse the server's certificate/key check for explicit non-loopback listeners. Generate a random bearer token at install, write it with owner-only permissions or ACL, and never print or put it in process arguments, unit files, or logs. A third-party client using a tunnel needs a separately provisioned bearer credential; `acorn daemon --host` management reads it only on the host.
- Release paired `acorn`/`acornd` bundles for Linux x86_64/ARM64, macOS x86_64/ARM64, and Windows x86_64. Sign the complete versioned bundle with a maintainer-controlled Minisign key and pin the public key in the installer. A release-built CLI uses its embedded versioned ACORN package-registry base URL; a source build without that metadata requires `--release-url URL` or `--bundle PATH`. The SSH installer detects target OS/architecture, downloads and verifies the exact bundle locally, uploads it, verifies bytes on the host, and activates it. `--bundle` supplies an already downloaded signed bundle for offline targets. Provisioning the private release key is an external prerequisite, never a repo file. [Minisign signing and verification](https://github.com/jedisct1/minisign).
- Treat [Hotcode](https://docs.rs/hotcode/latest/hotcode/) as an optional development experiment on one pure, synchronous function. Its documented ABI and static-reference hazards make hot reload unsuitable for daemon state, scheduler, transport, and production release paths. Normal rebuild/restart remains the supported development loop; no production Hotcode dependency is added.

# Validation and acceptance criteria

- Config tests cover valid defaults, duplicate jobs, path escape, no trigger, malformed intervals, and no listener; token and error diagnostics are redacted. Service tests prove the existing JSON-RPC/gRPC methods and health behavior still work while the worker is active.
- Deterministic clock/watcher and fake executable tests prove startup runs, interval and file-event coalescing, one-at-a-time execution, pause/resume/reload, timeout/cancellation, crash recovery, report paths, expected check findings, failed commands, and retention. No test executes arbitrary configured commands.
- API/CLI tests cover authentication, mutation denial, bounded history, unknown jobs, SSH argument safety, no token transfer, installer preflight, idempotent reinstall, failed upgrade rollback, wrong architecture, and corrupted or unsigned bundles.
- Native host acceptance proves installation, request handling, scheduled and watched jobs, logout/reboot persistence, service-manager restart, status/history, and uninstall on each supported OS/architecture family. Record the exact tested host and version. Windows service-account credentials or service-logon rights and the release signing key must be available before their dependent live/release phases.
- Final integration runs focused crate tests, relevant feature/portable checks, `make lint`, `make test`, generated CLI help, and documentation build. Do not mark the plan complete from simulated platform tests alone.

# Execution phases

## ACD-01 — Freeze portable job contract

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-service/src/job.rs`, `crates/acorn-service/src/lib.rs`
- Primary integration lock: Service/RPC contract
- Outcome: Define validated job IDs, kinds, triggers, run states, and pure coalescing transitions without host I/O; keep the alloc/WASM feature path building.
- Focused validation: `cargo test -p acorn-service`; `cargo check -p acorn-service --no-default-features --features alloc`.
- Handoff: Preserve the serialized contract in code/tests, remove ACD-01, and release the service contract lock.

## ACD-02 — Add daemon package and validated config

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-01
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/lib.rs`, `crates/acorn-daemon/src/main.rs`, `crates/acorn-daemon/src/config.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Workspace graph
- Outcome: Build `acornd` with a strict version-1 YAML config loader, paired CLI/cache-path dependencies, target paths, and a no-work foreground run mode; no service-manager registration yet.
- Focused validation: `cargo test -p acorn-daemon`; invalid-config CLI smoke; workspace metadata check.
- Handoff: Remove ACD-02 after recording the config shape; release the workspace graph lock.

## ACD-03 — Persist daemon job and run history

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-01, ACD-02
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-store/Cargo.toml`, `crates/acorn-store/src/lib.rs`, `crates/acorn-store/src/daemon.rs`, `crates/acorn-daemon/Cargo.toml`
- Primary integration lock: Workspace graph
- Outcome: Add a narrow SQLite adapter for job pause state and run history with atomic claim/finish, interrupted-run recovery, ordered queries, bounded retention, and the existing durable status/replay port mapped to run IDs; do not migrate existing `acorn-lib` tables.
- Focused validation: `cargo test -p acorn-store`; reopen/recovery, concurrent claim, migration, and retention fixtures.
- Handoff: Remove ACD-03, record the storage schema version, and release the workspace graph lock.

## ACD-04 — Execute fixed check jobs

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-02, ACD-03
- Exact write set: `crates/acorn-daemon/src/runner.rs`, `crates/acorn-daemon/src/lib.rs`
- Primary integration lock: Daemon runtime
- Outcome: Invoke the typed `acorn check` form using the paired CLI binary; capture its JSON report in a fresh standard project cache path and persist exit/report metadata.
- Focused validation: Fake executable tests for check arguments, working directory, findings, exit failure, timeout, bounded streams, and no shell expansion.
- Handoff: Remove ACD-04 after recording the check executable and artifact contract.

## ACD-05 — Execute fixed gather jobs

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-04
- Exact write set: `crates/acorn-daemon/src/runner.rs`
- Primary integration lock: Daemon runtime
- Outcome: Add the typed `acorn gather` form and an explicit report path under a fresh standard project cache path, reusing the check runner's process limits and history contract.
- Focused validation: Fake executable tests for gather arguments, report contents, expected command failure, timeout, and exclusion of the report path from watched inputs.
- Handoff: Remove ACD-05 after preserving gather-specific report behavior.

## ACD-06 — Add watcher and interval scheduling

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-01, ACD-03, ACD-04, ACD-05
- Exact write set: `crates/acorn-daemon/src/scheduler.rs`, `crates/acorn-daemon/src/lib.rs`
- Primary integration lock: Daemon runtime
- Outcome: Run one global worker with startup, debounced file-change, and fixed-interval triggers; persist pause state and collapse pending work into one rerun while active.
- Focused validation: Fake clock/event tests for duplicate events, missed intervals, in-flight changes, pause/resume, failed runs, and clean shutdown.
- Handoff: Remove ACD-06 and publish the observed trigger semantics for control integration.

## ACD-07 — Expose composable server lifecycle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-02
- Exact write set: `crates/acorn-server/src/config/mod.rs`, `crates/acorn-server/src/config/tests/mod.rs`, `crates/acorn-server/src/runtime.rs`, `crates/acorn-server/src/lib.rs`, `crates/acorn-server/src/runtime/tests.rs`
- Primary integration lock: Service/RPC contract
- Outcome: Add explicit secret injection and injected-registry/external-shutdown serving while preserving current standalone entry points, TLS checks, readiness, and listener drain.
- Focused validation: `cargo test -p acorn-server`; injected operation, signal, listener failure, and graceful drain tests.
- Handoff: Remove ACD-07 and release the service contract lock before job-control registration.

## ACD-08 — Register job control and coordinate daemon shutdown

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-03, ACD-06, ACD-07
- Exact write set: `crates/acorn-daemon/src/control.rs`, `crates/acorn-daemon/src/lib.rs`, `crates/acorn-daemon/src/main.rs`
- Primary integration lock: Daemon runtime
- Outcome: Compose worker and server; expose authenticated, mutation-gated job-control methods and only report readiness after store and scheduler initialization.
- Focused validation: Local HTTP integration tests for existing and job methods, auth/policy, reload rejection, worker failure, readiness, and drain.
- Handoff: Remove ACD-08 and freeze the CLI-facing control method names and result fields.

## ACD-09 — Add local CLI daemon management

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-08; CLI syntax lock released by its current owner
- Exact write set: `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/tests.rs`
- Primary integration lock: CLI contract
- Outcome: Add local status/history/run/pause/resume/reload and lifecycle command syntax using the protected local token, without changing existing `serve` commands.
- Focused validation: CLI parser and command tests for auth, errors, redaction, absent daemon, and stable exit behavior.
- Handoff: Remove ACD-09 and release the CLI syntax lock; defer generated help to integration.

## ACD-10 — Register persistent Linux service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-08, ACD-09
- Exact write set: `crates/acorn-daemon/src/install/linux.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Generate/install an owner-scoped systemd user unit, enable lingering where authorized, and provide start/stop/status/uninstall with idempotent cleanup.
- Focused validation: Unit rendering/preflight tests and local systemd syntax check; live logout/reboot acceptance belongs to ACD-20.
- Handoff: Remove ACD-10 after recording unit syntax and preflight behavior; ACD-20 owns live host evidence.

## ACD-11 — Register persistent macOS service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-08, ACD-09
- Exact write set: `crates/acorn-daemon/src/install/macos.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Register a system LaunchDaemon that runs as the chosen user, with correct ownership, boot startup, status, and uninstall behavior.
- Focused validation: Plist/preflight tests and launchd syntax check; live logout/reboot acceptance belongs to ACD-21.
- Handoff: Remove ACD-11 after recording plist syntax and preflight behavior; ACD-21 owns live host evidence.

## ACD-12 — Run acornd under Windows Service Control Manager

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-08
- Exact write set: `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/main.rs`, `crates/acorn-daemon/src/windows_service.rs`, `Cargo.lock`
- Primary integration lock: Workspace graph
- Outcome: Use a safe Windows Service wrapper to accept SCM start/stop events and drain worker/listeners within the service timeout while retaining foreground development mode; preserve the workspace ban on unsafe code.
- Focused validation: Cross-target compile and fake SCM start/stop/forced-termination tests; native service acceptance belongs to ACD-22.
- Handoff: Remove ACD-12 after recording the SCM event contract; release the workspace graph lock.

## ACD-13 — Register persistent Windows service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-09, ACD-12
- Exact write set: `crates/acorn-daemon/src/install/windows.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Install/start/stop/uninstall a Windows Service under the selected user. Registration obtains credentials through a protected target-side interactive prompt or a preprovisioned managed identity; no credential enters arguments, config, or logs.
- Focused validation: Registration/ACL tests with a fake SCM adapter; live logout/reboot acceptance belongs to ACD-22.
- Handoff: Remove ACD-13 after recording rights preflight and fake-SCM evidence; ACD-22 owns live host evidence and no credentials are retained.

## ACD-14 — Define signed bundle verification

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-09; maintainer provisions the release signing key and corresponding public key
- Exact write set: `Cargo.toml`, `Cargo.lock`, `crates/acorn-cli/Cargo.toml`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/bundle.rs`, `crates/acorn-cli/assets/acornd-release.pub`
- Primary integration lock: Workspace graph
- Outcome: Pin the release public key, define version/target/binary metadata, and verify a complete Minisign-signed bundle before extraction or execution.
- Focused validation: Valid signature, wrong key, altered manifest/binary, missing member, version mismatch, and archive path traversal fixtures.
- Handoff: Remove ACD-14 after recording the public-key fingerprint and bundle format; release the workspace graph lock.

## ACD-15 — Produce Linux release bundles

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-10, ACD-14; Linux x86_64 and ARM64 builders available
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
- Prerequisites: ACD-10, ACD-14, ACD-15; other platform adapters are exercised after their respective registration and bundle phases
- Exact write set: `crates/acorn-cli/src/commands/daemon/bundle.rs`, `crates/acorn-cli/src/commands/daemon/install.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: CLI contract
- Outcome: Stage a verified paired bundle and config, generate a protected token, invoke platform registration, verify readiness, and roll back a failed upgrade while preserving user data.
- Focused validation: Local install/reinstall/upgrade/uninstall fixtures for all platform adapters, plus one native host activation; corrupt signature, unsupported target, and failed-readiness rollback.
- Handoff: Remove ACD-18 and freeze the installer transaction contract.

## ACD-19 — Implement SSH transfer and remote control

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-09, ACD-14, ACD-18
- Exact write set: `crates/acorn-cli/src/commands/daemon/remote.rs`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/tests.rs`
- Primary integration lock: CLI contract
- Outcome: Detect remote target, verify and upload a signed bundle, invoke fixed host commands, check remote bytes/readiness, and support `--bundle` for offline hosts without transferring the token.
- Focused validation: Fake SSH tests for quoting, target mismatch, interrupted transfer, elevation denial, secret redaction, idempotence, and rollback.
- Handoff: Remove ACD-19 after freezing remote command and transfer behavior; release the CLI contract lock.

## ACD-20 — Accept SSH installation on Linux

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACD-10, ACD-15, ACD-19; named Linux SSH host and approved lingering/elevation
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
- Prerequisites: ACD-13, ACD-17, ACD-19; named Windows SSH host, administrator approval, and provisioned service account
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
- Prerequisites: ACD-01 through ACD-23; five bundles and all three OS acceptance records available
- Exact write set: repository CI and named Linux, macOS, and Windows acceptance records
- Primary integration lock: CI/build
- Outcome: Run final workspace gates, review native host evidence against every acceptance criterion, and retire this plan only when no required work remains.
- Focused validation: `make lint`, `make test`, `make check-portable`, bundle signature verification, and clean documentation build.
- Handoff: Move any remaining durable facts to code/docs, delete this concept, and remove all wiki index, network, and lane references.
