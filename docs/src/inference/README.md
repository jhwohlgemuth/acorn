# Local inference and tools

ACORN can publish a conservative tool catalog through two local interfaces:

- a managed [Needle sidecar](needle.md) for small, offline tool-selection loops; and
- `acorn serve mcp`, a standard-input/standard-output Model Context Protocol server for external agents.

Both interfaces are projections of the same registry and invoke the same handlers. A tool being described to a model does not grant extra authority: ACORN still enforces its exposure and execution policy at dispatch time.
