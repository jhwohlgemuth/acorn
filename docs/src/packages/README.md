# Packages

ACORN publishes separate packages for its portable schemas, host services, CLI, and WebAssembly bindings. The Python bindings are maintained in a companion project.

```mermaid
graph LR
    CORE["acorn-core</br>portable values"] --> H["acorn-host</br>native adapters"]
    DIFF["acorn-diff</br>portable diffs"] --> A
    M["acorn-cmd</br>native commands"] --> H
    S["acorn-schema</br>portable Rust schemas"] --> ANALYSIS["acorn-analysis</br>portable findings and metrics"]
    ANALYSIS --> SERVICE["acorn-service</br>application operations"]
    S --> SERVICE
    S --> A["acorn-lib</br>native services"]
    ANALYSIS --> A
    H --> A
    M --> A
    A --> B["acorn-cli</br>command-line application"]
    A -.-> PY["acorn-py</br>companion project"]
    SERVICE --> D["acorn-wasm</br>JavaScript package"]
    SERVICE --> SERVER["acorn-server</br>JSON-RPC and typed gRPC"]
```

Use [`acorn-diff`](acorn-diff.md) for in-memory diffs, [`acorn-schema`](acorn-schema.md) for data models and validation, [`acorn-analysis`](acorn-analysis.md) for portable findings and readability, and [`acorn-service`](acorn-service.md) for supplied-content application operations. Small native tools can use [`acorn-cmd`](acorn-cmd.md) and [`acorn-host`](acorn-host.md). Use [`acorn-lib`](acorn-lib.md) for native discovery, persistence, providers, documents, and artifact workflows.

Use an [ACORN MCP Bundle](mcpb.md) to install the target-specific CLI and its static, read-only MCP tool declaration as one validated archive.
