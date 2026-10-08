#!make
-include .env
ACP_AGENT ?= opencode

setup: ## Install the Rust toolchain and project development dependencies.
	@rustup update stable
	@cargo install $(DEV_DEPENDENCIES) --locked
	@rustup target add $(TARGETS)

help: ## Show the available development tasks.
	@awk 'BEGIN {FS = ":.*## "; printf "Usage: make <target> [VERSION=x.y.z] [ACP_AGENT=opencode|codex] [BENCH_ARGS=\"...\"] [BENCH_BASELINE=x.y.z] [BENCH_HISTORY=...] [WASM_PACK_BUILD_ARGS=\"...\"] [FUZZ_CRASH=path]\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-22s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

audit: ## Run dependency and security audits for the project.
	@cargo +nightly udeps
	@cargo audit \
		--ignore RUSTSEC-2020-0071 \
		--ignore RUSTSEC-2023-0071 \
		--ignore RUSTSEC-2024-0388 \
		--ignore RUSTSEC-2024-0436 \
		--ignore RUSTSEC-2025-0056 \
		--ignore RUSTSEC-2025-0134 \
		--ignore RUSTSEC-2025-0141 \
		--ignore RUSTSEC-2026-0009 \
		--ignore RUSTSEC-2026-0037 \
		--ignore RUSTSEC-2026-0044 \
		--ignore RUSTSEC-2026-0045 \
		--ignore RUSTSEC-2026-0046 \
		--ignore RUSTSEC-2026-0047 \
		--ignore RUSTSEC-2026-0048 \
		--ignore RUSTSEC-2026-0049 \
		--ignore RUSTSEC-2026-0097 \
		--ignore RUSTSEC-2026-0098 \
		--ignore RUSTSEC-2026-0099 \
		--ignore RUSTSEC-2026-0104
# 	@cargo deny check

bench: lint ## Benchmark the library and related crates.
	@cargo bench --package acorn-analysis --benches $(BENCH_ARGS)
	@cargo bench --package acorn-lib --benches $(BENCH_ARGS)

bench-baseline: ## Save a benchmark baseline for future comparisons.
	@$(MAKE) bench BENCH_ARGS="-- --save-baseline $(BENCH_BASELINE)"

bench-compare: ## Compare current benchmarks against the saved baseline.
	@$(MAKE) bench BENCH_ARGS="-- --baseline $(BENCH_BASELINE)"

bench-report: ## Build a benchmark summary report from stored results.
	@cargo xtask benchmark-history build "$(ARTIFACT_TARGET_DIR)/criterion" "$(BENCH_OUTPUT)" $(if $(BENCH_HISTORY),"$(BENCH_HISTORY)")

book: ## Build the documentation site with mdBook.
	@mdbook clean docs
	@mdbook build docs

book-write: ## Serve the documentation site locally for preview.
	@mdbook serve --open docs

build: ## Build release binaries for the CLI.
ifeq ($(OS),Windows_NT)
	@$(CLEAN_BIN_CMD)
	@pwsh -Command New-Item -ItemType Directory -Path ".\bin" -Force
	@cargo build --release --package acorn-cli --target $(DEFAULT_WINDOWS_TARGET)
	@pwsh -Command Move-Item -Path "$(ARTIFACT_TARGET_DIR)/$(DEFAULT_WINDOWS_TARGET)/release/acorn.exe" -Destination "./bin/acorn-v${VERSION}.exe" -Force
else
	@$(CLEAN_BIN_CMD)
	@mkdir -p ./bin
	@$(MUSL_BUILD) --release --package acorn-cli --target $(DEFAULT_TARGET)
	@mv $(ARTIFACT_TARGET_DIR)/$(DEFAULT_TARGET)/release/acorn ./bin/acorn-v$(VERSION)
	@cargo build --release --package acorn-cli --target $(DEFAULT_WINDOWS_TARGET)
	@mv $(ARTIFACT_TARGET_DIR)/$(DEFAULT_WINDOWS_TARGET)/release/acorn.exe ./bin/acorn-v$(VERSION).exe
endif

