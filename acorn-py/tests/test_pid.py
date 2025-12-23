from acorn.schema.pid import DOI, Patent


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
