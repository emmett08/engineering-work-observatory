#!/usr/bin/env python3
"""Project-local figure rendering and bounded checks; not the unavailable skill renderer."""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

import fitz
import jsonschema
from PIL import Image

PAPER = Path(__file__).resolve().parents[2]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def local(relative: str) -> Path:
    path = (PAPER / relative).resolve()
    if not path.is_relative_to(PAPER):
        raise ValueError(f"Path escapes paper root: {relative}")
    return path


def validate_spec(path: Path) -> dict:
    spec = json.loads(path.read_text())
    schema = json.loads((Path(__file__).parent / "schemas/figure-spec.schema.json").read_text())
    jsonschema.Draft202012Validator(schema).validate(spec)
    files = {spec["build"]["source"], *spec["build"]["dependencies"]}
    for item in spec["inputs"]:
        if "path" in item:
            files.add(item["path"])
            if "sha256" in item and sha(local(item["path"])) != item["sha256"]:
                raise ValueError(f"Stale input hash: {item['path']}")
    for name in files:
        if not local(name).is_file():
            raise ValueError(f"Missing declared local file: {name}")
    objects = {item["id"] for group in ("quantities", "constructions") for item in spec.get(group, [])}
    inputs = {item["id"] for item in spec["inputs"]}
    for group in ("quantities", "constructions"):
        for item in spec.get(group, []):
            if not set(item["input_ids"]) <= inputs:
                raise ValueError("Unknown input reference")
    for item in spec["encodings"]:
        if not set(item.get("quantity_ids", []) + item.get("construction_ids", [])) <= objects:
            raise ValueError("Unknown encoding reference")
    return spec


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, help="Relative to paper/")
    parser.add_argument("--output", required=True, help="Relative to paper/")
    parser.add_argument("--preview-dir", required=True, type=Path, help="Scratch colour/greyscale/thumbnail destination")
    parser.add_argument("--spec", type=Path)
    parser.add_argument("--audit", type=Path)
    args = parser.parse_args()
    source, output = local(args.source), local(args.output)
    spec = validate_spec(args.spec.resolve()) if args.spec else None
    text = source.read_text()
    if text.count("\\begin{tikzpicture}") != 1 or any(token in text for token in ("\\documentclass", "\\begin{figure}", "\\caption{")):
        raise ValueError("Expected one include-ready tikzpicture without a document or float")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="horizon-figure-") as tmp:
        work = Path(tmp)
        wrapper = work / "preview.tex"
        wrapper.write_text(
            "\\documentclass[border=0pt]{standalone}\n"
            "\\usepackage[T1]{fontenc}\n\\usepackage{lmodern,pgfplots}\n"
            "\\pgfplotsset{compat=1.18}\n\\begin{document}\n"
            f"\\input{{{source.as_posix()}}}\n\\end{{document}}\n"
        )
        command = ["pdflatex", "-no-shell-escape", "-halt-on-error", "-interaction=nonstopmode",
                   "-recorder", f"-output-directory={work}", str(wrapper)]
        result = subprocess.run(command, cwd=PAPER / "article", capture_output=True, text=True, timeout=120)
        log = result.stdout + result.stderr
        if result.returncode:
            raise RuntimeError(log[-12000:])
        shutil.copyfile(work / "preview.pdf", output)
        typesetting = [line for line in log.splitlines() if any(s in line for s in ("Warning:", "Overfull", "Underfull"))]
    doc = fitz.open(output)
    page = doc[0]
    spans = [span for block in page.get_text("dict")["blocks"] if "lines" in block
             for line in block["lines"] for span in line["spans"] if span["text"].strip()]
    sizes = [s["size"] * 72.27 / 72 for s in spans]
    strokes = [p["width"] * 72.27 / 72 for p in page.get_drawings()
               if p["type"] in ("s", "fs") and p.get("stroke_opacity", 0) > 0]
    extracted = " ".join(page.get_text().split())
    errors, warnings = [], []
    target = spec["target"] if spec else {"width_mm": 152.4, "min_text_pt": 8, "min_stroke_pt": .35}
    width_mm, height_mm = page.rect.width * 25.4 / 72, page.rect.height * 25.4 / 72
    if len(doc) != 1:
        errors.append("Figure PDF must have one page")
    if abs(width_mm - target["width_mm"]) > target.get("width_tolerance_mm", .5):
        errors.append(f"Width {width_mm:.3f} mm differs from declared width")
    if min(sizes) + .02 < target["min_text_pt"]:
        errors.append(f"Minimum extracted font {min(sizes):.3f} pt is below floor")
    if strokes and min(strokes) + .02 < target.get("min_stroke_pt", .35):
        errors.append(f"Minimum stroked width {min(strokes):.3f} pt is below floor")
    for label in target.get("expected_labels", []):
        if label not in extracted:
            errors.append(f"Expected label absent from extracted text: {label}")
    for s in spans:
        if not page.rect.contains(fitz.Rect(s["bbox"])):
            warnings.append("Text bounding box extends beyond PDF page: " + s["text"])
    args.preview_dir.mkdir(parents=True, exist_ok=True)
    pix = page.get_pixmap(matrix=fitz.Matrix(2, 2), alpha=False)
    colour = args.preview_dir / "colour.png"
    pix.save(colour)
    im = Image.open(colour)
    im.convert("L").save(args.preview_dir / "greyscale.png")
    im.thumbnail((360, 240))
    im.save(args.preview_dir / "thumbnail.png")
    declared = set(spec["build"]["dependencies"]) if spec else set()
    if spec:
        declared.update(i["path"] for i in spec["inputs"] if "path" in i)
    audit = {
        "audit_kind": "project-local bounded mechanical checks",
        "limitation": "The skill's bundled figure_spec.py/render_tikz.py/figure_review.py were unavailable. This script does not claim their full validation coverage.",
        "source": args.source, "source_sha256": sha(source),
        "pdf": args.output, "pdf_sha256": sha(output),
        "spec_sha256": sha(args.spec.resolve()) if args.spec else None,
        "engine": "pdflatex", "python": platform.python_version(),
        "width_mm": width_mm, "height_mm": height_mm,
        "min_text_pt": min(sizes), "min_stroke_pt": min(strokes) if strokes else None,
        "fonts": sorted({s["font"] for s in spans}), "raster_images": len(page.get_images()),
        "inputs": [{"path": name, "sha256": sha(local(name))} for name in sorted(declared)],
        "errors": errors, "warnings": warnings, "typesetting_warnings": typesetting,
        "semantic_review_required": True,
    }
    if args.audit:
        args.audit.write_text(json.dumps(audit, indent=2) + "\n")
    print(json.dumps({k: audit[k] for k in ("width_mm", "height_mm", "min_text_pt", "min_stroke_pt", "errors", "warnings", "typesetting_warnings")}, indent=2))
    if errors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
