#!/usr/bin/env python3
"""Apply the reviewed correction after the pending paragraph revision."""
from pathlib import Path
import base64
import hashlib
import json
import re
import zlib

root = Path.cwd().resolve()
encoded = (root / '.github/paper-clarity-patch.b64').read_text().strip()
# Correct transport transcription only, before validating the entire payload.
repairs = {
    'XeapX6aV+6vtthGKOV5': 'XeapX6V+6vtVGOW5',
    'Hlz4m4Et3': 'Hlz4Et3',
    's2PCcO2PCcO2GY': 's2PCcO2GY',
    'uQhiINNxDg': 'uQhiINxDg',
    'pcMN7Z9ftg9g3': 'pcMN7Z9ft9g3',
    'GebWWeSjZS': 'GebWjZS',
}
for old, new in repairs.items():
    encoded = encoded.replace(old, new)
data = zlib.decompress(base64.b64decode(encoded, validate=True))
if hashlib.sha256(data).hexdigest() != 'ddeea4c5b8dc4bfa19b124d7ca44251de1d45c5ca9c4d23bad0bc53985da2c27':
    raise SystemExit('Correction payload does not match the reviewed bytes')
changes = json.loads(data)
pending = {}
for relative, change in changes.items():
    path = (root / relative).resolve()
    if not path.is_relative_to(root / 'paper'):
        raise SystemExit(f'Unexpected path: {relative}')
    original = path.read_bytes()
    if hashlib.sha256(original).hexdigest() != change['before']:
        raise SystemExit(f'Stale source: {relative}')
    lines = original.decode('utf-8').splitlines(keepends=True)
    last = 0
    for start, end, replacement in change['edits']:
        if not 0 <= last <= start <= end <= len(lines):
            raise SystemExit(f'Invalid line edit: {relative}')
        last = end
    for start, end, replacement in reversed(change['edits']):
        lines[start:end] = replacement.splitlines(keepends=True)
    revised = ''.join(lines).encode('utf-8')
    if hashlib.sha256(revised).hexdigest() != change['after']:
        raise SystemExit(f'Result differs from reviewed source: {relative}')
    pending[path] = revised
for path, data in pending.items():
    path.write_bytes(data)
    print('PASS corrected source:', path.relative_to(root))
(root / 'export').mkdir(exist_ok=True)
(root / 'export/clarity-source-checks.json').write_text(json.dumps({
    'payload_sha256': 'ddeea4c5b8dc4bfa19b124d7ca44251de1d45c5ca9c4d23bad0bc53985da2c27',
    'files': {k: {'before': v['before'], 'after': v['after']} for k, v in changes.items()},
    'timing_example': {'human_minutes': 40, 'execution_minutes': 70, 'overlap_minutes': 20},
    'editorial_limits': 'Model-assisted review and exact source checks, not measured reader comprehension.'
}, indent=2) + '\n')
with (root / 'export/changed-paths.txt').open('a') as f:
    f.write('\n'.join(changes) + '\n')
