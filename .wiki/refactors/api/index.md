# API Refactor Plans

## In progress

- [Outbound MCP client embedder interface](outbound-mcp-client-interface.md) - Document the landed direct-construction and cancellation contract, then record portable/WASM evidence before retirement.
- [Shared API client contract](shared-client-contract.md) - Complete endpoint, request, invocation, resolved configuration, response, error, health, OpenAPI, and RAiD workflow boundaries in dependency order.

Provider-specific model request and metadata work now lives in [model acquisition and metadata hardening](../../features/model-acquisition-hardening.md). New provider plans should consume the shared contract and retain provider-specific policy in their leaf modules.