build-in-image: clean ## Build the project in a Docker image environment.
	@docker image inspect $(REGISTRY)/acorn/rust:$(VERSION) > /dev/null 2>&1 || $(MAKE) rust-image
	@MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' docker run --rm -e CARGO_TARGET_DIR=/app/target/acorn -v "$(CURDIR):/app" -v acorn-target:/app/target -w /app $(REGISTRY)/acorn/rust:$(VERSION) make build

mcpb: ## Build the MCP bundle artifacts for release packaging.
	@./bin/acorn-v$(VERSION) create mcp --binary ./bin/acorn-v$(VERSION) --platform linux --output ./bin/acorn-v$(VERSION)-x86_64-unknown-linux-musl.mcpb --clobber
	@./bin/acorn-v$(VERSION) create mcp --binary ./bin/acorn-v$(VERSION).exe --platform win32 --output ./bin/acorn-v$(VERSION)-x86_64-pc-windows-gnu.mcpb --clobber
	@$(call WRITE_SHA256,./bin/acorn-v$(VERSION)-x86_64-unknown-linux-musl.mcpb)
	@$(call WRITE_SHA256,./bin/acorn-v$(VERSION)-x86_64-pc-windows-gnu.mcpb)

release-checksums: ## Generate SHA256 checksums for release artifacts.
	@$(call WRITE_SHA256,./bin/acorn-v$(VERSION))
	@$(call WRITE_SHA256,./bin/acorn-v$(VERSION).exe)

build-wasm: ## Build and test the WebAssembly package.
	@wasm-pack build $(WASM_PACK_BUILD_ARGS) crates/acorn-wasm --target web --out-dir ../../target/acorn-wasm/pkg
	@wasm-pack test --node crates/acorn-wasm

clean: ## Remove build artifacts and generated binaries.
	@cargo clean
	@$(CLEAN_BIN_CMD)

changelog: ## Generate the changelog from git tags and metadata.
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN}

changelog-release: ## Generate the latest release changelog entry.
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN} --latest

check: check-dependencies check-portable check-host ## Run the full validation suite for the project.

check-dependencies: ## Validate workspace dependency declarations and source linkage.
	@actual="$$(cargo-shear --version 2>/dev/null || echo 'not installed')"; \
		expected="Version: $(CARGO_SHEAR_VERSION)"; \
		test "$$actual" = "$$expected" || { \
			echo "cargo-shear $(CARGO_SHEAR_VERSION) is required ($$actual). Run: cargo install --force --locked cargo-shear@$(CARGO_SHEAR_VERSION)" >&2; \
			exit 1; \
		}
	@cargo-shear --locked --offline --deny-warnings

check-host: ## Validate host-side feature builds and dependency checks.
	@cargo check -p acorn-cmd --no-default-features
	@cargo check -p acorn-cmd --features shell-lint
	@cargo check -p acorn-host --no-default-features --features fs
	@cargo check -p acorn-host --no-default-features --features config
	@cargo check -p acorn-host --no-default-features --features http
	@cargo check -p acorn-host --no-default-features --features download
	@cargo check -p acorn-host --no-default-features --features source
	@cargo check -p acorn-host --no-default-features --features terminal
	@cargo check --manifest-path tests/downstream-host/Cargo.toml
	@bash scripts/check-host-dependencies.sh
	@cargo check -p acorn-lib
	@cargo check -p acorn-lib --no-default-features --features std
	@cargo check -p acorn-lib --no-default-features --features acp
	@cargo check -p acorn-lib --no-default-features --features analysis
	@cargo check -p acorn-lib --no-default-features --features dataframe
	@cargo check -p acorn-lib --no-default-features --features diagnostics
	@cargo check -p acorn-lib --no-default-features --features exec
	@cargo check -p acorn-lib --no-default-features --features acp,analysis
	@cargo check -p acorn-lib --no-default-features --features acp,mcp-client
	@cargo check -p acorn-lib --no-default-features --features agentic,acp,analysis
	@cargo check -p acorn-lib --no-default-features --features mcp-client
	@cargo check -p acorn-lib --no-default-features --features agentic,mcp-client
	@cargo check -p acorn-lib --features std,analysis
	@cargo check -p acorn-cli

