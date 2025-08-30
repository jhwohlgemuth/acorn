#!make
.PHONY: audit clean lint test coverage doc build release version
-include .env

audit:
	@cargo audit

clean:
	@cargo clean

lint:
	@cargo fmt --verbose
	@cargo clippy --workspace --no-deps --all-targets --fix --allow-dirty

test:
	@cargo nextest run

test-ci: clean lint
	@cargo test

coverage: clean lint
	@cargo tarpaulin --out lcov --line --skip-clean

doc:
	@cargo clean --doc
	@cargo doc --no-deps --all-features

build: clean
	@mkdir -p ./bin
	@cargo build --release --package acorn --target $(DEFAULT_TARGET)
	@mv target/$(DEFAULT_TARGET)/release/acorn ./bin/acorn-v$(VERSION)

acorn-image:
	@docker build -t acorn:$(VERSION) -f Dockerfile .
	@docker tag acorn:$(VERSION) acorn:latest

bucket-image:
	@docker build -t bucket:$(VERSION) -f Dockerfile.bucket .
	@docker tag bucket:$(VERSION) bucket:latest

release: clean
	@echo "Under Construction"

changelog:
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN}

version: 
	@echo $(VERSION)
#
# Variables
#
VERSION = `awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
# DEFAULT_TARGET = `awk -F ' = ' '$$1 ~ /default-target/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cross.toml | tr . _`
DEFAULT_TARGET = x86_64-unknown-linux-gnu
TARGETS = \
	i586-unknown-linux-musl \
	i686-unknown-linux-gnu \
	aarch64-unknown-linux-musl \
	arm-unknown-linux-gnueabi \
	arm-unknown-linux-musleabi \
	riscv64gc-unknown-linux-gnu \
	x86_64-unknown-linux-gnu \
	x86_64-unknown-linux-musl