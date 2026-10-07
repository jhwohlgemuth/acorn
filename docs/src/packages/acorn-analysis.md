# Portable analysis

`acorn-analysis` provides ACORN findings, batches, summaries, validation issue folding, FAIR evaluation, and readability metrics. Its operations consume caller-supplied values and compile with `alloc` on native and `wasm32-unknown-unknown` targets.

```bash
cargo add acorn-analysis
```

Filesystem discovery, Vale execution, terminal rendering, fixes, inference, and databases remain native behavior in `acorn-lib` and its adapters.
