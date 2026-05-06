# 💼 Software Portability

Software portability describes how easily software can move across platforms and run in different environments with little or no modification. In ASPECT, this attribute helps measure whether software can work across different hardware, operating systems, and cloud setups.

## Levels of Portability
### Limited (Level 0)
Software at this level is tied to a specific environment. Running it elsewhere often requires major code or configuration changes. It may depend on platform-specific hardware, operating systems, or libraries that are hard to reproduce.

### Source (Level 1)
Software at this level can be compiled and run on different platforms when the required compiler and build tools are available. In most cases, only minor changes are needed. Some platform-specific dependencies may still need attention during build or setup.

### Containerized (Level 2)
Software at this level is packaged and run in containers such as Docker[^docker], Podman[^podman], or Apptainer[^apptainer]. Containers isolate dependencies and provide a consistent runtime. As a result, deployment and scaling are easier across local, on-premise, and cloud environments when a compatible container runtime is available.

### Installable (Level 3)
Software at this level is designed to be installed and configured on multiple platforms, usually through installers or setup scripts. Some manual setup may still be required, but the process is generally user-friendly. Platform-specific installers or options are often provided to support different operating systems and hardware.

### Automated Installation Available (Level 4)
Software at this level is distributed through one or more package managers such as npm[^npm], Homebrew[^homebrew], Scoop[^scoop], or the Ubuntu advanced package management tool[^apt]. It is typically published to package repositories so users can install and update it with standard tools. This approach improves deployment speed and ongoing maintenance across many environments.

### WebAssembly[^wasm] (Level 5)
Software at this level is compiled to WebAssembly (WASM)[^compile], which allows it to run in modern browsers and other environments that support a WebAssembly runtime. It can often run across desktop, mobile, and server targets without platform-specific changes. Because runtime support is broad and growing, WebAssembly is often treated as a leading portability target.

[^apptainer]: [https://apptainer.org/](https://apptainer.org/)
[^apt]: [https://ubuntu.com/server/docs/how-to/software/package-management/](https://ubuntu.com/server/docs/how-to/software/package-management/)
[^compile]: [https://webassembly.org/getting-started/developers-guide/](https://webassembly.org/getting-started/developers-guide/)
[^docker]: [https://www.docker.com/](https://www.docker.com/)
[^homebrew]: [https://brew.sh/](https://brew.sh/)
[^npm]: [https://www.npmjs.com/](https://www.npmjs.com/)
[^podman]: [https://podman.io/](https://podman.io/)
[^scoop]: [https://scoop.sh/](https://scoop.sh/)
[^wasm]: [https://webassembly.org/](https://webassembly.org/)
