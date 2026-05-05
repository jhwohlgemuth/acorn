# 💼 Software Portability

Software portability describes how easily software can move across platforms and run in different environments with little or no modification. In ASPECT, this attribute helps measure whether software can work across different hardware, operating systems, and cloud setups.

## Levels of Portability
### Level 0: Limited
Software at this level is tied to a specific environment. Running it elsewhere often requires major code or configuration changes. It may depend on platform-specific hardware, operating systems, or libraries that are hard to reproduce.

### Level 1: Source
Software at this level can be compiled and run on different platforms when the required compiler and build tools are available. In most cases, only minor changes are needed. Some platform-specific dependencies may still need attention during build or setup.

### Level 2: Containerized
Software at this level is packaged and run in containers such as [Docker](https://www.docker.com/), [Podman](https://podman.io/), or [Apptainer](https://apptainer.org/). Containers isolate dependencies and provide a consistent runtime. As a result, deployment and scaling are easier across local, on-prem, and cloud environments when a compatible container runtime is available.

### Level 3: Installable
Software at this level is designed to be installed and configured on multiple platforms, usually through installers or setup scripts. Some manual setup may still be required, but the process is generally user-friendly. Platform-specific installers or options are often provided to support different operating systems and hardware.

### Level 4: Package manager
Software at this level is distributed through package managers such as [npm](https://www.npmjs.com/), [Homebrew](https://brew.sh/), [Scoop](https://scoop.sh/), or `apt`. It is typically published to package repositories so users can install and update it with standard tools. This approach improves deployment speed and ongoing maintenance across many environments.

### Level 5: WebAssembly
Software at this level is compiled to WebAssembly (WASM), which allows it to run in modern browsers and other environments that support a WebAssembly runtime. It can often run across desktop, mobile, and server targets without platform-specific changes. Because runtime support is broad and growing, WebAssembly is often treated as a leading portability target.
