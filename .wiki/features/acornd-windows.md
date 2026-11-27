---
type: Plan
title: Windows acornd service, bundle, and acceptance
description: Run acornd under Windows Service Control Manager, add owner-verified local control, publish the signed Windows bundle, and accept local and SSH installation on a native Windows host.
difficulty: high
impact: medium
package: [acorn-daemon, acorn-cli]
tags: [feature, daemon, service, windows, release, ssh, os:windows, x86_64]
status: planned
execution: phased
phase_budget_hours: 5
created: "2026-11-26"
updated: "2026-11-26"
---

# Goal

Deliver the Windows-only remainder of the [acornd background service](acornd-service.md): SCM hosting, persistent service registration, owner-verified local control, a signed Windows x86_64 bundle, local signed activation, and SSH installation. Platform-neutral, Linux, and macOS work stays in the ACD plan; every phase here needs a native Windows OS host or VM with administrator access.

# Current evidence — November 26, 2026

- The portable daemon, server, store, signed-bundle verifier, Linux and macOS acceptance, and the CLI management surface are accepted under ACD. See that plan's current evidence for details.
- ACDW-12 lacks a verified reachable native Windows host or VM with administrator access. The recorded Windows shell runner still awaits a completed native smoke job (see [GitLab shell runner readiness](../testing/gitlab-shell-runner-readiness.md)).
- This plan was split from ACD on November 26; phase IDs were renamed from `ACD-12`, `ACD-13`, `ACD-17`, `ACD-19W`, `ACD-22`, `ACD-26`, and `ACD-33` to the `ACDW-` prefix. Prerequisites and handoffs are otherwise unchanged.

# Boundaries and dependencies

- Prerequisite from ACD: accepted foreground daemon composition, local CLI management, and the native macOS MVP. ACDW-17 also needs ACD's accepted signed bundle verification.
- ACDW-19 shares `crates/acorn-cli/src/commands/daemon/remote.rs` with ACD-19M and serializes with it. ACDW-12 and ACDW-17 claim the workspace graph and release/CI locks that ACD-16 also uses; run them one at a time.
- The daemon runs as the selected user; only SCM registration crosses the administrator boundary. No credentials enter arguments, config, or logs.
- Cross-target compilation and fake SCM, peer and SSH tests cannot replace native Windows execution. Record the exact tested host and version.
- ACD-24 final acceptance waits for ACDW-22 and ACDW-33 records.

# Execution phases

## ACDW-12 — Run acornd under Windows Service Control Manager

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: accepted foreground daemon composition and native macOS MVP; native Windows OS host or VM with administrator access available for temporary test registration and phase acceptance
- Exact write set: `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/main.rs`, `crates/acorn-daemon/src/windows_service.rs`, `Cargo.lock`
- Primary integration lock: Workspace graph
- Outcome: Use a safe Windows Service wrapper to accept SCM start/stop events and drain worker/listeners within the service timeout while retaining foreground development mode; preserve the workspace ban on unsafe code.
- Focused validation: Cross-target compile and fake SCM start/stop/forced-termination tests, followed by actual SCM start/stop and worker/listener shutdown through a temporary test registration on Windows. ACDW-22 owns logout/reboot and remote deployment acceptance.
- Handoff: Remove ACDW-12 after recording the SCM event contract and native Windows start/stop evidence; release the workspace graph lock.

## ACDW-13 — Register persistent Windows service

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: accepted local CLI management and ACDW-12; native Windows OS host or VM with administrator access and a test service identity available for phase acceptance
- Exact write set: `crates/acorn-daemon/src/install/windows.rs`, `crates/acorn-daemon/src/install/mod.rs`
- Primary integration lock: Host service registration
- Outcome: Install/start/stop/uninstall a Windows Service under the selected user. Registration obtains credentials through a protected target-side interactive prompt or a preprovisioned managed identity; no credential enters arguments, config, or logs.
- Focused validation: Registration/ACL tests with a fake SCM adapter, then native Windows install/start/stop/uninstall and service-identity/rights checks. ACDW-22 owns logout/reboot and remote deployment acceptance.
- Handoff: Remove ACDW-13 after recording rights preflight, fake-SCM tests, and native Windows registration evidence; no credentials are retained.

