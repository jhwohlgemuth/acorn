# ACORN 🌱
[<img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-research_enablement-00662C?style=for-the-badge&logo=gitlab&color=%2300662C
" height="20">](https://code.ornl.gov/research-enablement/acorn)
[<img alt="unsafe forbidden" src="https://img.shields.io/badge/unsafe-forbidden-00662C?style=for-the-badge&logo=rust" height="20">](https://github.com/rust-secure-code/safety-dance/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/acorn-cli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/acorn-cli)
[![Latest Release](https://code.ornl.gov/research-enablement/acorn/-/badges/release.svg?style=flat-square)](https://code.ornl.gov/research-enablement/acorn/-/releases)
> Accessible Content Optimization for Research Needs

## Installation
> Homebrew and Scoop packages are planned. Check back soon for updates.
### Install with cargo
- Install `acorn` command
  ```shell
  cargo install acorn
  ```
### Download pre-compiled binary
- Download newest binary from [releases page](https://code.ornl.gov/research-enablement/acorn/-/releases)
    ```shell
    curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-gnu/v0.1.26/acorn
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

## Architecture
> See [ARCHITECTURE.md](./ARCHITECTURE.md)

## Roadmap
> 🚧 Under Construction

## Contributing
> See [CONTRIBUTING.md](./CONTRIBUTING.md)