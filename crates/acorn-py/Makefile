#!make
-include .env

.PHONY: pypi
pypi:
	@maturin publish \
		--find-interpreter \
		--no-sdist \
		--non-interactive \
		--skip-existing \
		--auditwheel repair \
		--compatibility pypi \
		--username __token__ \
		--password $(MATURIN_PASSWORD)