#!make
-include .env

setup:
	@rustup update stable
	@cargo install $(DEV_DEPENDENCIES) --locked
	@rustup target add $(TARGETS)

audit:
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

bench: lint
	@cargo bench --package acorn-lib --benches $(BENCH_ARGS)

bench-baseline:
	@$(MAKE) bench BENCH_ARGS="-- --save-baseline $(BENCH_BASELINE)"

bench-compare:
	@$(MAKE) bench BENCH_ARGS="-- --baseline $(BENCH_BASELINE)"

bench-report:
	@cargo xtask benchmark-history build "$(ARTIFACT_TARGET_DIR)/criterion" "$(BENCH_OUTPUT)" $(if $(BENCH_HISTORY),"$(BENCH_HISTORY)")

book:
	@mdbook clean docs
	@mdbook build docs

book-write:
	@mdbook serve --open docs

build:
ifeq ($(OS),Windows_NT)
	@$(CLEAN_BIN_CMD)
	@pwsh -Command New-Item -ItemType Directory -Path ".\bin" -Force
	@cargo build --release --package acorn-cli --target $(DEFAULT_WINDOWS_TARGET)
	@pwsh -Command Move-Item -Path "$(ARTIFACT_TARGET_DIR)/$(DEFAULT_WINDOWS_TARGET)/release/acorn.exe" -Destination "./bin/acorn-v${VERSION}.exe" -Force
else
	@$(CLEAN_BIN_CMD)
	@mkdir -p ./bin
	@cargo build --release --package acorn-cli --target $(DEFAULT_TARGET)
	@mv $(ARTIFACT_TARGET_DIR)/$(DEFAULT_TARGET)/release/acorn ./bin/acorn-v$(VERSION)
	@cargo build --release --package acorn-cli --target $(DEFAULT_WINDOWS_TARGET)
	@mv $(ARTIFACT_TARGET_DIR)/$(DEFAULT_WINDOWS_TARGET)/release/acorn.exe ./bin/acorn-v$(VERSION).exe
endif

build-in-image: clean
	@docker image inspect $(REGISTRY)/acorn/rust:$(VERSION) > /dev/null 2>&1 || $(MAKE) rust-image
	@MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' docker run --rm -e CARGO_TARGET_DIR=/app/target/acorn -v "$(CURDIR):/app" -v acorn-target:/app/target -w /app $(REGISTRY)/acorn/rust:$(VERSION) make build


clean:
	@cargo clean
	@$(CLEAN_BIN_CMD)

changelog:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN}

changelog-release:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN} --latest

check:
	@cargo check -p acorn-core
	@cargo check -p acorn-core --no-default-features --features core
	@cargo check -p acorn-core --no-default-features --features alloc
	@cargo check -p acorn-core --no-default-features
	@cargo check -p acorn-schema
	@cargo check -p acorn-schema --no-default-features --features core
	@cargo check -p acorn-schema --no-default-features --features alloc
	@cargo check -p acorn-schema --no-default-features
	@cargo check -p acorn-lib
	@cargo check -p acorn-lib --no-default-features --features core
	@cargo check -p acorn-lib --no-default-features --features alloc
	@cargo check -p acorn-lib --no-default-features
	@$(MAKE) wasm
	@cargo check -p acorn-lib --features std,analysis
	@cargo check -p acorn-cli

wasm:
	@cargo check -p acorn-core --no-default-features --features core --target wasm32-unknown-unknown
	@cargo check -p acorn-core --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features core --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features alloc --target wasm32-unknown-unknown
	@cargo check -p acorn-schema --no-default-features --features wasm --target wasm32-unknown-unknown
	@cargo check -p acorn-lib --no-default-features --features wasm --target wasm32-unknown-unknown

cover:
	@cargo tarpaulin --out lcov --line --skip-clean --no-dead-code --engine llvm --profile coverage --locked

doc:
	@cargo clean --doc
	@cargo doc --no-deps --all-features

doctest:
	@cargo test --doc

lint:
	@cargo fmt --check
	@cargo clippy --fix --allow-dirty --workspace --no-deps --all-features --all-targets -- -D warnings

publish-book: book
	@git clone "https://user:${ACCESS_TOKEN}@code-int.ornl.gov/sites/acorn.git" site
	@rm -rf ./site/html
	@mv docs/book ./site/html
	@cd ./site && git add html && (git diff --cached --quiet || (git commit -m "Publish ACORN Book" && git push))

test:
	@cargo nextest run

test-all:
	@cargo nextest run --run-ignored all

test-ci: lint
	@cargo test --lib --bins --tests
	@cargo test --lib --bins --tests --no-default-features

fuzz:
	@cargo +nightly fuzz run --fuzz-dir fuzz $(FUZZ_TARGET) $(FUZZ_ARGS)

fuzz-repro:
	@test -n "$(FUZZ_CRASH)" || (echo "Set FUZZ_CRASH to an artifact file path" && exit 1)
	@cargo +nightly fuzz run --fuzz-dir fuzz $(FUZZ_TARGET) $(FUZZ_CRASH)

fuzz-smoke:
	@cargo +nightly fuzz run --fuzz-dir fuzz location_repository fuzz/corpus/location_repository -- -max_total_time=10
	@cargo +nightly fuzz run --fuzz-dir fuzz research_activity fuzz/corpus/research_activity -- -max_total_time=10

