# From activity to evidence

An editable, 39-slide introduction to Engineering Work Observatory for engineers
without advanced mathematics. It connects PR #9's execution instrumentation to
human-work measurement, retained requirement output, exact labour bounds,
sample-based estimation and the separate merged-PR forecast.

- [PowerPoint presentation](observatory-model.pptx)
- [PDF handout](observatory-model.pdf)
- [Speaker notes and sources](speaker-notes.md)
- [Slide manifest](slide-manifest.json)
- [Build validation](build-validation.json)

Allow 45–60 minutes plus discussion. Slides 1–32 are the teaching sequence;
33–39 are reference slides. The opening thought experiment and slide 31 invite
audience participation; the notes include answers and suggested explanations.
The diagrams and text are native PowerPoint objects and can be edited. The PDF
contains slides only; use PowerPoint's Notes view or the Markdown notes for the
full explanation.

## Teaching sequence

| Slides | Question | Approach |
| --- | --- | --- |
| 1–5 | What do we want to know? | A tool-runtime puzzle, a requirement journey and an evidence map |
| 6–15 | What actually executed, and how do we know? | Account versus controller, invocation identity, wrappers, collection and gaps |
| 16–21 | How do we measure work and retained output? | Three clocks, shared attention, fixed horizons and a small cohort |
| 22–25 | What can incomplete observations tell us? | Timeline bounds, shared capacity and four sampled intervals |
| 26–27 | What might merge next? | A retained public forecast and its adverse checks |
| 28–32 | Which claims can we defend? | Causal limits, the observed pilot, adoption and discussion |
| 33–39 | Where are the details? | Field provenance, invariants, equations, assumptions and linked sources |

Each slide labels its evidence status. Manufactured teaching examples are
separate from the observed public pilot and the retained October 2026 forecast
case. Sources are pinned to implementation revision
`e252eae943a2cc5de27c2db9f60ed9c73a790536`; the deck is not a live forecast.
GitHub's account identity remains separate from declared control. Runtime does
not become human labour, unknown values do not become zero, and the pilot does
not establish an AI productivity effect.

## Rebuild

From the repository root, with Python 3.10 or later:

```sh
python3 -m venv work/deck-venv
work/deck-venv/bin/pip install -r docs/presentations/observatory-model/requirements.txt
work/deck-venv/bin/python docs/presentations/observatory-model/build.py
```

The builder uses local source fixtures and native shapes; it downloads no assets.
It writes the PPTX, notes, manifest and validation JSON beside the source. Use
`--output-dir PATH` for an alternate destination. ZIP timestamps and document
metadata are fixed for reproducibility. Exact bytes also depend on the Python
and dependency versions.

Export the PDF with PowerPoint, or with LibreOffice:

```sh
soffice --headless --convert-to pdf \
  --outdir docs/presentations/observatory-model \
  docs/presentations/observatory-model/observatory-model.pptx
```

Arial is the requested presentation font. Font substitution and application
versions can change line wrapping; inspect a new export before presenting.

## Verification

The builder checks the retained pilot counts, historical daily counts and worked
arithmetic, reopens the PPTX, verifies slide bounds, and requires speaker notes
on every slide. The generated deck contains 39 slides and 529 native text
objects. Independent content reviews checked provenance claims and mathematical
examples; the 44-minute sampling example was also reproduced with the
repository's `estimate-labour/v2` command.

The committed presentation was exported with LibreOffice, checked for retained
slide text, and visually reviewed as slide thumbnails and at full size on the
denser and revised slides. Text crowding found during review was corrected.
These checks cover the supplied render; the deck has not been opened in
Microsoft PowerPoint. Speaker notes provide a text alternative to the diagrams.
