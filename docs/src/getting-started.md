# 🚀 Getting Started

> [!IMPORTANT]
> The most popular way to use **ACORN** is to not install it at all --- simply leverage the power and promise of **ACORN** within a <span title="Continuous Integration">CI</span> job (see this [example `.gitlab-ci.yml`](https://code.ornl.gov/research-enablement/templates/-/blob/main/bucket.gitlab-ci.yml))

## Installation

### Install with Scoop
> [!WARNING]  
> 🚧 Under construction

### Install with Homebrew
> [!WARNING]  
> 🚧 Under construction

### Install with Cargo
- Ensure you have Rust and Cargo installed. If not, you can install them from [rustup.rs](https://rustup.rs/).
- Open your terminal and run the following command to install ACORN
    ```sh
    cargo install acorn-cli
    ```
- After installation, you can verify the installation by running:
    ```sh
    acorn help
    ```

### Download pre-compiled binary
- Download the latest release from the [ACORN GitHub Releases](https://code.ornl.gov/research-enablement/acorn/-/releases)
    - Linux
        ```sh
        curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-musl/v0.1.45/acorn
        ```
    - MacOS
        > 🚧 Under construction
    - Windows
        > 🚧 Under construction

### Install from source
- Clone the ACORN repository
    ```sh
    git clone https://code.ornl.gov/research-enablement/acorn.git
    cd acorn
    ```
- Install `acorn` command
    ```sh
    cargo install --path ./acorn-cli
    ```
- After installation, you can verify the installation by running:
    ```sh
    acorn help
    ```