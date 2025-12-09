# 🐍 Python Bindings
[<img alt="PyPI" src="https://img.shields.io/pypi/v/acorn-py?style=for-the-badge&logo=python" height="20">](https://pypi.org/project/acorn-py/)
> [!CAUTION]
> `acorn-py` API is currently experimental and may change without warning.

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