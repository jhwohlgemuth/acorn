from pathlib import Path

import acorn
from acorn.schema import ResearchActivity


def test_name():
    assert acorn.__name__ == "acorn"


def test_research_activity():
    location = "../../tests/fixtures/data/valid_project/index.json"
    data = ResearchActivity.read(location)
    assert data.title == "Valid Project"
    data = ResearchActivity.read(Path(location))
    assert data.title == "Valid Project"
    assert data.contact.email == "wohlgemuthjh@ornl.gov"
