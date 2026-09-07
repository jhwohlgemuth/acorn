# Packages

ACORN publishes separate packages for its portable schemas, host services, CLI, and WebAssembly bindings. The Python bindings are maintained in a companion project.

```mermaid
graph LR
    S["acorn-schema</br>portable Rust schemas"] --> A["acorn-lib</br>host services and analysis"]
    A --> B["acorn-cli</br>command-line application"]
    A -.-> C["acorn-py</br>companion project"]
    A --> D["acorn-wasm</br>JavaScript package"]
```

Use [`acorn-schema`](acorn-schema.md) when an application only needs the data model, validation, or persistent identifiers. Use [`acorn-lib`](acorn-lib.md) for filesystem and network I/O, analysis, persistence, model APIs, or artifact generation.
