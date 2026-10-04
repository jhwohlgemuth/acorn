# Homebrew Custom Tap Helpers

This directory contains utilities for publishing `acorn-cli` to a custom Homebrew tap.

## Generate a formula locally

```sh
./scripts/homebrew/generate-formula.sh 0.1.56 ./Formula/acorn-cli.rb
```

Inputs:
- `version`: crate version without a `v` prefix
- `output-path` (optional): formula file path (defaults to `Formula/acorn-cli.rb`)

The script downloads the crate artifact from crates.io, computes `sha256`, and writes a formula from `formula.template.rb`.

## CI publishing flow

The `publish-homebrew-formula` job in `.gitlab-ci.yml` runs on version tags and pushes the rendered formula to your tap repository.

The `test-homebrew-macos` job runs after the tagged crate is published. It generates the formula, installs it in an isolated Tart VM, runs `brew test`, and checks the installed `acorn --version`. Relevant merge requests can start the same job manually against the version already published on crates.io.

Required CI variable:
- `HOMEBREW_TAP_ACCESS_TOKEN`: masked project access token created in the tap repository with the `write_repository` scope and a role permitted to push to the target branch. The job falls back to the legacy `ACCESS_TOKEN` variable when this is unset. If the selected variable is protected, version tags must also be protected so tagged pipelines can access it.

Optional CI variables:
- `HOMEBREW_TAP_REPO_URL` (default: `https://code.ornl.gov/research-enablement/homebrew-acorn.git`)
- `HOMEBREW_TAP_BRANCH` (default: `main`)
- `HOMEBREW_TAP_FORMULA_PATH` (default: `Formula/acorn-cli.rb`)
- `HOMEBREW_TAP_GIT_USER` (default: `ACORN CI`)
- `HOMEBREW_TAP_GIT_EMAIL` (default: `${GITLAB_USER_EMAIL}`)
