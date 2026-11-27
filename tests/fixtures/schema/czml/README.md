# CZML conformance fixtures

The 36 files in `czml-writer/` are byte-for-byte copies of `Schema/Examples/*.json` from [AnalyticalGraphicsInc/czml-writer at `61cd1d5a890ad955da081250849b99c5fd238921`](https://github.com/AnalyticalGraphicsInc/czml-writer/tree/61cd1d5a890ad955da081250849b99c5fd238921/Schema/Examples). They are distributed under Apache-2.0; `NOTICE` includes the upstream copyright and license text. The files in `acorn/` are fictional ACORN-authored examples.

To regenerate the copied examples from a checkout of that revision:

```sh
cp /path/to/czml-writer/Schema/Examples/*.json tests/fixtures/schema/czml/czml-writer/
```

`schema-inventory.json` was generated from the same revision's top-level `Schema/*.json` files with:

```sh
python3 - <<'PY'
from pathlib import Path
import json
root = Path('/path/to/czml-writer/Schema')
types = {}
for path in sorted(root.glob('*.json')):
    properties = json.loads(path.read_text()).get('properties', {})
    types[path.stem] = {
        'properties': sorted(properties),
        'requiredForDisplay': sorted(key for key, value in properties.items() if value.get('czmlRequiredForDisplay') is True),
    }
Path('tests/fixtures/schema/czml/schema-inventory.json').write_text(
    json.dumps({'revision': '61cd1d5a890ad955da081250849b99c5fd238921', 'types': types}, indent=2) + '\n'
)
PY
```

The schema and examples disagree in a few places. The conformance test records each reviewed difference explicitly, rather than silently altering the upstream files.
