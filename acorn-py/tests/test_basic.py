import acorn
from acorn.schema.validate import format_phone_number, is_doi, is_ip6, is_orcid, is_ror
from pytest import raises


def test_name():
    assert acorn.__name__ == "acorn"


def test_format_phone_number():
    assert format_phone_number("555-123-4567") == "555.123.4567"
    assert format_phone_number("(123) 456-7890") == "123.456.7890"
    with raises(ValueError) as err:
        format_phone_number("not a phone number")
    assert err.value.args[0] == "Unable to format telephone number"


def test_is_doi():
    # ACORN on DOE CODE
    assert is_doi("10.11578/dc.20250604.1")
    # Should not include domain
    assert not is_doi("https://doi.org/10.11578/dc.20250604.1")


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
