# Packages

ACORN publishes separate packages for its portable schemas, host services, CLI, and WebAssembly bindings. The Python bindings are maintained in a companion project.

```mermaid
graph LR
    CORE["acorn-core</br>portable values"] --> H["acorn-host</br>native adapters"]
    M["acorn-cmd</br>native commands"] --> H
    S["acorn-schema</br>portable Rust schemas"] --> A["acorn-lib</br>services and analysis"]
    H --> A
    M --> A
    A --> B["acorn-cli</br>command-line application"]
    A -.-> PY["acorn-py</br>companion project"]
    A --> D["acorn-wasm</br>JavaScript package"]
```

Use [`acorn-schema`](acorn-schema.md) when an application only needs the data model, validation, or persistent identifiers. Small native tools can use [`acorn-cmd`](acorn-cmd.md) and [`acorn-host`](acorn-host.md) without enabling `acorn-lib`'s analysis, database, document, or export graph. Use [`acorn-lib`](acorn-lib.md) for ACORN's higher-level analysis, persistence, model APIs, and artifact workflows.

Use an [ACORN MCP Bundle](mcpb.md) to install the target-specific CLI and its static, read-only MCP tool declaration as one validated archive.
