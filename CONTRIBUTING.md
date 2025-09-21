# Getting Started
- Read this guide (including the [Code of Conduct](CODE_OF_CONDUCT.md))
- Fork this repository and clone your fork locally
- [Setup your environment](#project-setup) to use development tasks (test, lint, etc...)
- Adhere to the [project standards](#project-standards)
- Write some code and stuff...
- Push your changes and create a pull request

## Code of Conduct
> Please read and adhere to the [code of conduct](CODE_OF_CONDUCT.md)

## Introduction
> First off, thank you for considering contributing to `ACORN`!

If you would like to make a feature request or enhancement suggestion, please open an issue.

If you would like to generously provide a pull request to correct a verified issue, please adhere to this project's [standards](#project-standards). Before making a pull request for a desired feature, please socialize the concept by opening an issue first.

## Project Architecture
> See [ARCHITECTURE.md](./ARCHITECTURE.md)

## Project Setup
- When on Windows, execute `./setup.bat` to install and configure OpenSSL before trying to use any [development task](#development-tasks)

### Development Tasks
| Purpose                                  | Command            |
| ---------------------------------------: | ------------------ |
| Clean code artifacts                     | `make clean`       |
| Build code artifacts                     | `make build`       |
| Lint **ALL** code                        | `make lint`        |
| Lint **ALL** code and run **ALL** tests  | `make lint test`   |
| Run **ALL** tests                        | `make test`        |
| Run **ALL** tests with coverage          | `make coverage`    |
| Run security audit on dependencies       | `make audit`       |
| Generate documentation                   | `make doc`         |
| Generate CHANGELOG<sup>[[1]](#1)</sup>   | `make changelog`   |

### Visual Studio Code Configuration
**General Development**
- Install [VSCode](https://code.visualstudio.com/)
- Install [rust-analyzer VSCode extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)..they are all really handy and highly recommended 🤓

**Linux Development within Docker Container**
- [Follow the instructions of this article](https://code.visualstudio.com/docs/remote/containers)

## Project Standards
- Running `make lint` should not introduce any new lint and all format results should be included in proposed changes
- Running `make test` should not have any failures
- Running `make coverage` should not decrease coverage
- Exceptions to any of these standards should be supported by strong reasoning and sufficient effort


-------------

**Footnotes**
-------------

[1]
---
> CHANGELOG generation requires settting `GITLAB_TOKEN` in an `.env` file and installing [typos](https://github.com/crate-ci/typos)