check-macos-unwind: ## Measure and enforce Darwin debug CLI and test unwind-section limits.
	@bash scripts/check-macos-unwind.sh

check-portable: ## Validate portable/core feature builds and compatibility checks.
	@cargo check -p acorn-diff
	@cargo check -p acorn-diff --no-default-features --features alloc
	@cargo check -p acorn-core
	@cargo check -p acorn-core --no-default-features --features core
	@cargo check -p acorn-core --no-default-features --features alloc
	@cargo check -p acorn-core --no-default-features
	@cargo check -p acorn-schema
	@cargo check -p acorn-schema --no-default-features --features core
	@cargo check -p acorn-schema --no-default-features --features alloc
	@cargo check -p acorn-schema --no-default-features
	@cargo test -p acorn-schema --test portable_semantics
	@cargo test -p acorn-schema --no-default-features --features alloc --test portable_semantics
	@cargo check -p acorn-analysis
	@cargo check -p acorn-analysis --no-default-features --features alloc
	@cargo check -p acorn-service
	@cargo check -p acorn-service --no-default-features --features alloc
	@cargo test -p acorn-service --no-default-features --features alloc
	@cargo check -p acorn-lib --no-default-features --features core
	@cargo check -p acorn-lib --no-default-features --features alloc
	@cargo check -p acorn-lib --no-default-features
	@$(MAKE) check-wasm
	@$(MAKE) build-wasm WASM_PACK_BUILD_ARGS=--dev
	@bash scripts/check-portable-dependencies.sh

check-wasm: ## Validate the WebAssembly target compatibility of the workspace.
	@cargo check -p acorn-diff --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-core --no-default-features --features core --target wasm32-unknown-unknown
	@cargo check -p acorn-core --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features core --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features wasm --target wasm32-unknown-unknown
	@cargo check -p acorn-analysis --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-service --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown
	@cargo check -p acorn-wasm --target wasm32-unknown-unknown

cover: ## Run coverage analysis with tarpaulin.
	@cargo tarpaulin $(COVER_JOBS) --out lcov --line --force-clean --no-dead-code --engine llvm --profile coverage --locked

cover-cli: ## Run scoped coverage analysis for acorn-cli.
	@cargo tarpaulin $(COVER_JOBS) -p acorn-cli --out lcov --line --force-clean --no-dead-code --engine llvm --profile coverage --locked

cover-lib: ## Run scoped coverage analysis for acorn-lib.
	@cargo tarpaulin $(COVER_JOBS) -p acorn-lib --out lcov --line --force-clean --no-dead-code --engine llvm --profile coverage --locked

doc: ## Build the Rust documentation for the workspace.
	@cargo clean --doc
	@cargo doc --no-deps --all-features

doctest: ## Run the project doctests.
	@cargo test --doc

lint: protolint shellcheck sqlcheck ## Format and lint the entire workspace.
	@cargo fmt --all
	@cargo clippy --fix --allow-dirty --workspace --no-deps --all-features --all-targets -- -D warnings

protolint: ## Lint protobuf definitions with buf.
	@command -v buf >/dev/null 2>&1 || (echo "buf not found; install it from https://buf.build/docs/cli/installation/" >&2; exit 1)
	@buf lint

shellcheck: ## Run shellcheck across the project scripts.
	@if command -v shellcheck >/dev/null 2>&1; then \
		for file in $$(find scripts -type f -name '*.sh'); do tr -d '\r' < "$$file" > "$$file.tmp" && mv "$$file.tmp" "$$file"; done; \
		find scripts -type f -name '*.sh' -print0 | sort -z | xargs -0 shellcheck; \
	else echo "shellcheck not found, skipping"; fi

sqlcheck: ## Run SQL linting and formatting for migrations.
	@if command -v sqruff >/dev/null 2>&1; then \
		sqruff fix crates/acorn-lib/assets/migrations && sqruff lint crates/acorn-lib/assets/migrations; \
	else echo "sqruff not found, skipping"; fi

publish-book: book ## Publish the built docs site to the configured destination.
	@git clone "https://user:${ACCESS_TOKEN}@code-int.ornl.gov/sites/acorn.git" site
	@rm -rf ./site/html
	@mv docs/book ./site/html
	@cd ./site && git add html && (git diff --cached --quiet || (git commit -m "Publish ACORN Book" && git push))

