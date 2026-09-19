---
type: Plan
title: "Automate Power Apps logbook intake and artifact reporting"
description: "Turn authenticated Power Automate form submissions into reviewed logbook changes and bounded artifacts delivered through GitLab and Power Automate."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: in-progress
---

# Goal

Deliver one reviewed, duplicate-safe path from Microsoft Forms through a rolling GitLab merge request, selective logbook graduation, bounded PDF creation, immutable GitLab package publication, and Power Automate notifications. ACORN generates the intake credential so setup does not depend on OpenSSL or an HTTPS security gateway.

# Current evidence

- `crates/acorn-lib/src/io/api/powerautomate/` implements strict form decoding, project and submitter authorization, shared-header authentication, durable intake, and direct callback resolution.
- `crates/acorn-lib/src/io/api/webhooks/` provides constant-time shared-secret verification, durable idempotent operations, retryable callbacks, readiness, and graceful drain state.
- `crates/acorn-lib/src/io/api/gitlab/` implements rolling merge-request automation, authorized graduation commands, and immutable report-package publication.
- `crates/acorn-cli/src/commands/report/` composes bounded artifact creation and package publication.
- `docs/src/automation/powerautomate-reporting.md` documents the direct Power Automate workflow and its Mermaid sequence diagram.
- Automated repository tests cover the workflow. A manual deployment smoke test remains before this plan is complete.

# Architecture decisions

- `acorn create secret` prints one 256-bit lowercase hexadecimal value sourced from the operating system cryptographic random-number generator. The `--clipboard` option copies the value without exposing it on standard output.
- Secret generation is generic and lives below `acorn create`; it is not coupled to Power Automate wire types.
- ACORN either prints the value or copies it to the system clipboard and does not persist it. The operator places the same value in the ACORN runtime and the Power Automate intake flow.
- Intake requests send the value in `x-acorn-webhook-secret` and a stable submission UUID in `x-acorn-delivery-id` over HTTPS. ACORN compares the credential in constant time and deduplicates by the normalized form submission identifier.
- Outbound callbacks use the SAS credential embedded in the Power Automate HTTP trigger URL. The MVP requires the trigger's `Anyone` mode; deployments that prohibit it need an approved intermediary or Microsoft Entra service-principal support in ACORN.
- Power Automate owns recipient selection and message formatting. ACORN sends artifact metadata and never embeds GitLab credentials or PDF bytes in callback bodies.

# Public interfaces

- `acorn create secret` writes one newline-terminated 64-character lowercase hexadecimal secret to standard output.
- `acorn create secret --clipboard` copies the secret to the system clipboard, suppresses the value on standard output, and reports clipboard failure as a command error.
- `acorn::util::generate_secret` returns a newly generated secret or an operating-system randomness error.
- `POST /webhooks/powerautomate/forms` requires `Content-Type: application/json`, `x-acorn-delivery-id`, and `x-acorn-webhook-secret`.
- `PowerAutomateEnvironment` names `POWER_AUTOMATE_INBOUND_SECRET` and `POWER_AUTOMATE_CALLBACK_URL`; configuration never contains their values.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/util/std.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/commands/{mod,create/mod}.rs`, CLI and utility tests, generated CLI help, workflow documentation, and planning indexes.
- **Create:** `crates/acorn-cli/src/commands/create/secret.rs` and its CLI integration test.
- **Delete:** None while the deployment smoke test remains. Delete this plan and its index/network entries after that final acceptance check.

# Dependencies

The workflow consumes the implemented webhook runtime, GitLab repository automation, reporting-window selection, PDF renderer, and package manifest contracts. Secret generation uses the existing `rand` dependency and does not add a package or external executable dependency.

# Implementation steps

## 1. Add generic secret generation

Expose a small library function backed by `OsRng`, add `acorn create secret`, and support either shell capture or explicit clipboard delivery without printing the secret.

- **Verify:** library tests prove the 64-character lowercase hexadecimal contract and distinct invocations; CLI tests prove successful standard output, option parsing, and clipboard delivery through an injected copier.

## 2. Make ACORN generation the documented bootstrap

Replace external secret-generation commands with `acorn create secret --clipboard`. Show the operator pasting the value into `POWER_AUTOMATE_INBOUND_SECRET` and `x-acorn-webhook-secret`, retain a headless standard-output example, and include the clipboard path in the sequence diagram.

- **Verify:** the documentation book renders and repository search finds no obsolete external generation command in the Power Automate guide.

## 3. Validate the deployment

Run one form submission through the direct intake flow, verify HTTP `202`, retry the same submission without duplication, graduate and merge its entry, publish an artifact, and receive the callback through the configured SAS URL.

- **Verify:** the deployment produces one reviewed entry, one immutable package version, and one notification for the accepted artifact.

# Validation

- Run focused utility, CLI parsing, CLI integration, and Power Automate tests.
- Regenerate `CommandLineHelp.md` and build the documentation book.
- Run `make lint` and `make test` before deployment.
- Confirm the default command emits no labels, diagnostics, or surrounding text on standard output and clipboard mode emits no secret value there.
- Confirm generated values, shared-secret headers, callback URLs, and SAS query parameters do not appear in logs or flow run histories.

# Acceptance criteria

- ACORN generates the intake secret without invoking an external program and can copy it directly to the system clipboard.
- The generated secret contains 256 bits of operating-system randomness and is represented as 64 lowercase hexadecimal characters.
- The intake flow and ACORN runtime use the same generated value without persisting it in repository configuration.
- Missing or incorrect intake credentials fail closed.
- Duplicate submissions remain idempotent.
- The setup guide and sequence diagram show ACORN generating and distributing the secret before form intake begins.
- One deployed end-to-end smoke test completes before this plan is removed as finished.
