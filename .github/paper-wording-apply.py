#!/usr/bin/env python3
"""Apply the reviewed line edits only when every source/result hash matches."""
from pathlib import Path
import base64
import hashlib
import json
import re
import zlib

root = Path.cwd().resolve()
parts = [(root / f'.github/wording-payload/part-{i}.b64').read_text() for i in range(4)]
# Repair the single known transport duplication, then validate the complete payload.
parts[0] = parts[0].replace('a16piHcpiHcZd', 'a16piHcZd')
data = zlib.decompress(base64.b64decode(''.join(parts), validate=True))
if hashlib.sha256(data).hexdigest() != 'cd4355ce1c147e0037157021066c54366a0063b09e7d92450687a999674b0838':
    raise SystemExit('Editorial payload hash mismatch')
changes = json.loads(data)
patterns = {name: r'\\begin\{' + re.escape(name) + r'\}.*?\\end\{' + re.escape(name) + r'\}'
            for name in ('equation', 'align', 'align*', 'proposition', 'proof')}
patterns.update(display=r'\\\[.*?\\\]',
                citation=r'\\cite\w*\*?(?:\[[^\]]*\])*\{[^}]+\}',
                label=r'\\label\{([^}]+)\}')
pending = {}
receipt = {'baseline_revision': '0f820e6b0dac8f7f69f165ecf9a7a78becf37417',
           'scope': 'Plain-word revision followed by engineering-explanation revision of every prose paragraph, including footnotes and captions. Formal blocks reviewed and retained unchanged.',
           'files': {}, 'protected_counts': {name: 0 for name in patterns},
           'limits': ['Literal checks do not prove semantic equivalence or reader comprehension.',
                      'The independent formal manuscript concordance binding was already stale at the baseline and remains unchanged.']}
for relative, change in changes.items():
    path = (root / relative).resolve()
    if not path.is_relative_to(root / 'paper') or path.suffix not in ('.tex', '.json'):
        raise SystemExit(f'Unexpected edit path: {relative}')
    before_bytes = path.read_bytes()
    if hashlib.sha256(before_bytes).hexdigest() != change['before']:
        raise SystemExit(f'Stale source: {relative}')
    before = before_bytes.decode('utf-8')
    lines = before.splitlines(keepends=True)
    previous_end = 0
    for start, end, replacement in change['edits']:
        if not 0 <= previous_end <= start <= end <= len(lines):
            raise SystemExit(f'Invalid line edit: {relative}')
        previous_end = end
    for start, end, replacement in reversed(change['edits']):
        lines[start:end] = replacement.splitlines(keepends=True)
    after = ''.join(lines)
    after_bytes = after.encode('utf-8')
    if hashlib.sha256(after_bytes).hexdigest() != change['after']:
        raise SystemExit(f'Result differs from reviewed source: {relative}')
    checks = {}
    if path.suffix == '.tex':
        for name, pattern in patterns.items():
            old = re.findall(pattern, before, re.S)
            new = re.findall(pattern, after, re.S)
            if old != new:
                raise SystemExit(f'Protected content changed: {relative}/{name}')
            checks[name] = True
            receipt['protected_counts'][name] += len(old)
    receipt['files'][relative] = {'before_sha256': change['before'], 'after_sha256': change['after'], 'checks': checks}
    pending[path] = after_bytes
for path, content in pending.items():
    path.write_bytes(content)
    print('PASS reviewed result hash:', path.relative_to(root))
(root / 'export').mkdir(exist_ok=True)
(root / 'export/editorial-checks.json').write_text(json.dumps(receipt, indent=2) + '\n')
(root / 'export/changed-paths.txt').write_text('\n'.join(changes) + '\n')
print('PASS protected blocks:', receipt['protected_counts'])
