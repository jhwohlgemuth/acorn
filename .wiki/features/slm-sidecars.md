---
type: Plan
title: Ask routing and sidecar coordination
description: Coordinate native capability parity, generic adapters, and independently gated Swift and Julia sidecar profiles.
difficulty: mixed
impact: high
package: [acorn-lib, acorn-cli, acorn-schema, acorn-core]
tags: [coordination, inference, routing, sidecar]
status: in-progress
execution: coordination
---

# Goal

Coordinate four atomic outcomes without assigning this record as implementation work. Each child owns its remaining phases, evidence, validation, and acceptance criteria.

# Child plans and order

1. [Needle routing and native capability parity](needle-router-native-parity.md) establishes the catalog, caller-owned registry, bounded terminal execution, and measured native parity.
2. [Generic ask executor adapters](ask-executor-adapters.md) implements configured MCP, ACP, endpoint, and local-sidecar bindings after the registry and execution contracts land; heterogeneous release waits for native parity.
3. [Swift SLM capability profile](swift-slm-profile.md) builds on generic sidecar registration and releases Tongue, Redact, and Title after generic router release, legal approval, build hosts, and quality gates.
4. [Julia decision sidecar profile](julia-decision-sidecar.md) builds on generic sidecar registration and releases advisory Choice and Noul decisions independently of the Swift profile after generic router release and its own parity, quality, and resource gates.

The [dependency network](../plan-network.md) and [parallel guide](../parallel-execution.md) own cross-plan sequencing and shared locks. Unfinished phase IDs remain stable in the child plans.
