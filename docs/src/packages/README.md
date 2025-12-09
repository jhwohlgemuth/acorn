# 📦 Packages

To enable broad adoption of ACORN, we provide packages for multiple popular programming contexts.

```mermaid
graph LR
    A["acorn-lib</br>(Rust crate)"] -->|"&nbsp;is dependency of&nbsp;"|B["acorn-cli</br>(Rust crate)"]
    A -->|"generates</br>&nbsp;bindings for&nbsp;"|C["acorn-py</br>(Python package)"]
    A -->|"transpiles</br>&nbsp;API subset to&nbsp;"|D["acorn-web</br>(WASM package)"]
```