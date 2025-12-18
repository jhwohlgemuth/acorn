from acorn.schema.pid import DOI


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
