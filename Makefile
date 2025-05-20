.PHONY: clean lint test coverage doc build release version

clean:
	@rm -rf ./bin
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
	@cargo build --release --target $(DEFAULT_TARGET)
	@mv target/$(DEFAULT_TARGET)/release/acorn-cli ./bin/acorn-v$(VERSION)

bucket-image:
	@docker build -t bucket:$(VERSION) -f Dockerfile.bucket .

release: clean
	@echo "Under Construction"

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