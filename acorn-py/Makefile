#!make
-include .env

.PHONY: pypi
pypi:
	@maturin publish --manifest-path ./acorn-py/Cargo.toml --no-sdist --non-interactive --username __token__ --password $(MATURIN_PASSWORD)