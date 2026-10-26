"""Prepare isolated PA-15 inbox bytes locally; never upload or change a flow."""
import argparse
import json
from pathlib import Path


def build_fixtures(baseline, envelope):
    """Return exact byte fixtures for independent preservation/tail/size cases."""
    line = json.dumps(envelope, ensure_ascii=False, separators=(',', ':')).encode() + b'\n'
    limit = 10_485_759
    mixed = baseline + b'\n' + b'TEST ONLY rejected line\r\n' + '{"TEST_ONLY":"café 地図"}\n'.encode()
    prefix = b'{"TEST_ONLY":"'
    suffix = b'"}\n'
    def padded(total):
        return prefix + b'x' * (total - len(prefix) - len(suffix)) + suffix
    return {
        'malformed-tail.jsonl': mixed.removesuffix(b'\n'),
        'mixed-preservation.jsonl': mixed,
        'size-exact.jsonl': padded(limit - len(line)),
        'size-over.jsonl': padded(limit + 1 - len(line)),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('envelope', type=Path, help='Frozen single-line submission fixture')
    parser.add_argument('output', type=Path, help='New local directory; nothing is uploaded')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    envelope = json.loads(args.envelope.read_text())
    fixtures = build_fixtures((root / 'baseline.jsonl').read_bytes(), envelope)
    args.output.mkdir(parents=True, exist_ok=False)
    for name, content in sorted(fixtures.items()):
        (args.output / name).write_bytes(content)
        print(f'{name}: {len(content)} bytes')


if __name__ == '__main__':
    main()
