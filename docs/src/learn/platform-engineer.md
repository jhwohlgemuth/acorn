# Platform engineer trail

Goal: make ACORN repeatable across repositories, services, and local inference tools.

<div class="acorn-growth emoji-growth five-stages" aria-label="Platform engineer trail from seed to tree"><span>Configure</span><span>Download</span><span>Models</span><span>Sync</span><span>Serve</span></div>

1. Define sources and defaults with [ACORN configuration](../cli-reference/configuration.md).
2. Aggregate versioned buckets with [`download`](../cli-reference/commands/download.md).
3. Follow the [model workflow](../cli-reference/model-workflows.md) from metadata import through weight selection.
4. Route downloaded models to applications with [`sync`](../cli-reference/commands/sync.md).
5. Use [`serve`](../cli-reference/commands/serve.md) for supported GitLab bot automation, the local Model Context Protocol (MCP)[^mcp] server, or authenticated JSON-RPC and [gRPC](../packages/grpc.md) endpoints. Distribute [`acorn skill`](../cli-reference/commands/skill.md) for compatible agents, and keep [`doctor`](../cli-reference/commands/doctor.md) nearby as the rescue guide.

For CI commands that support it, use global `--offline` mode when the job must be deterministic and ensure every required network-backed asset is cached or configured explicitly. Commands reject offline mode when they cannot honor that guarantee.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Define the workflow in <a href="../cli-reference/configuration.md">ACORN configuration</a>.</p>

[^mcp]: Model Context Protocol, ["Architecture"](https://modelcontextprotocol.io/specification/2025-06-18/architecture) and ["Server features"](https://modelcontextprotocol.io/specification/2025-06-18/server/index).
