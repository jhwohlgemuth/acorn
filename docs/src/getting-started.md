# 🚀 Getting Started

> [!IMPORTANT]
> The most popular way to use **ACORN** is to not install it at all --- simply leverage the power and promise of **ACORN** within a <span title="Continuous Integration">CI</span> job (see this [example `.gitlab-ci.yml`](https://code.ornl.gov/research-enablement/templates/-/blob/main/bucket.gitlab-ci.yml))

## Installation

### Install with Scoop (Windows only)
- Ensure you have [Scoop](https://scoop.sh/) installed.
- Open your terminal and add the extras bucket to Scoop
    ```sh
    scoop bucket add extras
    ```
- Run the following command to install ACORN
    ```sh
    scoop install acorn
    ```
- After installation, you can verify the installation by running:
    ```sh
    acorn help
    ```
- You can keep ACORN up to date with Scoop by running:
    ```sh
    scoop update acorn
    ```

### Install with Homebrew (Linux and MacOS)
- Add the custom tap:
    ```sh
    brew tap research-enablement/acorn https://code.ornl.gov/research-enablement/homebrew-acorn.git
    ```
- Trust the custom tap:
    ```sh
    brew trust research-enablement/acorn
    ```
- Install ACORN:
    ```sh
    brew install acorn-cli
    ```
- Verify installation:
    ```sh
    acorn help
    ```
- Upgrade to the latest version:
    ```sh
    brew update
    brew upgrade acorn-cli
    ```
### Install with Cargo
- Ensure you have Rust and Cargo installed. If not, you can install them using [rustup.rs](https://rustup.rs/).
- Open your terminal and run the following command to install ACORN
    ```sh
    cargo install --locked acorn-cli
    ```
- After installation, you can verify the installation by running:
    ```sh
    acorn help
    ```

### Run Inside a Container
- Ensure you have Docker[^oci] installed.
- Pull the latest ACORN image with Docker[^oci]
    ```sh
    docker pull savannah.ornl.gov/research-enablement/acorn/runner
    ```
- Run ACORN using Docker[^oci]
    ```sh
    docker run --rm savannah.ornl.gov/research-enablement/acorn/runner help
    ```

> [!TIP]
> You will probably need to mount a volume to use ACORN with your local files. For example:
> ```sh
> docker run --rm -v $(pwd):/data savannah.ornl.gov/research-enablement/acorn/runner:latest check /data/project
> ```

### Download pre-compiled binary
- Download the latest release from the [ACORN GitLab Releases page](https://code.ornl.gov/research-enablement/acorn/-/releases)
    - 🪟 Windows
        - Open a PowerShell terminal, visit the releases page to find the latest version, and run:
            ```powershell
            irm -OutFile acorn.exe -Uri https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-pc-windows-gnu/v<version>/acorn.exe
            ```
        - Test the downloaded executable:
            ```powershell
            .\acorn.exe help
            ```
    - 🐧 Linux
        - Open a terminal, visit the releases page to find the latest version, and run:
            ```sh
            curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-musl/v<version>/acorn
            ```
        - Make the downloaded file executable:
            ```sh
            chmod +x acorn
            ```
        - Test the downloaded executable:
            ```sh
            acorn help
            ```
    - 🍎 macOS
        - Standalone macOS binaries are not currently published. Use the [Homebrew](#install-with-homebrew-linux-and-macos), [Cargo](#install-with-cargo), or [source](#install-from-source) installation above.

### Install from source
- Clone the ACORN repository
    ```sh
    git clone https://code.ornl.gov/research-enablement/acorn.git
    cd acorn
    ```
- Install `acorn` command
    ```sh
    cargo install --locked --path ./crates/acorn-cli
    ```
- After installation, you can verify the installation by running:
    ```sh
    acorn help
    ```

[^oci]: These instructions will work with any OCI-compliant container runtime, such as [Docker](https://www.docker.com/get-started) or [Podman](https://podman.io/).
