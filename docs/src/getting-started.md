# 🚀 Getting Started

> [!IMPORTANT]
> The most popular way to use **ACORN** is to not install it at all --- simply leverage the power and promise of **ACORN** within a <span title="Continuous Integration">CI</span> job (see this [example `.gitlab-ci.yml`](https://code.ornl.gov/research-enablement/templates/-/blob/main/bucket.gitlab-ci.yml))

## Installation

### Install with Scoop (Windows only)
> [!WARNING]  
> 🚧 Under construction ([Pull Request](https://github.com/ScoopInstaller/Extras/pull/16876)/[Issue](https://github.com/ScoopInstaller/Extras/issues/16896))

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

### Install with Homebrew (Linux and MacOS)
> [!WARNING]  
> 🚧 Under construction

### Install with Cargo
- Ensure you have Rust and Cargo installed. If not, you can install them using [rustup.rs](https://rustup.rs/).
- Open your terminal and run the following command to install ACORN
    ```sh
    cargo install acorn-cli
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
- Download the latest release from the [ACORN GitHub Releases](https://code.ornl.gov/research-enablement/acorn/-/releases)
    - 🪟 Windows
        - Open a PowerShell terminal and run:
            ```powershell
            irm -OutFile acorn.exe -Uri https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-pc-windows-gnu/v0.1.50/acorn.exe
            ```
        - Test the downloaded executable:
            ```powershell
            .\acorn.exe help
            ```
    - 🐧 Linux
        - Open a terminal and run:
            ```sh
            curl -LO https://code.ornl.gov/api/v4/projects/16689/packages/generic/x86_64-unknown-linux-musl/v0.1.45/acorn
            ```
        - Make the downloaded file executable:
            ```sh
            chmod +x acorn
            ```
        - Test the downloaded executable:
            ```sh
            acorn help
            ```
    - 🍎 MacOS
        > 🚧 Under construction
    
> [!TIP]
> Move the binary to a directory included in your system's `PATH` for system-wide access.

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

[^oci]: These instructions will work with any OCI-compliant container runtime, such as [Docker](https://www.docker.com/get-started) or [Podman](https://podman.io/).
