# ACORN 🌱
[![Latest Release](https://code.ornl.gov/research-enablement/acorn/-/badges/release.svg?style=flat-square)](https://code.ornl.gov/research-enablement/acorn/-/releases)
> Accessible Content Optimization for Research Needs

## Installation
> Homebrew and Scoop packages are planned. Check back soon for updates.
### Download pre-compiled binary
- Download newest binary from [releases page](https://code.ornl.gov/research-enablement/acorn/-/releases)
    ```shell
    curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-gnu/v0.1.24/acorn
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

### Install with cargo
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