## ACDW-26 — Add owner-verified Windows local control

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACDW-12 and ACDW-13; Windows service identity and ACL contract available; native Windows OS host or VM available for phase acceptance
- Exact write set: `crates/acorn-cli/Cargo.toml`, `crates/acorn-cli/src/commands/daemon/mod.rs`, `crates/acorn-cli/src/commands/daemon/windows.rs`, `crates/acorn-daemon/Cargo.toml`, `crates/acorn-daemon/src/lib.rs`, `crates/acorn-daemon/src/local/windows.rs`, `crates/acorn-server/Cargo.toml`, `crates/acorn-server/src/client/mod.rs`, `crates/acorn-server/src/runtime.rs`, `Cargo.lock`
- Primary integration lock: Local daemon control transport
- Outcome: Give the Windows CLI an owner-verified local control channel to the selected service identity, with no bearer token sent to an unverified loopback listener.
- Focused validation: Fake peer and ACL denial tests and a Windows target compile, followed by native Windows peer-identity denial, stopped/restarting service behavior, and local job-control round trips. ACDW-22 owns remote deployment and persistence acceptance.
- Handoff: Remove ACDW-26 after the control channel and fail-closed behavior pass on the Windows OS test environment; release the local transport lock.

## ACDW-17 — Produce Windows release bundle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACDW-13 and accepted signed bundle verification; native Windows x86_64 build and runtime test environment and protected release signing access available
- Exact write set: `Makefile`, `.gitlab-ci.yml`, `scripts/upload-binaries.sh`, `scripts/package-acornd.ps1`
- Primary integration lock: CI/build
- Outcome: Build, sign, and publish a paired Windows x86_64 bundle, including the SCM-capable daemon.
- Focused validation: Build and run the paired binaries on Windows, then check the signature round trip, target metadata, and downloaded artifact on that OS.
- Handoff: Remove ACDW-17 after recording artifact and builder evidence; release the CI/build lock.

## ACDW-33 — Accept local signed activation on Windows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACDW-13, ACDW-17, and ACDW-26; native Windows OS test environment with a signed bundle available
- Exact write set: `crates/acorn-cli/src/commands/daemon/install.rs`, `crates/acorn-daemon/src/install/mod.rs`, `crates/acorn-daemon/src/install/windows.rs`, `crates/acorn-daemon/acceptance/windows-mvp.md`, and the named Windows OS test environment
- Primary integration lock: External Windows service host
- Outcome: Activate and replace a verified paired Windows bundle through SCM, preserve user data, and restore the prior release after failed readiness.
- Focused validation: On the Windows OS test environment, run bundle install/reinstall/upgrade, signature and target rejection, local control and jobs, failed-readiness rollback, and data-preserving uninstall.
- Handoff: Record native Windows OS release and rollback evidence before removing ACDW-33.

## ACDW-19 — Extend SSH transfer and control to Windows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACDW-17 and ACDW-33
- Exact write set: `crates/acorn-cli/src/commands/daemon/remote.rs`, `crates/acorn-cli/src/commands/daemon/tests.rs`, `CommandLineHelp.md`, `README.md`
- Primary integration lock: CLI contract
- Outcome: Detect Windows x86_64 over SSH, verify and transfer its signed bundle, invoke native SCM activation and fixed host commands, and check transferred bytes and readiness without moving the token.
- Focused validation: Fake Windows SSH tests for target selection, transfer, quoting, elevation denial, and rollback.
- Handoff: Remove ACDW-19 before Windows SSH host acceptance.

## ACDW-22 — Accept SSH installation on Windows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ACDW-19 and ACDW-33; named native Windows OS SSH test host or VM, administrator approval, and provisioned service account
- Exact write set: named Windows SSH test host and its Service Control Manager
- Primary integration lock: External Windows service host
- Outcome: Prove signed remote install, update, control, logout/reboot persistence, rollback, and uninstall on Windows.
- Focused validation: On the Windows OS test host, capture SCM status, health, job report/history, before/after version, logout and post-reboot results, rollback, and uninstall without exposing credentials.
- Handoff: Remove ACDW-22 after preserving host/version evidence in durable deployment documentation.

