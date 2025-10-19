from acorn.schema.validate import (
    format_doi,
    format_orcid,
    format_phone_number,
    is_doi,
    is_ip6,
    is_orcid,
    is_ror,
)
from pytest import raises


def test_format_phone_number():
    assert format_phone_number("555-123-4567") == "555.123.4567"
    assert format_phone_number("(123) 456-7890") == "123.456.7890"
    with raises(ValueError) as err:
        format_phone_number("not a phone number")
    assert err.value.args[0] == "Unable to format telephone number"


def test_format_doi():
    expected = "10.11578/dc.20250604.1"
    assert format_doi(expected) == expected
    assert format_doi(f"https://doi.org/{expected}") == expected
    assert not format_doi("https://doi.org/42.11578/dc.20250604.1")
    assert not format_doi("totally invalid string")


def test_format_orcid():
    assert (
        format_orcid("https://orcid.org/0000-0002-2057-9115")
        == "https://orcid.org/0000-0002-2057-9115"
    )
    assert format_orcid("0000-0002-2057-9115") == "https://orcid.org/0000-0002-2057-9115"
    assert (
        format_orcid("https://orcid.org/0009-0005-5568-6526")
        == "https://orcid.org/0009-0005-5568-6526"
    )
    assert format_orcid("0009-0005-5568-6526") == "https://orcid.org/0009-0005-5568-6526"


def test_is_doi():
    # ACORN on DOE CODE
    assert is_doi("10.11578/dc.20250604.1")
    # Can include domain
    assert is_doi("https://doi.org/10.11578/dc.20250604.1")
    # 42 is not valid directory indicator
    assert not is_doi("https://doi.org/42.11578/dc.20250604.1")
    # Just plain wrong
    assert not is_doi("totally invalid string")


def test_is_ip6():
    assert is_ip6("::")
    assert is_ip6("2001:0db8:85a3:0000:0000:8a2e:0370:7334")
    assert not is_ip6("192.168.1.1")


def test_is_orcid():
    # Jason Wohlgemuth
    assert is_orcid("https://orcid.org/0000-0002-2057-9115")
    assert is_orcid("https://orcid.org/0009-0005-5568-6526")
    assert not is_orcid("1234-5678-9101-1121")
    assert not is_orcid("abc-0000-0000-0000")


def test_is_ror():
    # ORNL
    assert is_ror("01qz5mb56")
    assert is_ror("https://ror.org/01qz5mb56")
    assert not is_ror("03ebg0v16-")
    assert not is_ror("10.1000/182")
