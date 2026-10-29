"""Check the exact reviewed files after moving the supplement into a repository."""
from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parent
manifest = json.loads((root / "SHA256SUMS.json").read_text())
for relative, expected in manifest.items():
    path = root / relative
    if not path.is_file():
        raise SystemExit(f"Missing packaged file: {relative}")
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Changed reviewed file: {relative}")
for name in ("main.tex", "references.bib"):
    if (root / "article" / name).read_bytes() != (
        root / "formal/manuscript" / name
    ).read_bytes():
        raise SystemExit(f"Article and formal source differ: {name}")
print(f"PASS: {len(manifest)} packaged file hashes; manuscript copies match.")
