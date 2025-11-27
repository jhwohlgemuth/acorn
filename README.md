<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/src/assets/banner_dark_mode.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/src/assets/banner_light_mode.png">
    <img src="docs/src/assets/banner_light_mode.png" alt="ACORN Banner" width="350px">
  </picture>
</div>
<br>
<br>

[<img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-research_enablement-00662C?style=for-the-badge&logo=gitlab&color=%2300662C
" height="20">](https://code.ornl.gov/research-enablement/acorn)
[<img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-00662C?style=for-the-badge&logo=rust" height="20">](https://github.com/rust-secure-code/safety-dance/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-cli)
[![Latest Release](https://code.ornl.gov/research-enablement/acorn/-/badges/release.svg?style=flat-square)](https://code.ornl.gov/research-enablement/acorn/-/releases)
> Accessible Content Optimization for Research Needs

:seedling: ACORN provides an ontology for research activity data (RAD) and enables adding linked data context and transforming RAD into a knowledge graph that is amenable to automated reasoning and artifact generation (e.g. PDFs, PPTX, etc.)

[[_TOC_]]

## So what, big deal, who cares?
ACORN is a command line multi-tool that employs automated processes for informing and enforcing defined content schemas. With these content schemas, ACORN builds communication assets such as PDFs, presentation files, and web pages. It also lays the foundation for deep data insights about ORNL’s — and any institution’s — corpus of research. Built using the memory-safe Rust programming language, ACORN can be used on any Windows, Mac, or Linux machine

## Installation
> Homebrew and Scoop packages are planned. Check back soon for updates.
### Install with cargo
- Install `acorn` command
  ```shell
  cargo install acorn-cli
  ```

- Test the installation
  ```shell
  acorn help
  ```

### Download pre-compiled binary
- Download newest binary from [releases page](https://code.ornl.gov/research-enablement/acorn/-/releases)
    ```shell
    curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-musl/v0.1.33/acorn
    ```
- Make binary executable
    ```shell
    chmod +x ./acorn
    ```
- Move binary to folder on path (ex. `/usr/local/bin`)
    ```shell
    mv acorn /usr/local/bin
    ```
- Verify installation with `acorn --version`
> ⚠️ **CAUTION** `acorn export` is [not currently supported on MacOS](https://code.ornl.gov/research-enablement/acorn/-/issues/4)

### Build from source
- Clone this project and navigate to it
  ```shell
  git clone https://code.ornl.gov/research-enablement/acorn
  cd ./acorn
  ```

- Install `acorn` command
  ```shell
  cargo install --path ./acorn-cli
  ```

- Test the installation
  ```shell
  acorn help
  ```

### ACORN Rust Libary
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-lib)
- Add `acorn-lib` as a dependency in your `Cargo.toml`
  ```toml
  [dependencies]
  acorn-lib = "0.1.30"
  ```  
- Use `acorn-lib` functions in your Rust code
  ```rust
  use acorn_lib::schema::validate::{is_doi, is_orcid, is_ror};

  assert!(is_doi("10.11578/dc.20250604.1"));
  assert!(is_orcid("https://orcid.org/0000-0002-2057-9115"));
  assert!(is_ror("01qz5mb56"));
  ```
- If you want to use `acorn-lib` to build your own CLI app, be sure to include the following in your `Cargo.toml` to enable the `cli` feature
  ```toml
  [dependencies]
  acorn-lib = { version = "0.1.30", features = ["cli"] }
  ```

### Python Bindings
[<img alt="PyPI" src="https://img.shields.io/pypi/v/acorn-py?style=for-the-badge&logo=python" height="20">](https://pypi.org/project/acorn-py/)
> ⚠️ **CAUTION** `acorn-py` API is currently experimental and may change without warning.
- Install `acorn-py` package
  ```shell
  pip install acorn-py
  ```
- Use `acorn-lib` functions in Python
  ```python
  from acorn.schema.validate import is_doi, is_orcid, is_ror

  assert is_doi("10.11578/dc.20250604.1")
  assert is_orcid("https://orcid.org/0000-0002-2057-9115")
  assert is_ror("01qz5mb56")
  ```

## Architecture
> See [ARCHITECTURE.md](./ARCHITECTURE.md)

## Contributing
> See [CONTRIBUTING.md](./CONTRIBUTING.md)