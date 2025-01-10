.PHONY: doc lint test

clean:
	@rm -rf ./bin
	@cargo clean

lint:
	@cargo fmt --verbose
	@cargo clippy --no-deps --all-targets --fix --allow-dirty

test: clean lint
	@cargo build
	@cargo test
	@cargo tarpaulin --out lcov --line --skip-clean

test-ci: clean lint
	@cargo build
	@cargo test

test-cross: clean
	@for target in $(TARGETS) ; do \
        cross test --target $$target; \
    done

doc:
	@cargo clean --doc
	@cargo doc --no-deps --all-features

build: clean
	@mkdir -p ./bin
	@cargo build --release --target $(DEFAULT_TARGET)
	@mv target/$(DEFAULT_TARGET)/release/pipe-cli ./bin/pipe-v$(VERSION)

build-cross: clean
	@mkdir -p ./bin
	@for target in $(TARGETS) ; do \
		export DISPLAY=:0; \
        cross build --release --target $$target; \
		mv target/$${target}/release/pipe-cli ./bin/pipe_v$(VERSION)_$${target}; \
    done

release: clean
	@echo "Under Construction"

version: 
	@echo $(VERSION)
#
# Variables
#
VERSION = `awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
DEFAULT_TARGET = `awk -F ' = ' '$$1 ~ /default-target/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cross.toml | tr . _`
TARGETS = \
	i586-unknown-linux-musl \
	i686-unknown-linux-gnu \
	aarch64-unknown-linux-musl \
	arm-unknown-linux-gnueabi \
	arm-unknown-linux-musleabi \
	riscv64gc-unknown-linux-gnu \
	x86_64-unknown-linux-gnu \
	x86_64-unknown-linux-musl