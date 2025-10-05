import acorn
from acorn.schema.validate import is_orcid

def test_name():
    assert acorn.__name__ == "acorn"

def test_is_orcid():
    assert is_orcid("https://orcid.org/0000-0002-2057-9115")
    assert is_orcid("https://orcid.org/0009-0005-5568-6526")
    assert not is_orcid("1234-5678-9101-1121")
    assert not is_orcid("abc-0000-0000-0000")