help:
	@cargo run --bin acorn -- --markdown-help | awk 'length($$0) > 0 { while (blank > 0) { print ""; blank-- } print; next } { blank++ }' > CommandLineHelp.md

homebrew-formula:
	@./scripts/homebrew/generate-formula.sh $(VERSION) ./Formula/acorn-cli.rb

version: 
	@echo $(VERSION)

rust-version:
	@echo $(RUST_VERSION)

check-version:
	@cargo xtask check-version

release-version:
	@cargo xtask release-version $(VERSION)
	@$(MAKE) check-version

acorn-clean:
	@docker rmi -f acorn:$(VERSION)
	@docker rmi -f acorn:latest
	@docker rmi -f $(REGISTRY)/acorn/runner:$(VERSION)
	@docker rmi -f $(REGISTRY)/acorn/runner:latest
	@docker rmi -f jhwohlgemuth/acorn:$(VERSION)
	@docker rmi -f jhwohlgemuth/acorn:latest
	@docker rmi -f ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker rmi -f ghcr.io/jhwohlgemuth/acorn:latest

acorn-image:
	@docker build --no-cache --platform $(PLATFORM) --build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) --build-arg RUST_VERSION=$(RUST_VERSION) -t acorn:$(VERSION) -f Dockerfile .
	@docker tag acorn:$(VERSION) acorn:latest
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:$(VERSION)
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:latest
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:latest
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:latest

acorn-push: acorn-push-savannah acorn-push-docker acorn-push-ghcr

acorn-push-savannah:
	@docker push $(REGISTRY)/acorn/runner:$(VERSION)
	@docker push $(REGISTRY)/acorn/runner:latest

acorn-push-docker:
	@docker push jhwohlgemuth/acorn:$(VERSION)
	@docker push jhwohlgemuth/acorn:latest

acorn-push-ghcr:
	@docker push ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker push ghcr.io/jhwohlgemuth/acorn:latest

bucket-clean:
	@docker rmi -f bucket:$(VERSION)
	@docker rmi -f bucket:latest
	@docker rmi -f $(REGISTRY)/bucket/runner:$(VERSION)
	@docker rmi -f $(REGISTRY)/bucket/runner:latest
	@docker rmi -f jhwohlgemuth/bucket:$(VERSION)
	@docker rmi -f jhwohlgemuth/bucket:latest
	@docker rmi -f ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker rmi -f ghcr.io/jhwohlgemuth/bucket:latest

bucket-image:
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

bucket-push: bucket-push-savannah bucket-push-docker bucket-push-ghcr

bucket-push-savannah:
	@docker push $(REGISTRY)/bucket/runner:$(VERSION)
	@docker push $(REGISTRY)/bucket/runner:latest

bucket-push-docker:
	@docker push jhwohlgemuth/bucket:$(VERSION)
	@docker push jhwohlgemuth/bucket:latest

bucket-push-ghcr:
	@docker push ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker push ghcr.io/jhwohlgemuth/bucket:latest

rust-clean:
	@docker rmi -f $(REGISTRY)/acorn/rust:$(VERSION)
	@docker rmi -f $(REGISTRY)/acorn/rust:latest

rust-image:
	@docker build --no-cache --platform $(PLATFORM) --build-arg DOCKER_HUB_REGISTRY=$(DOCKER_HUB_REGISTRY) --build-arg RUST_VERSION=$(RUST_VERSION) -t $(REGISTRY)/acorn/rust:$(VERSION) -f Dockerfile.rust .
	@docker tag $(REGISTRY)/acorn/rust:$(VERSION) $(REGISTRY)/acorn/rust:latest

rust-push:
	@docker push $(REGISTRY)/acorn/rust:$(VERSION)
	@docker push $(REGISTRY)/acorn/rust:latest
#
# Variables
#
ifeq ($(OS),Windows_NT)
CLEAN_BIN_CMD = pwsh -NoProfile -Command "if (Test-Path ./bin) { Remove-Item -Recurse -Force ./bin }"
else
CLEAN_BIN_CMD = rm -rf ./bin
endif
# PLATFORM = linux/amd64,linux/arm64
PLATFORM = linux/amd64
VERSION = `awk -F ' = ' '$$1 ~ /^version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
RUST_VERSION = `awk -F ' = ' '$$1 ~ /rust-version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
DOCKER_HUB_REGISTRY = mirror.gcr.io/library
ORNL_HUB_PROXY = regproxy.ornl.gov/hub_proxy
REGISTRY = savannah.ornl.gov/research-enablement
DEFAULT_TARGET = `awk -F ' = ' '$$1 ~ /default-target/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml | tr . _`
DEFAULT_WINDOWS_TARGET = x86_64-pc-windows-gnu
ARTIFACT_TARGET_DIR = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
DEV_DEPENDENCIES = \
	cargo-fuzz \
	cargo-audit \
	cargo-auditable \
	cargo-deny \
	cargo-insta \
	cargo-msrv \
	cargo-nextest \
	cargo-shear \
	cargo-tarpaulin \
	cargo-udeps \
	mdbook \
	mdbook-mermaid \
	mdbook-pikchr
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
TASKS = \
	audit \
	book \
	build \
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
	fuzz \
	fuzz-repro \
	help \
	homebrew-formula \
	publish-book \
	setup \
	test \
	test-ci \
	wasm \
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
	bucket-push-ghcr

.PHONY: $(TASKS)