test: ## Run the main test suite with nextest.
	@cargo nextest run

test-all: ## Run the full suite including ignored tests.
	@cargo nextest run --run-ignored all

test-acp: ## Run interoperability tests for ACP_AGENT=opencode|codex.
	@case "$(ACP_AGENT)" in codex|opencode) ;; *) echo "Unsupported ACP_AGENT '$(ACP_AGENT)'; expected codex or opencode"; exit 2;; esac
	@if [ "$(ACP_AGENT)" = codex ] && ! command -v codex-acp >/dev/null 2>&1; then \
		echo "codex-acp is required for ACP_AGENT=codex but was not found on PATH."; \
		echo "Install it with: npm install -g @agentclientprotocol/codex-acp"; \
		echo "The global install does not require npx. If npm is unavailable, install Node.js/npm or download a standalone binary:"; \
		echo "https://github.com/agentclientprotocol/codex-acp/releases"; \
		exit 2; \
	fi
	@ACORN_TEST_ACP_AGENT="$(ACP_AGENT)" cargo nextest run -p acorn-lib --test acp_interoperability --no-default-features --features acp,analysis --run-ignored only --success-output final

test-ci: lint ## Run the CI-style test workflow.
	@XDG_CACHE_HOME=$(CURDIR)/.cache cargo test --lib --bins --tests
	@XDG_CACHE_HOME=$(CURDIR)/.cache cargo test --lib --bins --tests --no-default-features

test-openai: ## Run configured OpenAI-compatible interoperability tests.
	@test -n "$(TEST_OPENAI_BASE_URL)" || (echo "TEST_OPENAI_BASE_URL must be set in .env or the make environment"; exit 2)
	@test -n "$(TEST_OPENAI_MODEL)" || (echo "TEST_OPENAI_MODEL must be set in .env or the make environment"; exit 2)
	@ACORN_TEST_OPENAI_COMPATIBLE=1 ACORN_TEST_OPENAI_BASE_URL="$(TEST_OPENAI_BASE_URL)" ACORN_TEST_OPENAI_MODEL="$(TEST_OPENAI_MODEL)" OPENAI_API_KEY="$(TEST_OPENAI_API_KEY)" cargo nextest run -p acorn-lib --test openai_interoperability --no-default-features --features analysis --run-ignored only --success-output final

fuzz: ## Run the configured fuzz target.
	@cargo +nightly fuzz run --fuzz-dir fuzz $(FUZZ_TARGET) $(FUZZ_ARGS)

fuzz-repro: ## Reproduce a specific fuzz crash from an artifact path.
	@test -n "$(FUZZ_CRASH)" || (echo "Set FUZZ_CRASH to an artifact file path" && exit 1)
	@cargo +nightly fuzz run --fuzz-dir fuzz $(FUZZ_TARGET) $(FUZZ_CRASH)

fuzz-smoke: ## Run lightweight fuzz smoke checks for the project.
	@rm -rf fuzz/artifacts
	@sh -c 'if command -v timeout >/dev/null 2>&1; then timeout --foreground --kill-after=30 180 cargo +nightly fuzz run --fuzz-dir fuzz location_repository fuzz/corpus/location_repository -- -max_total_time=5 -rss_limit_mb=2048 -timeout=5 -print_final_stats=1; else cargo +nightly fuzz run --fuzz-dir fuzz location_repository fuzz/corpus/location_repository -- -max_total_time=5 -rss_limit_mb=2048 -timeout=5 -print_final_stats=1; fi'
	@sh -c 'if command -v timeout >/dev/null 2>&1; then timeout --foreground --kill-after=30 180 cargo +nightly fuzz run --fuzz-dir fuzz research_activity fuzz/corpus/research_activity -- -max_total_time=5 -rss_limit_mb=2048 -timeout=5 -print_final_stats=1; else cargo +nightly fuzz run --fuzz-dir fuzz research_activity fuzz/corpus/research_activity -- -max_total_time=5 -rss_limit_mb=2048 -timeout=5 -print_final_stats=1; fi'

