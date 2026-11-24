# 🐍 Python bindings

[`acorn-py`](https://code.ornl.gov/research-enablement/acorn-py) is an independently maintained companion project that provides Python bindings for ACORN's published Rust libraries.

The Python package has its own source, issue tracker, documentation, CI, release process, and version lifecycle. Its version does not need to match the version of this ACORN workspace.

- [Source and documentation](https://code.ornl.gov/research-enablement/acorn-py)
- [Python package on PyPI](https://pypi.org/project/acorn-py/)
- [Rust API documentation](https://docs.rs/acorn-lib/latest/acorn/)

## Example: validate and format identifiers

After installing `acorn-py`, use its [identifier functions](https://pypi.org/project/acorn-py/#example-usage) directly:

```python
from acorn.schema.validate import (
    format_doi,
    is_ark,
    is_arxiv,
    is_doi,
    is_orcid,
    is_ror,
)

doi = format_doi("https://doi.org/10.11578/dc.20250604.1")
assert doi == "10.11578/dc.20250604.1"
assert is_doi(doi)
assert is_ark("https://n2t.net/ark:12148/btv1b8449691v/f29")
assert is_arxiv("arXiv:2106.09685")
assert is_orcid("https://orcid.org/0000-0002-2057-9115")
assert is_ror("01qz5mb56")
assert not is_doi("not-a-doi")
```

These functions check or format individual values locally. Use `acorn check` for full RAD validation and link checks.

Report Python binding and packaging issues in the standalone [`acorn-py` issue tracker](https://code.ornl.gov/research-enablement/acorn-py/-/issues). Report issues in the underlying Rust APIs in the [ACORN issue tracker](https://code.ornl.gov/research-enablement/acorn/-/issues).
