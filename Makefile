#!make
.PHONY: $(TASKS)
-include .env

audit:
	@cargo audit

build: clean
	@mkdir -p ./bin
	@cargo build --release --package acorn-cli --target $(DEFAULT_TARGET)
	@mv target/$(DEFAULT_TARGET)/release/acorn ./bin/acorn-v$(VERSION)

clean:
	@cargo clean

lint:
	@cargo fmt --verbose
	@cargo clippy --workspace --no-deps --all-targets --fix --allow-dirty

test:
	@cargo nextest run

test-ci: clean lint
	@cargo test

changelog:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN}

changelog-release:
	@git pull origin main --tags
	@git-cliff --output CHANGELOG.md --gitlab-token ${GITLAB_TOKEN} --latest

coverage: clean lint
	@cargo tarpaulin --out lcov --line --skip-clean --no-dead-code --engine llvm

doc:
	@cargo clean --doc
	@cargo doc --no-deps --all-features

publish-docs: doc
	@git clone "https://user:${ACCESS_TOKEN}@code-int.ornl.gov/sites/acorn.git" site
	@rm -rf ./site/html
	@mv target/doc/ ./site/html
	@echo '<meta http-equiv="refresh" content="0; url=acorn_lib/index.html">' > ./site/html/index.html
	@cd site
	@git add html
	@git commit -m "Publish documentation"
	@git push
	@rm -rf site

release: clean
	@echo "Under Construction"

version: 
	@echo $(VERSION)

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
	@docker build --no-cache --platform $(PLATFORM) -t acorn:$(VERSION) -f Dockerfile .
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
	@docker build --no-cache --platform $(PLATFORM) -t bucket:$(VERSION) -f Dockerfile.bucket .
	@docker tag bucket:$(VERSION) bucket:latest
	@docker tag bucket:$(VERSION) $(REGISTRY)/bucket/runner:$(VERSION)
	@docker tag bucket:$(VERSION) $(REGISTRY)/bucket/runner:latest
	@docker tag bucket:$(VERSION) jhwohlgemuth/bucket:$(VERSION)
	@docker tag bucket:$(VERSION) jhwohlgemuth/bucket:latest
	@docker tag bucket:$(VERSION) ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker tag bucket:$(VERSION) ghcr.io/jhwohlgemuth/bucket:latest

bucket-push: bucket-push-docker bucket-push-ghcr
	@docker push $(REGISTRY)/bucket/runner:$(VERSION)
	@docker push $(REGISTRY)/bucket/runner:latest

bucket-push-docker:
	@docker push jhwohlgemuth/bucket:$(VERSION)
	@docker push jhwohlgemuth/bucket:latest

bucket-push-ghcr:
	@docker push ghcr.io/jhwohlgemuth/bucket:$(VERSION)
	@docker push ghcr.io/jhwohlgemuth/bucket:latest
#
# Variables
#
# PLATFORM = linux/amd64,linux/arm64
PLATFORM = linux/amd64
VERSION = `awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml`
REGISTRY = savannah.ornl.gov/research-enablement
# REGISTRY = camden.ornl.gov/research-enablement
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
TASKS = \
	audit \
	build \
	clean \
	lint \
	test \
	test-ci \
	changelog \
	changelog-release \
	coverage \
	doc \
	publish-docs \
	develop \
	pypi \
	release \
	version \
	acorn-clean \
	acorn-image \
	acorn-push \
	acorn-push-docker \
	acorn-push-ghcr \
	bucket-clean \
	bucket-image \
	bucket-push \
	bucket-push-docker \
	bucket-push-ghcr