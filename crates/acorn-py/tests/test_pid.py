from acorn.schema.pid import DOI, ISBN, ORCID, ROR, SWHID, Arxiv, Patent


def test_arxiv():
    pid = Arxiv.from_string("https://arxiv.org/pdf/1501.00001v2.pdf")
    assert pid.archive == ""
    assert pid.identifier == "arXiv:1501.00001v2"
    assert pid.schema_uri == "https://arxiv.org"
    assert pid.version == "v2"
    values = Arxiv.find_all("See arXiv:2106.09685 and arXiv:1412.7878v3.")
    assert [value.identifier for value in values] == [
        "arXiv:2106.09685",
        "arXiv:1412.7878v3",
    ]


def test_doi():
    pid = DOI.from_string("https://doi.org/10.11578/dc.20250604.1")
    assert pid.identifier == "10.11578/dc.20250604.1"
    assert pid.prefix == "10.11578"
    assert pid.schema_uri == "https://doi.org"
    assert pid.suffix == "dc.20250604.1"
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
    assert pid.schema_uri == "https://orcid.org/"
    assert pid.check_digit == "5"
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
    assert values[0].prefix_element == "978"
    assert values[0].registration_group == "0"
    assert values[0].publisher == "306"
    assert values[0].title == "40619"
    assert values[0].check_digit == "5"


def test_patent():
    pid = Patent.from_string("US 9,374,321 B2")
    assert str(pid) == "US 9374321 B2"
    assert pid.country_code == "US"
    assert pid.serial_number == "9374321"
    assert pid.kind_code == "B2"
    assert pid.year is None
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
    assert pid.schema_uri == "https://ror.org/"
    assert pid.check_digit == ""
    text = f"I know some orgs with ROR IDs {valid}, {valid} and {valid}. Please check them out."
    values = ROR.find_all(text)
    expected_count = 3
    assert len(values) == expected_count
    assert values[0].identifier == identifier
    assert values[1].identifier == identifier
    assert values[2].identifier == identifier


def test_swhid():
    identifier = "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a"
    pid = SWHID.from_string(identifier)
    assert str(pid) == identifier
    assert pid.identifier == identifier
    assert pid.core_identifier == identifier
    assert pid.object_type == "cnt"
    assert pid.schema_uri == "https://www.swhid.org/specification/v1.2"
    assert pid.url == f"https://archive.softwareheritage.org/{identifier}"
    values = SWHID.find_all(f"first {identifier};lines=1-2 and second {identifier}")
    assert len(values) == 2
    assert values[0].identifier == f"{identifier};lines=1-2"
    assert values[1].identifier == identifier
