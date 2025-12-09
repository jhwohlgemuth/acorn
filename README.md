<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/src/_assets/banner_dark_mode.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/src/_assets/banner_light_mode.png">
    <img src="docs/src/_assets/banner_light_mode.png" alt="ACORN Banner" width="350px">
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

:seedling: ACORN provides and operationalizes an ontology for research activity data (RAD) and enables adding linked data context and transforming RAD into a knowledge graph that is amenable to automated reasoning and artifact generation (e.g. PDFs, PPTX, etc.)

[[_TOC_]]

## So what, big deal, who cares?
ACORN is a command line multi-tool that employs automated processes for informing and enforcing defined content schemas. With these content schemas, ACORN builds communication assets such as PDFs, presentation files, and web pages. It also lays the foundation for deep data insights about ORNL’s — and any institution’s — corpus of research. Built using the memory-safe Rust programming language, ACORN can be used on any Windows, Mac, or Linux machine

## Installation
See [the ACORN book](https://acorn.ornl.gov) for detailed installation instructions for the [CLI application](http://acorn.ornl.gov/getting-started.html), [🦀Rust crate](http://acorn.ornl.gov/packages/acorn-lib.html#installation), [🐍Python package](http://acorn.ornl.gov/packages/acorn-py.html#installation), and [WebAssembly package](http://acorn.ornl.gov/packages/acorn-web.html#installation).

## Architecture
> See [ARCHITECTURE.md](./ARCHITECTURE.md)

## Contributing
> See [CONTRIBUTING.md](./CONTRIBUTING.md)