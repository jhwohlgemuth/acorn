# MCP Bundles

ACORN packages its local stdio Model Context Protocol server as a target-specific `.mcpb` archive. The bundle contains `manifest.json` and one `server/acorn` or `server/acorn.exe` executable. The manifest follows MCPB `0.3`, pinned to upstream revision `70fe3b34cd6dff1b3bba046638edc72a6467a4fb`, and declares the same safe online read-only tools as `acorn serve mcp`.

Create a local bundle with `acorn create mcp`. Use `--binary` and `--platform darwin|linux|win32` for a release or cross-compiled executable. `--dry-run` prints the manifest, inventory, and destination without writing. `--kitfile ./Kitfile` creates the bundle first and then idempotently upserts an authoring `mcpServers` entry named `acorn` using a relative path; resolved OCI layer identity is never written to authoring YAML.

Direct clients can run `acorn serve mcp` or use `acorn sync --mcp`. Synchronization configures the absolute executable path and does not install a bundle. A client that supports MCPB installation owns extraction and approval.

The default server is read-only. Global `--offline` removes tools that may contact external systems. Persistent mutation requires `serve mcp --allow-mutation` and an enabled local database. Bundles do not execute during creation, inspection, ModelKit packaging, or download; inspect their source and SHA-256 before installation.

Official release bundles target `x86_64-unknown-linux-musl` and `x86_64-pc-windows-gnu`, each with a SHA-256 sidecar and validation by `@anthropic-ai/mcpb@2.1.2`. Local macOS packaging is possible only by supplying a supported target binary contract; ACORN does not publish or claim a signed macOS MCPB release.
