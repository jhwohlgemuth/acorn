---
type: Plan
title: "Refine the RAiD API module"
description: "Turn the removed gather-command RAiD prototype into validated service-point and record workflows."
difficulty: medium
impact: medium
package: [acorn-lib]
tags: [refactor, api, raid]
status: planned
---

# Goal

Provide focused, testable RAiD API operations without embedding API experiments in unrelated CLI commands.

# Current evidence

`crates/acorn-lib/src/io/api/raid.rs` exposes service-point listing and lookup, record listing and lookup, and record creation. The removed gather prototype exercised service point `20000033`, record retrieval, metadata construction, schema validation, JSON serialization, and record creation, but it did so as command-local example code rather than tests or supported workflows.

# Recommendation

Move the useful prototype scenarios into the RAiD API module as typed request workflows and fixture-backed tests. Cover service-point collection and lookup, record collection and lookup, metadata validation before creation, sandbox configuration, authentication failures, and structured API error responses. Keep live create tests ignored and require explicit sandbox credentials.

# Validation

Run the RAiD unit and fixture tests offline, then run ignored sandbox integration tests with dedicated credentials. Verify that invalid metadata is rejected before transport and that list, lookup, and create responses retain their typed schemas.

# Acceptance criteria

- Service-point and record list and lookup operations have fixture-backed tests.
- Metadata is validated before a create request is sent.
- Sandbox configuration is explicit and cannot target production accidentally.
- Error envelopes preserve field-level RAiD validation failures.
- Live write tests are ignored by default and document their credential requirements.