help-cli: ## Regenerate the ACORN CLI markdown help reference.
	@cargo run --bin acorn -- --markdown-help | awk 'length($$0) > 0 { while (blank > 0) { print ""; blank-- } print; next } { blank++ }' > CommandLineHelp.md
	@printf "Updated CommandLineHelp.md with the current ACORN CLI help.\n"

homebrew-formula: ## Generate the Homebrew formula for the CLI.
	@./scripts/homebrew/generate-formula.sh $(VERSION) ./Formula/acorn-cli.rb

version: ## Print the project version string.
	@echo $(VERSION)

rust-version: ## Print the Rust toolchain version string.
	@echo $(RUST_VERSION)

check-version: ## Validate the workspace version metadata.
	@cargo xtask check-version

release-version: ## Bump the release version and verify it.
	@cargo xtask release-version $(VERSION)
	@$(MAKE) check-version

acorn-clean: ## Remove ACORN Docker images from the local registry.
	@docker rmi -f acorn:$(VERSION)
	@docker rmi -f acorn:latest
	@docker rmi -f $(REGISTRY)/acorn/runner:$(VERSION)
	@docker rmi -f $(REGISTRY)/acorn/runner:latest
	@docker rmi -f jhwohlgemuth/acorn:$(VERSION)
	@docker rmi -f jhwohlgemuth/acorn:latest
	@docker rmi -f ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker rmi -f ghcr.io/jhwohlgemuth/acorn:latest

acorn-image: ## Build the ACORN Docker image for release or local use.
	@docker build --no-cache --platform $(PLATFORM) --build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) --build-arg RUST_VERSION=$(RUST_VERSION) -t acorn:$(VERSION) -f Dockerfile .
	@docker tag acorn:$(VERSION) acorn:latest
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:$(VERSION)
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:latest
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:latest
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:latest

acorn-push: acorn-push-savannah acorn-push-docker acorn-push-ghcr ## Push the ACORN images to all configured registries.

acorn-push-savannah: ## Push ACORN images to the Savannah registry.
	@docker push $(REGISTRY)/acorn/runner:$(VERSION)
	@docker push $(REGISTRY)/acorn/runner:latest

acorn-push-docker: ## Push ACORN images to Docker Hub.
	@docker push jhwohlgemuth/acorn:$(VERSION)
	@docker push jhwohlgemuth/acorn:latest

acorn-push-ghcr: ## Push ACORN images to GitHub Container Registry.
	@docker push ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker push ghcr.io/jhwohlgemuth/acorn:latest

bucket-clean: ## Remove bucket Docker images from the local registry.
	@docker rmi -f bucket:$(VERSION)
	@docker rmi -f bucket:latest
	@docker rmi -f $(REGISTRY)/bucket/runner:$(VERSION)
	@docker rmi -f $(REGISTRY)/bucket/runner:latest
	@docker rmi -f jhwohlgemuth/bucket:$(VERSION)
	@docker rmi -f jhwohlgemuth/bucket:latest
	@docker rmi -f ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker rmi -f ghcr.io/jhwohlgemuth/bucket:latest

bucket-image: ## Build the bucket Docker image for deployment.
	@docker build --no-cache \
		--platform $(PLATFORM) \
		--build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) \
		--build-arg RUST_VERSION=$(RUST_VERSION) \
		--build-arg ACORN_VERSION=$(VERSION) \
		--build-arg DEFAULT_TARGET=$(DEFAULT_TARGET) \
		-t bucket:$(VERSION) \
		-f Dockerfile.bucket .
	@docker tag bucket:$(VERSION) bucket:latest
	@docker tag bucket:$(VERSION) $(REGISTRY)/bucket/runner:$(VERSION)
	@docker tag bucket:$(VERSION) $(REGISTRY)/bucket/runner:latest
	@docker tag bucket:$(VERSION) jhwohlgemuth/bucket:$(VERSION)
	@docker tag bucket:$(VERSION) jhwohlgemuth/bucket:latest
	@docker tag bucket:$(VERSION) ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker tag bucket:$(VERSION) ghcr.io/jhwohlgemuth/bucket:latest

