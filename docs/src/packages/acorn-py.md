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

## Working with scientific artifact identifiers
The `acorn-py` package provides tools to work with common scientific artifact identifiers such as [DOI](https://www.doi.org/)s, [ARK](https://arks.org/)s, [ORCID](https://orcid.org/)s, and [ROR](https://ror.org/)s and other indirectly related identifiers such as patent numbers and books (e.g., [ISBN](https://www.isbn.org/)s). You can validate these identifiers, work with their components, and even extract them from text!

> [!TIP]
> See the [acorn-lib documentation](https://docs.rs/acorn-lib/latest/acorn_lib/schema/pid/index.html) for more details on persistent identifiers and how to work with them in your code.

### Example
Find all patent numbers in a string
🐍 Python
```python
from acorn.schema.pid import Patent

text = "The patent number for my work is US1234567B1."
values = Patent.find_all(text)
patent = values[0]

assert str(patent) == "US 1234567 B1"
assert patent.country_code == "US"
assert patent.serial_number == "1234567"
assert patent.kind_code == "B1"
```
🦀 Rust
```rust
use acorn_lib::schema::pid::Patent;

let text = "The patent number for my work is US1234567B1.";
let values = Patent::find_all(&text);
let patent = values[0];

assert_eq!(patent.to_string(), "US 1234567 B1");
```

## API Consistency
The `acorn-py` API strives to adhere to the Rust API as closely as possible. If you know `acorn-lib`, you know `acorn-py`.

### Validate DOIs
🦀 Rust
```rust
use acorn_lib::schema::validate::DOI;

assert!("10.11578/dc.20250604.1".is_doi());
```
🐍 Python
```python
from acorn.schema.validate import is_doi

assert is_doi("10.11578/dc.20250604.1")
```

### Find all DOI values in a string
🦀 Rust
```rust
use acorn_lib::schema::pid::DOI;

let pid = "https://doi.org/10.11578/dc.20250604.1";
let text = format!("The DOI for ACORN is: {pid}");
let values = DOI::find_all(&text);
assert_eq!(values[0].identifier(), "10.11578/dc.20250604.1");
```
🐍 Python
```python
from acorn.schema.pid import DOI

pid = "https://doi.org/10.11578/dc.20250604.1"
text = f"The DOI for ACORN is: {pid}"
values = DOI.find_all(text)
assert values[0].identifier == "10.11578/dc.20250604.1"
```