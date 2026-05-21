#!make
.PHONY: $(TASKS)
-include .env

setup:
	@rustup update stable
	@cargo install $(DEV_DEPENDENCIES) --locked
	@rustup target add $(TARGETS)

audit:
	@cargo udeps
	@cargo deny check
	@cargo audit \
		--ignore RUSTSEC-2020-0071 \
		--ignore RUSTSEC-2025-0055 \
		--ignore RUSTSEC-2025-0067 \
		--ignore RUSTSEC-2025-0068 \
		--ignore RUSTSEC-2025-0056 \
		--ignore RUSTSEC-2024-0388 \
		--ignore RUSTSEC-2024-0436

bench: lint
	@cargo test bench --release --package acorn-lib

book:
	@mdbook clean docs
	@mdbook build docs

book-write:
	@mdbook serve --open docs

build: clean
	@pwsh -Command New-Item -ItemType Directory -Path ".\bin" -Force
	@cargo build --release --package acorn-cli --target $(DEFAULT_TARGET)
	@mv $(ARTIFACT_TARGET_DIR)/$(DEFAULT_TARGET)/release/acorn ./bin/acorn-v$(VERSION)
	@cargo build --release --package acorn-cli --target $(DEFAULT_WINDOWS_TARGET)
	@pwsh -Command Move-Item -Path "$(ARTIFACT_TARGET_DIR)/$(DEFAULT_WINDOWS_TARGET)/release/acorn.exe" -Destination "./bin/acorn-v${VERSION}.exe" -Force

build-in-image: clean
	@docker image inspect $(REGISTRY)/acorn/rust:$(VERSION) > /dev/null 2>&1 || $(MAKE) rust-image
	@MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*' docker run --rm -e CARGO_TARGET_DIR=/app/target/acorn -v "$(CURDIR):/app" -v acorn-target:/app/target -w /app $(REGISTRY)/acorn/rust:$(VERSION) make build


clean:
	@cargo clean
	@pwsh -Command Remove-Item -Path "./bin" -Recurse -Force

changelog:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN}

changelog-release:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN} --latest

cover: clean lint
	@cargo tarpaulin --out lcov --line --skip-clean --no-dead-code --engine llvm

doc:
	@cargo clean --doc
	@cargo doc --no-deps --all-features

doctest:
	@cargo test --doc

lint:
	@cargo fmt --verbose
	@cargo clippy --workspace --no-deps --all-targets --fix --allow-dirty

publish-book: book
	@git clone "https://user:${ACCESS_TOKEN}@code-int.ornl.gov/sites/acorn.git" site
	@rm -rf ./site/html
	@mv docs/book ./site/html
	@cd ./site && git add html && git commit -m "Publish ACORN Book" && git push

test:
	@cargo nextest run

test-all:
	@cargo nextest run --run-ignored all

test-ci: clean lint
	@cargo test --lib --bins --tests
	@cargo test --lib --bins --tests --no-default-features

version: 
	@echo $(VERSION)

rust-version:
	@echo $(RUST_VERSION)

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
	@docker build --no-cache --platform $(PLATFORM) --build-arg RUST_VERSION=$(RUST_VERSION) -t acorn:$(VERSION) -f Dockerfile .
	@docker tag acorn:$(VERSION) acorn:latest
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:$(VERSION)
	@docker tag acorn:$(VERSION) $(REGISTRY)/acorn/runner:latest
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) jhwohlgemuth/acorn:latest
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:$(VERSION)
	@docker tag acorn:$(VERSION) ghcr.io/jhwohlgemuth/acorn:latest

acorn-push: acorn-push-docker acorn-push-ghcr
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
	@docker build --no-cache --platform $(PLATFORM) --build-arg RUST_VERSION=$(RUST_VERSION) -t $(REGISTRY)/acorn/rust:$(VERSION) -f Dockerfile.rust .
	@docker tag $(REGISTRY)/acorn/rust:$(VERSION) $(REGISTRY)/acorn/rust:latest

rust-push:
	@docker push $(REGISTRY)/acorn/rust:$(VERSION)
	@docker push $(REGISTRY)/acorn/rust:latest
#
# Variables
#
# PLATFORM = linux/amd64,linux/arm64
PLATFORM = linux/amd64
VERSION = `awk -F ' = ' '$$1 ~ /^version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
RUST_VERSION = `awk -F ' = ' '$$1 ~ /rust-version/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
REGISTRY = savannah.ornl.gov/research-enablement
DEFAULT_TARGET = `awk -F ' = ' '$$1 ~ /default-target/ { gsub(/["\r]/, "", $$2); printf("%s",$$2) }' Cargo.toml | tr . _`
DEFAULT_WINDOWS_TARGET = x86_64-pc-windows-gnu
ARTIFACT_TARGET_DIR = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
DEV_DEPENDENCIES = \
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
	x86_64-pc-windows-gnu
TASKS = \
	audit \
	book \
	build \
	changelog \
	changelog-release \
	clean \
	cover \
	doc \
	lint \
	publish-book \
	setup \
	test \
	test-ci \
	version \
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