bucket-push: bucket-push-savannah bucket-push-docker bucket-push-ghcr ## Push the bucket images to all configured registries.

bucket-push-savannah: ## Push bucket images to the Savannah registry.
	@docker push $(REGISTRY)/bucket/runner:$(VERSION)
	@docker push $(REGISTRY)/bucket/runner:latest

bucket-push-docker: ## Push bucket images to Docker Hub.
	@docker push jhwohlgemuth/bucket:$(VERSION)
	@docker push jhwohlgemuth/bucket:latest

bucket-push-ghcr: ## Push bucket images to GitHub Container Registry.
	@docker push ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker push ghcr.io/jhwohlgemuth/bucket:latest

rust-clean: ## Remove Rust toolchain Docker images from the local registry.
	@docker rmi -f $(REGISTRY)/acorn/rust:$(VERSION)
	@docker rmi -f $(RUST_IMAGE_AMD64)
	@docker rmi -f $(RUST_IMAGE_ARM64)
	@docker rmi -f $(REGISTRY)/acorn/rust:latest

rust-image: rust-image-amd64 rust-image-arm64 ## Build the Rust toolchain images used for CI and container builds.

rust-image-amd64: ## Build the AMD64 Rust toolchain image on its native builder.
	@docker --context $(RUST_AMD64_CONTEXT) build --no-cache --platform linux/amd64 --build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) --build-arg RUST_VERSION=$(RUST_VERSION) -t $(RUST_IMAGE_AMD64) -t $(RUST_IMAGE_GHCR_AMD64) -f Dockerfile.rust .

rust-image-arm64: ## Build the ARM64 Rust toolchain image on its native builder.
	@docker --context $(RUST_ARM64_CONTEXT) build --no-cache --platform linux/arm64 --build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) --build-arg RUST_VERSION=$(RUST_VERSION) -t $(RUST_IMAGE_ARM64) -t $(RUST_IMAGE_GHCR_ARM64) -f Dockerfile.rust .

rust-publish-ghcr: rust-image ## Build and publish the multi-platform Rust toolchain image to GitHub Container Registry.
	@$(MAKE) rust-push-ghcr

rust-push: rust-push-savannah rust-push-ghcr ## Push the Rust toolchain images to all configured registries.

rust-push-ghcr: ## Push Rust toolchain images to GitHub Container Registry.
	@docker --context $(RUST_AMD64_CONTEXT) push $(RUST_IMAGE_GHCR_AMD64)
	@docker --context $(RUST_ARM64_CONTEXT) push $(RUST_IMAGE_GHCR_ARM64)
	@docker buildx imagetools create --tag $(RUST_IMAGE_GHCR_VERSION) --tag $(RUST_IMAGE_GHCR_LATEST) $(RUST_IMAGE_GHCR_AMD64) $(RUST_IMAGE_GHCR_ARM64)

rust-push-savannah: ## Push Rust toolchain images to the Savannah registry.
	@docker --context $(RUST_AMD64_CONTEXT) push $(RUST_IMAGE_AMD64)
	@docker --context $(RUST_ARM64_CONTEXT) push $(RUST_IMAGE_ARM64)
	@docker buildx imagetools create --tag $(REGISTRY)/acorn/rust:$(VERSION) --tag $(REGISTRY)/acorn/rust:latest $(RUST_IMAGE_AMD64) $(RUST_IMAGE_ARM64)
