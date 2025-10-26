# ACORN 🌱
[<img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-research_enablement-00662C?style=for-the-badge&logo=gitlab&color=%2300662C
" height="20">](https://research.ornl.gov)
[<img alt="PyPI" src="https://img.shields.io/pypi/v/acorn-py?style=for-the-badge&logo=python" height="20">](https://pypi.org/project/acorn-py/)
> Python bindings for the [acorn-lib](https://crates.io/crates/acorn-lib) crate

## Example Usage
```python
from acorn.schema.validate import is_ark, is_doi, is_orcid, is_ror

assert is_ark("https://n2t.net/ark:12148/btv1b8449691v/f29")
assert is_doi("10.11578/dc.20250604.1")
assert is_orcid("https://orcid.org/0000-0002-2057-9115")
assert is_ror("01qz5mb56")
```