# 🐍 Python Bindings
[<img alt="PyPI Version" src="https://img.shields.io/pypi/v/acorn-py?style=for-the-badge&logo=python" height="20">](https://pypi.org/project/acorn-py/)
[<img alt="PyPI Downloads" src="https://img.shields.io/pypi/dm/acorn-py?style=for-the-badge" height="20">](https://pypi.org/project/acorn-py/)

ACORN seeks to meet scientists where they are in all aspects. This includes the programming languages they use. The `acorn-py` package provides Python bindings to the core ACORN functionalities provided by the `acorn-lib` [Rust crate](./acorn-lib.md).

## Installation
- See the [PyPI page](https://pypi.org/project/acorn-py/) for installation and usage instructions
- Use `acorn-lib` functions in Python
  ```python
  from acorn.schema.validate import is_ark, is_doi, is_orcid, is_ror

  assert is_ark("ark:/1234/w5678")
  assert is_doi("10.11578/dc.20250604.1")
  assert is_orcid("https://orcid.org/0000-0002-2057-9115")
  assert is_ror("01qz5mb56")
  ```