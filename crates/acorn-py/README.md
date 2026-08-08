# ACORN 🌱
[<img alt="gitlab" src="https://img.shields.io/badge/code.ornl.gov-research_enablement-00662C?style=for-the-badge&logo=gitlab&color=%2300662C" height="20">](https://research.ornl.gov)
[<img alt="PyPI" src="https://img.shields.io/pypi/v/acorn-py?style=for-the-badge&logo=python" height="20">](https://pypi.org/project/acorn-py/)
> Python bindings for the [acorn-lib](https://crates.io/crates/acorn-lib) crate

## So what? Big deal...who cares?
`acorn-py` allows you to leverage best in class [persistent identifier](https://en.wikipedia.org/wiki/Persistent_identifier) validation and other [ACORN](https://acorn.ornl.gov) <sup>[[1]](#1)</sup> capabilities in your Python projects.

## Installation
### Pixi <sup>[[2]](#2)</sup>
> ⭐ Recommended
```shell
pixi add acorn-py --pypi
```

### Poetry
```shell
poetry add acorn-py
```

### Pip
```shell
pip install acorn-py
```

### Conda
> ⌛ Coming soon

## Example Usage
```python
from acorn.schema.validate import is_ark, is_doi, is_orcid, is_ror

# Archival Resource Keys
assert is_ark("https://n2t.net/ark:12148/btv1b8449691v/f29")

# Digital Object Identifiers
assert is_doi("10.11578/dc.20250604.1")

# Open Researcher and Contributor Identifiers
assert is_orcid("https://orcid.org/0000-0002-2057-9115")

# Research Organization Registry identifiers
assert is_ror("01qz5mb56")
```

---
**Footnotes**
-------------

[1]
---
The `acorn-py` Python wheel is built from Rust code using [Maturin](https://www.maturin.rs/). `acorn-py` does not support the entire ACORN API, see the [acorn-py project code](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-py/src/lib.rs) for details.

[2]
---
On linux, may need to augment your `pixi.toml` with
```toml
[system-requirements]
libc = { family = "glibc", version = "2.34" }
```