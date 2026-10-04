# Core Refactor Plans

## In Progress

- [Remote create container targets](remote-create-container-targets.md) - Add preflight, rollback, cleanup, and live validation for remote container creation.
- [Shared export and OOXML pipeline](export-pipeline-hardening.md) - Complete one export projection, portable interpolation, parser-backed atomic PowerPoint mutation, and fuzz coverage.

## Planned

- [Portable analysis and service boundary](acorn-service-boundary.md) - Extract supplied-content analysis and service operations, then host them through JSON-RPC and typed gRPC in `acorn-server`.
- [CLI composition boundary](cli-composition-boundary.md) - Normalize parser values, isolate startup, share remote serve dispatch, and map reports/exits once.
- [Structured research-activity enrichment](research-activity-enrichment-integration.md) - Map provider and logbook evidence into the existing typed activity fields with deterministic merge behavior.