#
# Variables
#
ifeq ($(OS),Windows_NT)
CLEAN_BIN_CMD = pwsh -NoProfile -Command "if (Test-Path ./bin) { Remove-Item -Recurse -Force ./bin }"
COVER_JOBS ?= --jobs 1
WRITE_SHA256 = pwsh -NoProfile -Command "(Get-FileHash -Algorithm SHA256 -LiteralPath '$(1)').Hash.ToLowerInvariant() + '  $(1)' | Set-Content -Encoding ascii -LiteralPath '$(1).sha256'"
else
CLEAN_BIN_CMD = rm -rf ./bin
COVER_JOBS ?=
WRITE_SHA256 = if command -v sha256sum >/dev/null 2>&1; then sha256sum "$(1)" > "$(1).sha256"; elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$(1)" > "$(1).sha256"; else echo "A SHA-256 utility (sha256sum or shasum) is required" >&2; exit 1; fi
endif
PLATFORM = linux/amd64
VERSION = `awk -F ' = ' '$$1 ~ /^version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
RUST_VERSION = `awk -F ' = ' '$$1 ~ /rust-version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
RUST_AMD64_CONTEXT ?= oklahoma
RUST_ARM64_CONTEXT ?= dante
RUST_IMAGE_AMD64 = $(REGISTRY)/acorn/rust:$(VERSION)-amd64
RUST_IMAGE_ARM64 = $(REGISTRY)/acorn/rust:$(VERSION)-arm64
RUST_IMAGE_GHCR = ghcr.io/jhwohlgemuth/acorn
RUST_IMAGE_GHCR_AMD64 = $(RUST_IMAGE_GHCR):rust-$(VERSION)-amd64
RUST_IMAGE_GHCR_ARM64 = $(RUST_IMAGE_GHCR):rust-$(VERSION)-arm64
RUST_IMAGE_GHCR_LATEST = $(RUST_IMAGE_GHCR):rust-latest
RUST_IMAGE_GHCR_VERSION = $(RUST_IMAGE_GHCR):rust-$(VERSION)
DOCKER_HUB_REGISTRY = mirror.gcr.io/library
ORNL_HUB_PROXY = regproxy.ornl.gov/hub_proxy
REGISTRY = savannah.ornl.gov/research-enablement
DEFAULT_TARGET = `awk -F ' = ' '$$1 ~ /default-target/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml | tr . _`
DEFAULT_WINDOWS_TARGET = x86_64-pc-windows-gnu
MUSL_BUILD ?= cargo build
ARTIFACT_TARGET_DIR = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
DEV_DEPENDENCIES = \
	cargo-audit \
	cargo-auditable \
	cargo-deny \
	cargo-fuzz \
	cargo-insta \
	cargo-msrv \
	cargo-nextest \
	cargo-shear@$(CARGO_SHEAR_VERSION) \
	cargo-tarpaulin \
	cargo-udeps \
	mdbook \
	mdbook-mermaid \
	mdbook-pikchr \
	sqruff \
	wasm-pack
TARGETS = \
	i586-unknown-linux-musl \
	i686-unknown-linux-gnu \
	aarch64-unknown-linux-musl \
	arm-unknown-linux-gnueabi \
	arm-unknown-linux-musleabi \
	riscv64gc-unknown-linux-gnu \
	x86_64-unknown-linux-gnu \
	x86_64-unknown-linux-musl \
	x86_64-pc-windows-gnu \
	wasm32-unknown-unknown
FUZZ_TARGET = research_activity
FUZZ_ARGS =
BENCH_BASELINE = $(VERSION)
BENCH_OUTPUT = $(ARTIFACT_TARGET_DIR)/benchmark-results
BENCH_HISTORY =
CARGO_SHEAR_VERSION = 1.14.0
WASM_PACK_BUILD_ARGS =
TASKS = \
	audit \
	book \
	build \
	build-wasm \
	mcpb \
	release-checksums \
	changelog \
	changelog-release \
	clean \
	cover \
	doc \
	bench \
	bench-baseline \
	bench-compare \
	bench-report \
	lint \
	shellcheck \
	sqlcheck \
	fuzz \
	fuzz-repro \
	help \
	help-cli \
	homebrew-formula \
	publish-book \
	setup \
	test \
	test-acp \
	test-ci \
	test-openai \
	check-dependencies \
	check-wasm \
	version \
	release-version \
	check-version \
	acorn-clean \
	acorn-build-in-image \
	acorn-image \
	acorn-push \
	acorn-push-docker \
	acorn-push-ghcr \
	bucket-clean \
	bucket-image \
	bucket-push \
	bucket-push-docker \
	bucket-push-ghcr \
	rust-image \
	rust-image-amd64 \
	rust-image-arm64 \
	rust-publish-ghcr \
	rust-push \
	rust-push-ghcr \
	rust-push-savannah

.PHONY: $(TASKS)
