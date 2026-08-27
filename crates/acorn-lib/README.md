# 🌱 ACORN Library
[<img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-research_enablement-00662C?style=for-the-badge&logo=gitlab&color=%2300662C
" height="20">](https://code.ornl.gov/research-enablement/acorn)
[<img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-00662C?style=for-the-badge&logo=rust" height="20">](https://github.com/rust-secure-code/safety-dance/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
[<img alt="pipeline" src="https://code.ornl.gov/research-enablement/acorn/badges/main/pipeline.svg?style=flat-square">](https://code.ornl.gov/research-enablement/acorn/-/pipelines)

`acorn-lib` is a one-stop-shop for everything related to building and maintaining research activity data (RAD)-related technology, including the Accessible Content Optimization for Research Needs ([ACORN](https://acorn.ornl.gov)) tool.

The modules, structs, enums and constants found here support the ACORN CLI, which checks, analyzes, and exports research activity data into useable formats.

## Validation imports

ACORN's pre-1.0 validation API is available from `acorn::validation`:

```rust
use acorn::validation::{Validate, ValidationError, ValidationReport};
```

The `Validate` derive macro is re-exported beside the trait and continues to use `#[validate(...)]` attributes. Code importing `validator::{Validate, ValidationError, ValidationErrors}` should migrate to the ACORN types above; `ValidationReport` is a deterministic flat collection whose issue paths use forms such as `field.subfield[2]`.
