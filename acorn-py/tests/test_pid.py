from acorn.schema.pid import DOI, ISBN, ORCID, ROR, Patent


def test_doi():
    pid = DOI.from_string("https://doi.org/10.11578/dc.20250604.1")
    assert pid.identifier == "10.11578/dc.20250604.1"
    valid = "https://doi.org/10.11578/dc.20250604.1"
    text = f"I made a thing with dois {valid}, {valid} and {valid}. Please check it out."
    values = DOI.find_all(text)
    expected_count = 3
    assert len(values) == expected_count
    assert values[0].identifier == "10.11578/dc.20250604.1"
    assert values[1].identifier == "10.11578/dc.20250604.1"
    assert values[2].identifier == "10.11578/dc.20250604.1"


def test_orcid():
    valid = "https://orcid.org/0000-0002-2057-9115"
    pid = ORCID.from_string(valid)
    assert pid.identifier == "0000-0002-2057-9115"
    text = f"I know some people with ORCIDs {valid}, {valid} and {valid}. Please check them out."
    values = ORCID.find_all(text)
    expected_count = 3
    assert len(values) == expected_count
    assert values[0].identifier == "0000-0002-2057-9115"
    assert values[1].identifier == "0000-0002-2057-9115"
    assert values[2].identifier == "0000-0002-2057-9115"


def test_isbn():
    text = "ISBN: 978-0-306-40619-5. It's my first book."
    values = ISBN.find_all(text)
    expected_count = 1
    assert len(values) == expected_count
    assert str(values[0]) == "978-0-306-40619-5"


def test_patent():
    pid = Patent.from_string("US 9,374,321 B2")
    assert str(pid) == "US 9374321 B2"
    assert pid.serial_number == "9374321"
    assert pid.is_granted()
    # Missing country code
    pid = Patent.from_string("9374321B2")
    assert str(pid) == "US 9374321 B2"
    assert pid.serial_number == "9374321"
    assert pid.kind_code == "B2"
    text = "Patent number: US 9,374,321 B2. It's my first one."
    values = Patent.find_all(text)
    expected_count = 1
    assert len(values) == expected_count
    values[0].serial_number == "9374321"


def test_ror():
    identifier = "01qz5mb56"
    valid = f"https://ror.org/{identifier}"
    pid = ROR.from_string(valid)
    assert pid.identifier == identifier
    text = f"I know some orgs with ROR IDs {valid}, {valid} and {valid}. Please check them out."
    values = ROR.find_all(text)
    expected_count = 3
    assert len(values) == expected_count
    assert values[0].identifier == identifier
    assert values[1].identifier == identifier
    assert values[2].identifier == identifier
