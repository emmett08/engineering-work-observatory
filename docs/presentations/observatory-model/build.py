#!/usr/bin/env python3
"""Build the editable engineering tutorial. No network access or external images."""
from __future__ import annotations

import argparse
import csv
import datetime as dt
import hashlib
import io
import json
from pathlib import Path
import zipfile

from pptx import Presentation
from pptx.dml.color import RGBColor
from pptx.enum.shapes import MSO_SHAPE
from pptx.enum.text import MSO_ANCHOR, PP_ALIGN
from pptx.util import Inches, Pt

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
REVISION = "e252eae943a2cc5de27c2db9f60ed9c73a790536"
BASE = f"https://github.com/emmett08/engineering-work-observatory/blob/{REVISION}/"
SOURCES = {
    "X": ("Execution-evidence contract", BASE + "docs/execution-evidence.md"),
    "P": ("Measurement protocol", BASE + "docs/measurement-protocol.md"),
    "L": ("Exact labour identification", BASE + "docs/labour-identification.md"),
    "E": ("Model-assisted estimation", BASE + "docs/design-based-estimation.md"),
    "F": ("Automatic forecasting contract", BASE + "docs/automatic-forecasting.md"),
    "D": ("Retained public forecast case, 30 October 2026", BASE + "docs/use-cases/engarg-eal-daily-2026-10-30/README.md"),
    "T": ("Public execution pilot and retained verification", BASE + "docs/execution-pilot.md"),
    "A": ("Architecture and evidence cuts", BASE + "docs/architecture.md"),
    "G": ("GitHub actor and triggering-actor contexts", "https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context"),
    "W": ("GitHub workflow_run completion event", "https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run"),
    "R": ("Receipt schema", BASE + "schemas/tool-receipt-v1.schema.json"),
    "J": ("Joint-capacity example input", BASE + "examples/joint-labour-bounds.json"),
}

NAVY = "142C40"
TEAL = "007F79"
BLUE = "3864A6"
ORANGE = "C45A32"
INK = "213B4D"
MUTED = "536773"
PAPER = "F7F5F0"
WHITE = "FFFFFF"
PALE = "E6EFEE"
PALEBLUE = "E8EDF6"
PALEORANGE = "F8E9E0"
LINE = "CCD7DB"
FONT = "Arial"


def rgb(value):
    return RGBColor.from_string(value)


class Deck:
    def __init__(self):
        self.prs = Presentation()
        self.prs.slide_width = Inches(13.333333)
        self.prs.slide_height = Inches(7.5)
        self.prs.core_properties.title = "From activity to evidence"
        self.prs.core_properties.subject = "Engineering Work Observatory: an engineer's guide to evidence, labour and forecasting"
        self.prs.core_properties.author = "Engineering Work Observatory"
        self.prs.core_properties.keywords = "execution evidence, human labour, retained output, forecasting, educational tutorial"
        self.prs.core_properties.created = dt.datetime(2026, 11, 1, 0, 0)
        self.prs.core_properties.modified = dt.datetime(2026, 11, 1, 0, 0)
        self.notes = []
        self.manifest = []
        self.slide = None
        self.dark = False

    def rect(self, x, y, w, h, fill, line=None, radius=False):
        shape = self.slide.shapes.add_shape(
            MSO_SHAPE.ROUNDED_RECTANGLE if radius else MSO_SHAPE.RECTANGLE,
            Inches(x), Inches(y), Inches(w), Inches(h),
        )
        shape.fill.solid()
        shape.fill.fore_color.rgb = rgb(fill)
        if line:
            shape.line.color.rgb = rgb(line)
            shape.line.width = Pt(1)
        else:
            shape.line.fill.background()
        if radius:
            shape.adjustments[0] = 0.08
        return shape

    def text(self, x, y, w, h, text, size=22, color=None, bold=False,
             align=PP_ALIGN.LEFT, font=FONT, valign=MSO_ANCHOR.TOP, link=None):
        shape = self.slide.shapes.add_textbox(Inches(x), Inches(y), Inches(w), Inches(h))
        frame = shape.text_frame
        frame.clear()
        frame.word_wrap = True
        frame.margin_left = frame.margin_right = 0
        frame.margin_top = frame.margin_bottom = 0
        frame.vertical_anchor = valign
        for i, line in enumerate(str(text).split("\n")):
            p = frame.paragraphs[0] if i == 0 else frame.add_paragraph()
            p.alignment = align
            p.space_after = Pt(7 if size >= 18 else 4)
            p.line_spacing = 1.08
            r = p.add_run()
            r.text = line
            r.font.name = font
            r.font.size = Pt(size)
            r.font.bold = bold
            r.font.color.rgb = rgb(color or (WHITE if self.dark else INK))
            if link:
                r.hyperlink.address = link
        return shape

    def arrow(self, x, y, w=0.35, h=0.28, color=TEAL):
        shape = self.slide.shapes.add_shape(MSO_SHAPE.CHEVRON, Inches(x), Inches(y), Inches(w), Inches(h))
        shape.fill.solid()
        shape.fill.fore_color.rgb = rgb(color)
        shape.line.fill.background()
        return shape

    def tag(self, x, y, text, color=TEAL, w=2.0):
        self.text(x, y, w, 0.26, text.upper(), 11, color, True)

    def base(self, section, title, notes, refs=(), status="Model definition", subtitle=None, dark=False):
        self.slide = self.prs.slides.add_slide(self.prs.slide_layouts[6])
        self.dark = dark
        self.slide.background.fill.solid()
        self.slide.background.fill.fore_color.rgb = rgb(NAVY if dark else PAPER)
        n = len(self.prs.slides)
        self.tag(0.55, 0.3, section, "68D0C0" if dark else TEAL, 10)
        self.text(0.55, 0.82, 12.1, 0.98, title, 32, bold=True)
        if subtitle:
            self.text(0.58, 1.83, 12.05, 0.65, subtitle, 18, "C8D7DE" if dark else MUTED)
        self.rect(0.55, 7.02, 12.22, 0.012, "4C6576" if dark else LINE)
        ref_text = " ".join(f"[{key}]" for key in refs)
        self.text(0.55, 7.13, 11.5, 0.2, f"{status}  •  {ref_text}  |  Sources and explanations in speaker notes", 9.5, "C8D7DE" if dark else MUTED)
        self.text(12.2, 7.11, 0.55, 0.25, f"{n:02}", 12, "C8D7DE" if dark else MUTED, align=PP_ALIGN.RIGHT)
        source_notes = "\n".join(f"[{key}] {SOURCES[key][0]}: {SOURCES[key][1]}" for key in refs)
        full_notes = f"SLIDE {n}: {title}\n\n{notes.strip()}\n\nEvidence status: {status}.\n\nSources\n{source_notes}"
        self.slide.notes_slide.notes_text_frame.text = full_notes
        self.notes.append(full_notes)
        self.manifest.append({"slide": n, "section": section, "title": title, "status": status, "sources": list(refs)})
        return self.slide

    def takeaway(self, text, y=6.23, color=TEAL):
        self.rect(0.55, y, 0.065, 0.48, color)
        self.text(0.76, y + 0.02, 11.9, 0.58, text, 18, "A5E2D8" if self.dark else color, True)

    def cards(self, items, y=2.55, h=3.15, colors=None):
        colors = colors or [TEAL, BLUE, ORANGE]
        gap = 0.25
        width = (12.22 - gap * (len(items) - 1)) / len(items)
        for i, (title, body) in enumerate(items):
            x = 0.55 + i * (width + gap)
            c = colors[i % len(colors)]
            self.rect(x, y, width, h, WHITE if not self.dark else "203F55")
            self.rect(x, y, width, 0.055, c)
            short = h < 2.8
            self.text(x + 0.22, y + 0.22, width - 0.44, 0.72, title, 21 if short else 23, c if not self.dark else "A5E2D8", True)
            self.text(x + 0.22, y + (0.92 if short else 1.13), width - 0.44, h - (1.05 if short else 1.32), body, 18 if short else 20)

    def table(self, columns, rows, widths, y=2.35, row_h=0.72, fontsize=18):
        x = 0.55
        for title, w in zip(columns, widths):
            self.rect(x, y, w, 0.58, NAVY)
            self.text(x + 0.15, y + 0.12, w - 0.3, 0.4, title, 16, WHITE, True)
            x += w
        for j, row in enumerate(rows):
            x = 0.55
            top = y + 0.58 + j * row_h
            for i, (cell, w) in enumerate(zip(row, widths)):
                self.rect(x, top, w, row_h - 0.025, WHITE if j % 2 == 0 else "EDF1F1")
                self.text(x + 0.15, top + 0.13, w - 0.3, row_h - 0.18, cell, fontsize, TEAL if i == 0 else INK, i == 0)
                x += w

    def save(self, output):
        output.mkdir(parents=True, exist_ok=True)
        memory = io.BytesIO()
        self.prs.save(memory)
        # Stable container timestamps and ordering make the PPTX reproducible.
        destination = output / "observatory-model.pptx"
        with zipfile.ZipFile(io.BytesIO(memory.getvalue())) as original, zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as new:
            for name in sorted(original.namelist()):
                info = zipfile.ZipInfo(name, (2026, 11, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                new.writestr(info, original.read(name))
        notes = "# From activity to evidence — speaker notes\n\n"
        notes += "Audience: engineers without advanced mathematics. Suggested delivery: 45–60 minutes plus discussion. Slides 1–32 form the main walkthrough; later slides are reference material.\n\n"
        notes += "\n\n---\n\n".join(self.notes)
        (output / "speaker-notes.md").write_text(notes, encoding="utf-8")
        (output / "slide-manifest.json").write_text(json.dumps({"source_revision": REVISION, "slides": self.manifest}, indent=2) + "\n")
        return destination


def build():
    d = Deck()
    d.base("ENGINEERING WORK OBSERVATORY", "From activity to evidence", "Open with a practical question: when a pipeline completes, what do we actually know about the engineering work? This tutorial connects PR #9's instrumentation to the repository's measurement and forecasting components. They answer different questions and must retain distinct evidence contracts. No calculus, probability course or optimisation background is assumed. Use the diagrams first; formulas and fuller assumptions are in the appendix. Suggested duration is 45–60 minutes with the two short audience exercises.", ["A", "X"], dark=True)
    d.text(0.6, 2.2, 8.0, 1.45, "What ran?\nWhat human work happened?\nWhat can we responsibly predict?", 29, WHITE)
    for i, label in enumerate(["OBSERVE", "MEASURE", "CHECK"]):
        d.rect(9.15, 2.2 + i * 1.1, 3.05, 0.8, [TEAL, BLUE, ORANGE][i])
        d.text(9.35, 2.43 + i * 1.1, 2.65, 0.4, label, 22, WHITE, True)
    d.text(0.6, 5.65, 10.8, 0.7, "An engineer's guide to the model, its implementation and its limits", 22, "C8D7DE")

    d.base("01 / THE QUESTION", "A tool ran for 20 minutes. How much human work?", "Ask the audience for an answer before revealing the three possibilities. The same tool runtime is compatible with unattended execution, active supervision or several people working together. The numbers are illustrative possibilities, not measurements of the pilot. Even zero attended minutes during execution would not establish zero labour for the whole requirement: specification, prompting, review and repair may occur outside that interval. Attended waiting is handled by a prespecified protocol convention.", ["P", "X"], "Illustrative thought experiment")
    d.text(0.6, 2.18, 3.0, 1.05, "20 min", 60, TEAL, True)
    d.text(3.8, 2.44, 8.4, 0.55, "is a tool-runtime observation.", 27)
    d.cards([("Unattended", "The engineer does other work while the tool runs."), ("Supervised", "The engineer watches, diagnoses or steers the run."), ("Collaborative", "Several people review or debug during the same interval.")], y=3.4, h=2.35)
    d.takeaway("A runtime receipt cannot choose between these human-work histories.")

    d.base("01 / THE MAP", "Three questions need three kinds of evidence", "Explain the architecture as three related contracts, not one all-purpose productivity score. Execution evidence establishes observed tool events and resources. The study-accounting path needs direct or sampled human allocations, acceptance and retention observations. The automatic forecasting path uses complete historical PR-count bins. PR #9 strengthens execution evidence, and its records can be linked to requirements. It does not supply the missing measurements needed by the other paths.", ["A", "P", "F", "X"])
    d.cards([("What executed?", "Receipts + GitHub metadata\nTool identity, outcome and reported resources"), ("What work and output?", "Human observations + assessments\nPerson-hours and retained requirement credit"), ("What might merge?", "Historical PR counts\nAn empirical forecast distribution")], h=3.35)
    d.takeaway("Connect the records; preserve the meaning of each quantity.")

    d.base("01 / THE STORY", "Follow one requirement through many activities", "Use this fictional requirement throughout the explanation. A single requirement can span several PRs, repositories, assistant sessions and test runs. A failed attempt still consumed resources and may have used human effort. PRs and tool calls therefore act as evidence about the requirement, rather than replacing the accounting unit. The frozen requirement and its score must precede implementation. The example does not assert any actual customer task from the public pilot.", ["P", "A"], "Illustrative engineering example", "Requirement R-17: export a report with verified checksums.")
    labels = [("Specify", "Agree the acceptance rule"), ("Implement", "Human + assistant work"), ("Verify", "Tests, failures and retries"), ("Accept", "Record explicit acceptance"), ("Revisit", "Assess retained function")]
    for i, (head, sub) in enumerate(labels):
        x = 0.6 + i * 2.48
        d.rect(x, 2.65, 2.15, 1.05, [TEAL, BLUE, BLUE, ORANGE, TEAL][i])
        d.text(x + 0.14, 2.97, 1.88, 0.4, head, 23, WHITE, True)
        d.text(x, 4.0, 2.18, 1.12, sub, 20)
        if i < 4: d.arrow(x + 2.2, 3.02, 0.23)
    d.takeaway("One requirement can have many executions—and unsuccessful work still counts.")

    d.base("02 / EVIDENCE", "Keep the source, the interpretation and the cut", "Walk left to right. Raw source bytes are retained by content hash. Validated records add explicit identity, observations and study meaning. A snapshot freezes the evidence available at a particular cut, so later observations or corrections do not rewrite an old result. Event time says when the work happened; observation time says when the Observatory learned about it. An old event collected today does not belong to yesterday's knowledge cut. Hash verification protects retained bytes; it does not establish that the original observation was true.", ["A"])
    d.cards([("1  Source bytes", "What was retrieved or imported?\nRetain the original bytes."), ("2  Validated records", "What does this record assert?\nKeep provenance and identity."), ("3  Frozen snapshot", "What was known at this cut?\nPreserve reproducible results.")])
    d.takeaway("Later evidence creates a new view; it does not silently rewrite an old one.")

    d.base("02 / IDENTITY", "The GitHub account is not the controller", "This is the central provenance distinction in PR #9. GitHub actor records the account associated with the original run; triggering_actor records the account that initiated the current run attempt, which can differ on a rerun. Neither field tells us whether a human, assistant or automation actually controlled the work. Controller and session provenance are caller declarations and remain labelled as such. Even runner-environment values inside an uploaded bundle are assertions until independently compared with the API.", ["G", "X"])
    d.cards([("Platform identity", "Original actor\nRerun actor\nTrigger event"), ("Declared controller", "human · assistant\nautomation · unknown\nSupporting provenance"), ("Session linkage", "Declared study session\nLogical call and parent\nTool-reported session, separately")], h=3.45)
    d.takeaway("Keep unknown when provenance is unavailable; do not infer it from the account.")

    d.base("02 / IDENTITY", "The same account can hide two different stories", "Both rows are hypothetical. In the first, an assistant uses Alice's credentials to run an ordinary test tool. Alice remains the GitHub actor, while assistant is a declaration supported by session provenance. In the second, Alice directly launches a workflow that uses Codex; human is the declared controller, while the tool is an assistant tool. A workflow containing AI does not by itself establish an assistant controller, and an assistant controller need not run an AI tool in every invocation. If evidence is missing, the controller stays unknown.", ["G", "X"], "Illustrative provenance scenarios")
    d.table(["Scenario", "GitHub actor", "Declared controller", "Tool"], [
        ["Assistant uses Alice's credentials", "Alice", "assistant", "cargo test"],
        ["Alice launches an AI workflow", "Alice", "human", "Codex"],
    ], [4.2, 2.1, 3.25, 2.67], row_h=1.22, fontsize=21)
    d.text(0.65, 5.55, 11.95, 0.6, "Credentials, control and tool type are separate observations.", 25, TEAL, True)

    d.base("02 / IDENTITY", "Identify an execution at the right level", "Explain the nested platform context first: stable repository ID, workflow revision, run ID, run attempt, job and matrix identity. A numeric job ID differs from the workflow job key. In this action, expanded job name, optional numeric ID and matrix metadata are caller supplied and marked job_context_source=caller. Within that context, each actual tool invocation receives a fresh UUID. Start and completion share that invocation ID. A logical-call link can connect retries, but does not collapse them into one physical execution.", ["X", "R"])
    levels = [("Repository + workflow revision", "Where and which definition?", 0.8), ("Run + attempt", "Original execution or rerun?", 1.4), ("Job + matrix identity", "Which parallel worker?", 2.0), ("Tool invocation + phase", "Which execution, start or completion?", 2.6)]
    for i, (a, b, x) in enumerate(levels):
        y = 2.25 + i * 0.83
        d.rect(x, y, 6.2 - (x - 0.8), 0.66, [NAVY, BLUE, TEAL, ORANGE][i])
        d.text(x + 0.18, y + 0.15, 5.84 - (x - 0.8), 0.4, a, 21, WHITE, True)
        d.text(7.6, y + 0.15, 4.4, 0.45, b, 17, MUTED)
    d.takeaway("Repeated delivery reuses identity. A real retry gets a new invocation ID.")

    d.base("02 / LIFECYCLE", "Write a start before the tool can fail", "The start record is written before launch. Completion records observed exit status and elapsed runtime after the process finishes. These are separate immutable files, not updates to one evolving row. The example shows a failed attempt followed by a new invocation for a retry. Both can retain the same logical-call ID; a declared parent link connects them. A workflow rerun changes the run attempt as well. Cancellation handling is best effort: runner loss can prevent completion and publication, so the independent job record remains essential.", ["X"], "Illustrative execution timeline")
    for row, (label, color, outcome) in enumerate([("Invocation A", ORANGE, "failure"), ("Invocation B", TEAL, "success")]):
        y = 2.55 + row * 1.4
        d.text(0.65, y + 0.18, 2.1, 0.5, label, 24, color, True)
        d.rect(3.0, y, 2.1, 0.85, color)
        d.text(3.2, y + 0.24, 1.7, 0.4, "START", 22, WHITE, True)
        d.arrow(5.45, y + 0.28, 0.55)
        d.text(6.2, y + 0.2, 2.0, 0.5, "Tool executes", 20)
        d.arrow(8.4, y + 0.28, 0.55)
        d.rect(9.25, y, 3.1, 0.85, color)
        d.text(9.42, y + 0.23, 2.8, 0.42, "COMPLETE: " + outcome, 18, WHITE, True)
    d.text(3.0, 5.5, 9.0, 0.45, "Same logical call; distinct actual invocations.", 23, TEAL)
    d.takeaway("Never overwrite the failed attempt with the successful retry.")

    d.base("02 / OBSERVABILITY", "A wrapper observes only the boundary you wrap", "The ordinary adapter launches an explicit command without adding its argv, stdout or unrestricted environment to receipts. Ordinary command output may still appear in normal Actions logs. The wrapper observes the process boundary, not every internal operation. The Codex adapter parses a small allowlist of documented JSON events, not the transcript. Separate jobs need their own setup and publication. Work outside Actions needs its own adapter or observation. There is no magic collector step that sees all tools, all jobs and all work. The coloured region is the declared instrumentation boundary, not an assertion of total coverage.", ["X"], subtitle="One setup step cannot automatically see every tool or every job.")
    d.rect(0.65, 2.7, 7.2, 2.5, PALE)
    d.tag(0.9, 2.93, "EXPLICITLY WRAPPED PROCESS", TEAL, 5)
    d.text(0.9, 3.5, 3.1, 0.8, "cargo test", 32, TEAL, True, font="Consolas")
    d.text(4.25, 3.5, 3.2, 1.1, "Start · exit status\nRuntime · named check", 22)
    d.text(8.55, 2.85, 3.8, 2.5, "Still outside this receipt:\nInternal tool operations\nOther jobs\nWork outside Actions", 21, MUTED)
    d.takeaway("Describe the boundary before interpreting the evidence.")

    d.base("02 / ARCHITECTURE", "Three components cooperate", "The action configures recording, captures explicit tool receipts and publishes a separate immutable bundle for each instrumented job. A durable collector downloads designated bundles, validates schema and hashes, deduplicates repeated deliveries and imports records. Independent reconciliation retrieves GitHub runs, attempts and jobs, including unsuccessful conclusions. This comparison is implemented within the collector command; independent refers to its GitHub metadata source, not a mandatory separate service. Parallel runners do not share an evolving SQLite database. The workflow_run completion observer can run regardless of the preceding conclusion, but still requires its own successful acquisition and durable retention.", ["X", "W"])
    d.cards([("1  Job recorders", "Explicit wrappers\nSeparate per-job bundles\nBest-effort publication"), ("2  Durable collector", "Download + validate\nDeduplicate + retain\nOne durable evidence store"), ("3  Reconciliation", "Independent GitHub metadata\nCompare runs and jobs\nKeep observation gaps")], y=2.45, h=3.2)
    d.arrow(4.45, 5.85, 0.42)
    d.arrow(8.6, 5.85, 0.42)
    d.takeaway("Runner-local recording and durable retention solve different problems.")

    d.base("02 / MISSINGNESS", "A missing receipt is an observation gap", "Read across the table rather than treating receipt count as tool count. Start plus completion supports a recorded process outcome; it does not prove complete job instrumentation. Start alone leaves completion unknown. Completion alone preserves the reported result while identifying a missing start record. A valid empty bundle reports setup context with zero receipt entries. Schema validity alone does not prove that recording was configured or that no tools ran. No bundle can reflect uninstrumented work or lost delivery. Independent failed, cancelled and skipped job records are retained in every case. A skipped platform job is not manufactured into a tool invocation.", ["X"])
    d.table(["What arrived?", "What the model can say"], [
        ["Start + completion", "A recorded invocation has both phases."],
        ["Start only / completion only", "Retain the phase; flag the missing observation."],
        ["Valid empty bundle", "The bundle reports setup context with zero receipt entries."],
        ["No bundle", "No receipts observed; tool use remains unknown."],
    ], [3.55, 8.67], row_h=0.74, fontsize=19)
    d.takeaway("Keep failed, cancelled and skipped GitHub jobs—even when no bundle arrives.")

    d.base("02 / THE RECEIPT", "A receipt answers six practical questions", "This is a human-readable map of the v1 receipt, not literal JSON. Repository/run/job context and caller declarations remain explicitly distinguishable. Tool version and configured model are supplied labels unless separately observed. Tool-reported usage remains a report rather than an invoice. Requirements and sessions are links; they do not certify acceptance or attention. Designated input and output files contribute hashes and sizes, without copying contents or paths into the receipt. The full field dictionary is in the appendix and schema.", ["R", "X"])
    d.table(["Question", "Evidence retained"], [
        ["Which execution?", "Repository, workflow revision, run/attempt, job/matrix, invocation"],
        ["Who or what initiated it?", "Actors + event; separate controller and provenance declarations"],
        ["Which tool and purpose?", "Tool/version, configuration, development or product execution"],
        ["What happened?", "Start/end, exit status, checks, runtime, reported usage and charges"],
        ["What does it relate to?", "Protocol/task, declared session, logical call, parent invocation"],
        ["Which artifacts, and what is missing?", "Missing fields and designated input/output artifact hashes"],
    ], [3.4, 8.82], y=2.1, row_h=0.69, fontsize=17)

    d.base("02 / RESOURCES", "Configured, reported and unknown stay separate", "The Codex adapter invokes codex exec --json and recognises thread.started and turn.completed events. In the verified event definitions, these report a thread ID and token usage, but no model identity. Therefore the observed model remains null even if the caller selected a model. The tool-reported session ID is separate from the declared research session. Token usage is not automatically complete billed usage or invoiced cost. Both initial adapters leave charge_amount and charge_currency null; they neither read invoices nor estimate charges. Product execution is also different from development assistance.", ["X", "R"])
    d.cards([("Configured", "The caller selected a model and supplied configuration metadata."), ("Tool-reported", "The adapter recognised a thread ID or token-usage field."), ("Unknown", "No recognised observation of model identity, charge or usage was available.")], h=3.2)
    d.takeaway("The initial Codex adapter leaves observed model identity unknown.")

    d.base("02 / TRUST", "A fingerprint checks bytes. Evidence checks claims.", "A SHA-256 hash is a content fingerprint: change the bytes and the fingerprint changes with overwhelming probability. It does not identify who truthfully observed the event. Schema validation checks permitted structure and values; it cannot certify the truth of those values. Independent API comparison can corroborate supported repository, run, attempt, actor and job fields, while caller declarations remain declarations. During collection, conflicting stable receipt identities are quarantined rather than overwritten, while independent GitHub evidence and raw acquisition records remain retained. Direct import rejects conflicting receipts.", ["X", "A"])
    d.cards([("Hash + schema", "Are these bytes unchanged?\nDoes the record satisfy the contract?"), ("Independent comparison", "Does the API support the platform context and job linkage?"), ("Human or tool provenance", "What supports the remaining declarations and observations?")])
    d.takeaway("Validation is a specific check—not a blanket certificate of truth.")

    d.base("03 / MEASUREMENT", "Now link the execution to a requirement", "Change the question from what ran to what engineering work and retained output occurred. The requirement is the accounting unit. Link its executions, PRs, sessions and revisions with explicit provenance. Human effort still needs an observation method; acceptance needs an explicit event and evidence; retained functionality needs an assessment against the frozen rubric. A passing CI receipt is evidence that a particular check passed, not that the requirement was accepted or retained in full. None of these study records is automatically created from a workflow duration.", ["P", "A", "X"])
    d.rect(4.75, 2.3, 3.8, 0.95, NAVY)
    d.text(4.95, 2.58, 3.4, 0.45, "Requirement R-17", 25, WHITE, True)
    d.cards([("Execution links", "Tool receipts, PRs, revisions and sessions"), ("Human work", "Observed or sampled allocations for every included role"), ("Outcome observations", "Acceptance and retained functionality against the rubric")], y=3.7, h=2.1)
    d.takeaway("The execution layer supplies links; the measurement layer supplies its own observations.")

    d.base("03 / UNITS", "Three clocks answer different questions", "The illustrative values need not move together. Elapsed acceptance time measures from enrolment to first acceptance, with the deadline and observation rules. Person-hours sum human contributions across people and included roles. Tool runtime is process elapsed time. A tool may run while nobody attends it, while a person works on something else, or while multiple people collaborate. Do not subtract machine time from elapsed time and label the remainder human labour. The protocol explicitly decides whether attended waiting counts.", ["P", "A"], "Illustrative units")
    d.cards([("Elapsed acceptance", "2 days\nCalendar duration from enrolment to acceptance"), ("Human labour", "5 person-hours\nTime allocated by people to the included work"), ("Tool execution", "20 minutes\nElapsed time inside the wrapped process")], h=3.4)
    d.takeaway("Always name the quantity and its unit before comparing numbers.")

    d.base("03 / CAPACITY", "One person has one hour of capacity per hour", "Pairing consumes time from two people, so two people pairing for one hour contribute two person-hours. One person supervising two agents for one elapsed hour has at most one person-hour to allocate across the included tasks. This is an attention-capacity assumption, not a way to infer the fractions from agent activity. All included roles—specification, prompting, implementation, review, testing, coordination and repair—need the frozen accounting convention. Fractions describe interval-average allocation, not necessarily constant attention at every instant.", ["P", "L"], "Illustrative capacity arithmetic")
    d.text(0.7, 2.25, 5.5, 0.6, "Two people pairing", 26, TEAL, True)
    for y in [3.2, 4.15]:
        d.rect(0.8, y, 4.65, 0.58, TEAL)
        d.text(1.0, y + 0.13, 4.25, 0.4, "1 hour allocated", 21, WHITE)
    d.text(0.8, 5.1, 5.1, 0.6, "1 + 1 = 2 person-hours", 27, TEAL, True)
    d.text(7.0, 2.25, 5.5, 0.6, "One person, two tasks", 26, BLUE, True)
    d.rect(7.05, 3.2, 2.35, 0.9, BLUE)
    d.rect(9.4, 3.2, 2.35, 0.9, ORANGE)
    d.text(7.2, 3.45, 2.0, 0.4, "Task A", 21, WHITE)
    d.text(9.55, 3.45, 2.0, 0.4, "Task B", 21, WHITE)
    d.text(7.05, 4.48, 5.0, 1.05, "Combined allocation\n≤ 1 person-hour", 27, BLUE, True)

    d.base("03 / HORIZONS", "Freeze the deadlines before observing outcomes", "This slide uses the repository's synthetic 30-day acceptance and 30-day retention-follow-up choices. They are protocol parameters, not universal defaults for every study. An acceptance on day 2 is assessed for retention on day 32; cohort labour continues through day 60. Acceptance on day 40 misses the day-30 output deadline, but its applicable labour still counts through day 60. Recording acceptance, assessing retention and closing the labour window are separate operations. Late or abandoned work must not disappear from the denominator.", ["P"], "Illustrative protocol: 30 + 30 days")
    x0, width = 1.0, 11.3
    d.rect(x0, 3.15, width, 0.05, LINE)
    for day, label, color, yy in [(0, "Enrol", NAVY, 2.25), (2, "Accept", TEAL, 3.65), (30, "Acceptance\ndeadline", ORANGE, 2.05), (32, "Retention\nassessment", TEAL, 3.65), (60, "Labour\nwindow ends", NAVY, 2.05)]:
        x = x0 + day / 60 * width
        d.rect(x, 2.99, 0.045, 0.4, color)
        left = max(0.6, min(x - 0.55, 10.9))
        d.text(left, yy, 1.85, 1.1, f"{label}\nDay {day}", 16, color, True)
    d.takeaway("Early acceptance does not shorten the cohort's labour window.")

    d.base("03 / OUTPUT", "Count retained requirement credit, then divide", "These are illustrative direct observations, not the repository's mixed-method synthetic demo and not the pilot's empirical results. Assume three eligible tasks, complete direct labour observation, closed horizons, no material scope changes and a frozen score of one per task. Task A is accepted on time with retained fraction 0.75; B is permanently abandoned; C is accepted late. Their output totals 0.75 reference units and their labour totals five person-hours. The cohort ratio is 0.15 task-equivalent units per person-hour. A retained fraction is an assessment against the rubric, not a probability or a calibrated customer-value score. Use the ratio of totals, not an average of task ratios.", ["P"], "Illustrative complete direct observations")
    d.table(["Task", "Deadline outcome", "Retained credit", "Human labour"], [["A", "Accepted on time; 75% retained", "0.75 units", "2 h"], ["B", "Permanently abandoned", "0", "2 h"], ["C", "Accepted after the deadline", "0", "1 h"]], [1.1, 5.4, 3.0, 2.72], row_h=0.67, fontsize=19)
    d.text(0.8, 5.25, 11.8, 0.8, "0.75 units ÷ 5 person-hours = 0.15 units/person-hour", 26, TEAL, True)
    d.takeaway("Report both totals. Include the labour of unsuccessful work.", y=6.3)

    d.base("03 / INTERPRETATION", "Zero, unknown and pending are different states", "A zero requires the relevant observation and rule: for example, permanent abandonment establishes zero deadline output, whereas missing follow-up for a timely accepted task leaves retained output unknown. Pending means the required horizon has not yet arrived. A complete direct frame can attest zero labour; missing coverage cannot. The default scope-change policy can remove protocol output credit after a material change by the fixed labour end, while preserving the observed acceptance event and applicable labour. The alternate baseline-only rule follows original scope explicitly. These endpoint rules must be fixed before outcomes are inspected.", ["P"])
    d.cards([("Observed zero", "Evidence and the frozen rule establish no counted contribution."), ("Unknown", "The required observation or coverage is missing."), ("Pending", "The observation window or follow-up has not yet closed.")], h=2.8)
    d.text(0.65, 5.65, 12.0, 0.83, "Scope changes also affect protocol credit. Freeze that rule before intake and keep all applicable labour.", 20, ORANGE, True)

    d.base("04 / BOUNDS", "An interval total may leave its timing unknown", "Here is the smallest useful bound example. We know one person devoted exactly 40 minutes to the target work somewhere in a 60-minute interval. We ask how much occurred in the first 30 minutes, without assuming attention was spread uniformly. At most 30 minutes can fit outside the target, so at least 10 must be inside. At most 30 can fit inside, so the upper endpoint is 30. Both endpoint schedules are possible under allocation between zero and one, so the range is sharp for this datum alone. Multiplying 40 by one half to get 20 would add an unsupported within-interval assumption.", ["P", "L"], "Illustrative exact interval observation", "Known: 40 minutes of human work somewhere in one hour. Target: the first half-hour.")
    for row, (label, first, second) in enumerate([("Minimum inside", 10, 30), ("Maximum inside", 30, 10)]):
        y = 2.85 + row * 1.2
        d.text(0.65, y + 0.1, 2.55, 0.8, label, 21, bold=True)
        d.rect(3.45, y, 4.3, 0.68, PALE)
        d.rect(7.75, y, 4.3, 0.68, PALEORANGE)
        d.rect(3.45, y, 4.3 * first / 30, 0.68, TEAL)
        d.rect(7.75, y, 4.3 * second / 30, 0.68, ORANGE)
        d.text(3.58, y + 0.17, 4.0, 0.35, f"{first} min", 18, WHITE, True)
        d.text(7.88, y + 0.17, 4.0, 0.35, f"{second} min", 18, WHITE, True)
    d.text(3.45, 5.42, 4.3, 0.4, "First 30 minutes", 18, TEAL)
    d.text(7.75, 5.42, 4.3, 0.4, "Second 30 minutes", 18, ORANGE)
    d.takeaway("The justified answer is 10–30 minutes, not an invented exact timing.")

    d.base("04 / SHARED CAPACITY", "Shared capacity can tighten the answer", "Use the existing exact joint-labour example. One person spends exactly one hour on task A and one hour on task B within the same two-hour interval. Each task alone could contribute between zero and one hour to the first hour, so summing the marginal ranges would give zero to two. That sum ignores shared capacity. Two hours of total work exhaust the person's two-hour capacity; neither hour can contain more than one person-hour, so each hour must contain exactly one combined person-hour. The coloured schedules are two compatible witnesses, not reconstructed actual histories. The solver splits time at every boundary, constrains cell totals by observations and shared capacity, then minimises and maximises the target. Incompatible constraints yield an inconsistent result, with no bounds. It returns cell allocations and rechecks constraints with exact rational arithmetic.", ["L", "J"], "Repository synthetic example, verified")
    d.text(0.65, 2.27, 12.0, 0.65, "One person. Two hours. Task A uses 1 h; task B uses 1 h.", 25, bold=True)
    for row, blocks in enumerate([[("A", TEAL), ("B", ORANGE)], [("B", ORANGE), ("A", TEAL)]]):
        y = 3.15 + row * 1.05
        d.text(0.7, y + 0.15, 2.8, 0.5, f"Possible history {row + 1}", 20)
        for i, (label, color) in enumerate(blocks):
            x = 3.65 + i * 4.05
            d.rect(x, y, 3.95, 0.68, color)
            d.text(x + 0.15, y + 0.17, 3.65, 0.4, f"Task {label}: 1 h", 21, WHITE, True)
    d.text(3.65, 5.37, 3.95, 0.4, "First hour", 18, TEAL)
    d.text(7.7, 5.37, 3.95, 0.4, "Second hour", 18, ORANGE)
    d.takeaway("Combined first-hour labour is exactly 1 h. The separate 0–2 h range is too wide.")

    d.base("04 / SAMPLING", "Use a random sample to correct frozen predictions", "This is a manufactured four-unit example, in minutes. The four units are nonoverlapping 30-minute person-intervals. Each has a frozen prediction of ten minutes of included labour, giving forty minutes across the frame. Select two of four units by simple random sampling without replacement, so each has inclusion probability one half. Each sampled error therefore represents two units on average; this is why the correction is multiplied by two. Suppose the observed contributions are fourteen and eight minutes. Their prediction errors are plus four and minus two. Expand each error by two: plus eight minus four. Add the net correction of four to the prediction total, giving a difference estimate of forty-four minutes. It is a frame-total estimate, not a known true total. The frame, selection and accurate labels are substantive assumptions. The next slide explains the uncertainty.", ["E"], "Illustrative probability sample", "Randomly select 2 of 4 intervals without replacement: each has a ½ chance.")
    d.table(["Unit", "Frozen prediction", "Observed in sample", "Error"], [["1", "10 min", "14 min", "+4 min"], ["2", "10 min", "8 min", "−2 min"], ["3", "10 min", "Not selected", "Unknown"], ["4", "10 min", "Not selected", "Unknown"]], [1.2, 3.3, 4.0, 3.72], row_h=0.62, fontsize=19)
    d.text(0.75, 5.72, 12.0, 0.75, "40 + (2 × 4) + (2 × −2) = 44 minutes estimated", 27, TEAL, True)

    d.base("04 / UNCERTAINTY", "A bound and a sampling estimate answer different questions", "A deterministic bound lists values compatible with declared observations and capacity assumptions. It is not a confidence interval. A sampling estimate uses a genuine selection design to estimate a finite-frame total, with sampling uncertainty where the design supports a variance calculation. In the four-unit example, residuals are four and minus two minutes; their sample variance is eighteen square minutes. The supported simple-random-sampling formula gives estimated total variance seventy-two square minutes, or standard error about 8.49 minutes. Standard error estimates the sample-to-sample variation in the total estimate. It is not an estimate of the remaining physical capacity, and this is not a ready-made 95% interval. General designs in the implementation do not receive an invented variance. No method repairs missing responses, inaccurate labels or work outside the declared frame.", ["L", "E"])
    d.cards([("Deterministic bounds", "Which totals are possible under these observations and constraints?\nExample: 10–30 min"), ("Sampling estimate", "What does the sample estimate about the whole declared frame?\n44 min; standard error ≈ 8.49 min")], colors=[TEAL, BLUE], h=3.15)
    d.takeaway("Uncertainty has a source. Name it, and retain the assumptions that support it.")

    d.base("05 / FORECASTING", "A separate model forecasts merged-PR counts", "Return to the third question from the opening map. This is a retained observed example, not a new live forecast. On 30 October 2026, complete daily bins from 24–29 October were 17, 11, 13, 12, 8 and 5 combined merges across EngArg and EAL. The expanding empirical method assigns one sixth probability to each distinct observed count. Its inverse-CDF median is eleven, not the conventional midpoint median of 11.5. Its mean is also eleven. Partial days are excluded, complete zero bins must be supported, and unknown periods are not filled with zeros. Forecasting has no requirement for task labels or labour diaries because it targets PR counts, not labour or functionality. Using these past days as a reference assumes they remain relevant to the next day: launch bursts, policy changes and seasonality can break that assumption.", ["F", "D"], "Observed public history: 24–29 October 2026", "Each complete historical day supplies one reference outcome.")
    counts = [17, 11, 13, 12, 8, 5]
    baseline, max_h = 5.55, 2.7
    d.text(0.63, 2.4, 1.4, 0.5, "Merged PRs", 15, MUTED)
    for i, value in enumerate(counts):
        x = 1.45 + i * 1.2
        h = max_h * value / 18
        d.rect(x, baseline - h, 0.72, h, BLUE)
        d.text(x, baseline - h - 0.4, 0.72, 0.3, value, 18, BLUE, True, align=PP_ALIGN.CENTER)
        d.text(x - 0.12, baseline + 0.16, 1.0, 0.35, f"Oct {24+i}", 15, MUTED, align=PP_ALIGN.CENTER)
    d.rect(1.25, baseline, 7.45, 0.018, MUTED)
    d.text(1.0, baseline - 0.2, 0.3, 0.3, "0", 14, MUTED)
    d.text(9.2, 2.8, 3.0, 0.9, "11", 58, TEAL, True)
    d.text(9.2, 3.85, 3.0, 1.45, "PRs at the\nempirical median", 24)
    d.text(9.2, 5.05, 3.05, 0.9, "Assumption: past days remain relevant.", 18, MUTED)
    d.takeaway("This is a reference distribution—not a promise of eleven merges.")

    d.base("05 / VALIDATION", "Preserve the awkward results", "The retained forecast case reports adverse checks, and the presentation must keep them. The empirical tenth-to-ninetieth percentile endpoints are five and seventeen, but that inclusive range contains all training mass because the distribution is discrete and small. It does not establish eighty-percent future coverage. At the two retrospective origins, the ranges missed both outcomes; mean absolute error of the medians was 5.5 PRs. Mean ranked probability score was 4.83125 versus 3.5 for the previous-day point comparator, with lower being better. Two retrospective origins cannot establish skill or justify promoting a comparator. There were zero complete Monday-based weeks, so weekly forecasting was unavailable, with four required.", ["D", "F"], "Retained public case; retrospective checks")
    d.cards([("Daily range: 5–17", "Contains 100% of the training mass.\nFuture coverage is unvalidated."), ("Earlier range hits: 0/2", "Both reconstructed outcomes missed their ranges.\nKeep this adverse evidence."), ("Weekly forecast", "Unavailable: zero complete weeks.\nFour complete weeks are required.")], h=3.45)
    d.takeaway("Do not multiply a daily prediction by seven to bypass missing weekly history.")

    d.base("06 / CLAIMS", "More evidence does not automatically identify an AI effect", "Execution receipts, descriptive labour accounting and forecasting each supply bounded evidence. A causal question asks what would have happened under another policy or treatment. That requires a comparison or assignment design, reliable outcomes and assumptions about dependencies and interference. EngArg and EAL are coupled components; two repositories do not create independent experimental units. A different GitHub actor or a workflow containing AI is not a treatment assignment. A successful receipt pipeline validates the instrumentation path, not an AI productivity effect or staffing rule.", ["P", "A", "X"])
    d.cards([("Observed", "This tool completed.\nThis person-time was observed.\nThis requirement was assessed."), ("Descriptive", "The cohort had this output and labour.\nThe historical counts had this distribution."), ("Causal", "Would another policy have changed the outcome?\nRequires a separate design.")], h=3.5)
    d.takeaway("Match the strength of the claim to the observations and study design.")

    d.base("06 / THE PILOT", "The public pilot verified the evidence path", "These are observed verification results retained by PR #9. The pilot replayed existing pinned EngArg and EAL checks inside Observatory's own workflow; it did not instrument the original repositories' workflows. The direct collection run produced four lifecycle receipts for two invocations, each matched to its corresponding completed matrix job. Repeating collection left receipt identity and content unchanged. Twenty-nine retained records include API and coverage records; that is not twenty-nine tool calls. The downloaded store and snapshot assertions replayed locally. The live pilot used ordinary checks, while Codex validation used fixtures and verified upstream event definitions. Neither pilot test counts nor runtimes measure engineering productivity.", ["T", "X"], "Observed live pilot: run 41329655162")
    for i, (value, label) in enumerate([("2", "tool invocations"), ("4", "start/completion receipts"), ("2", "matched matrix jobs")]):
        x = 0.8 + i * 4.15
        d.text(x, 2.5, 3.7, 1.2, value, 72, [TEAL, BLUE, ORANGE][i], True)
        d.text(x, 3.85, 3.7, 0.85, label, 23, bold=True)
    d.text(0.85, 5.2, 11.8, 0.7, "Repeated collection: unchanged receipts. Retained store: 29 verified records.", 24)
    d.takeaway("The pilot supports instrumentation and replay—not a productivity claim.")

    d.base("06 / ADOPTION", "Adopt one explicit boundary at a time", "Start with a selected tool command and a named job, not an ambition to capture everything implicitly. Configure the action in each instrumented job, use the explicit wrapper and publish under an always condition. Establish scheduled durable collection before artifact retention expires. Add an independent workflow_run observer from trusted code for completion metadata. Verify schema failures, missing uploads, retries, matrix identity and reruns before interpreting a larger rollout. Human work and quality observations belong to their own protocol, linked through stable task and session identifiers.", ["X", "W"])
    d.cards([("Instrument", "Setup → explicit wrapper → publish\nOne bundle per job; known metadata only"), ("Retain and reconcile", "Collect before expiry\nValidate, deduplicate and verify\nCompare independent jobs"), ("Measure deliberately", "Freeze the study protocol\nObserve people and outcomes\nLink evidence to requirements")], h=3.5)
    d.takeaway("Start narrow, inspect the gaps, and expand only after the evidence survives replay.")

    d.base("06 / DISCUSSION", "Which claim can we defend?", "Pause and invite a reason for each answer. A: yes, within scope, because an observed successful completion and independently matched job support the wrapped-process claim; they do not establish total tool coverage. B: no, because tool runtime does not measure human attention, and a saved-work comparison also needs a valid counterfactual. C: no, because unknown reported usage is not a measured zero. D: only conditionally: the joint bound is exact under the declared observations, streams and per-person capacity assumptions; those inputs still need evidence. Use the answers to return to the opening three-question map.", ["X", "P", "L"], "Audience exercise; answers in speaker notes")
    d.table(["Claim", "What evidence would it need?"], [
        ["A  The wrapped test completed successfully.", "Completion receipt + supported job linkage"],
        ["B  This 20-minute run saved 20 minutes of human work.", "Human observations + a justified comparison"],
        ["C  No tokens were reported, so no tokens were used.", "An actual zero-usage observation"],
        ["D  Combined first-hour labour is exactly one hour.", "The stated totals + shared capacity assumptions"],
    ], [7.25, 4.97], y=2.2, row_h=0.9, fontsize=18)

    d.base("06 / TAKE WITH YOU", "Make the evidence strong enough for the claim", "Close by returning to the practical engineering discipline rather than promising a universal score. Capture what the system actually observes. Keep identity, provenance and missingness explicit. Choose the measurement model for the question: tool events, human labour and retained output, or future PR counts. Check its assumptions and preserve adverse evidence. The benefit is a defensible chain from evidence to a bounded conclusion that another engineer can inspect and replay. Point readers to the appendix for equations, field names and source contracts.", ["A", "X", "P", "F"], dark=True)
    d.text(0.7, 2.55, 11.8, 2.3, "Collect what happened.\nState how you know.\nKeep the unknowns visible.", 38, WHITE, True)
    d.text(0.72, 5.35, 11.8, 0.9, "Evidence → explicit assumptions → a bounded, reproducible conclusion", 25, "A5E2D8")

    # Reference slides: deliberately separate from the teaching sequence.
    d.base("APPENDIX / FIELD MAP", "Receipt fields and their provenance", "This reference slide names the concrete schema groups. Platform values are copied from the runner environment, while job name, optional ID and matrix are caller supplied and separately labelled. Independent API comparison corroborates only supported fields; it does not verify every platform value, such as workflow_sha. Declared controller, protocol and session fields are not inferred from GitHub. Tool labels, purpose and configuration are declarations. Result values describe wrapper observations and recognised tool reports, with missing fields retained. Hashes describe designated artifact bytes, not their semantic quality. See the schema for types, enum values, constraints and nullable fields.", ["R", "X"])
    d.table(["Group", "Key fields", "Basis"], [
        ["platform", "repository_id; workflow_sha; run_id; run_attempt; actors; event", "Runner; selected fields corroborated"],
        ["job context", "job_key; job_name; job_id; matrix", "Key: runner; name/ID/matrix: caller"],
        ["declared", "controller; provenance; protocol/task/session; logical/parent IDs", "Caller declaration"],
        ["tool", "adapter; name/version; purpose; configured_model; configuration", "Supplied labels/configuration"],
        ["result", "status; exit_code; runtime; observed model/session; tokens; charges", "Wrapper / recognised tool report"],
        ["artifacts / missing", "role; label; SHA-256; size; missing_fields", "Designated bytes / recorded absence"],
    ], [1.7, 6.6, 3.92], row_h=0.63, fontsize=14.5)

    d.base("APPENDIX / INVARIANTS", "The implementation rejects ambiguous substitutions", "This is an operational reference for reviewers and implementers. The JSON or ZIP bundle is bounded at sixteen MiB and contains at most ten thousand receipt entries, each at most 256 KiB. The ZIP must contain exactly one bundle.json file; directories and symlinks are rejected. Unknown schema fields and duplicate JSON keys are rejected; hashes are checked over exact receipt JSON bytes. Same-phase content changes under a stable identity fail. Phase-invariant platform, declared and tool identity must agree, and completion cannot precede start. Incomplete phases remain admissible evidence. During collection, cross-delivery conflicts are quarantined without erasing independently retrieved job metadata. Direct import rejects conflicting receipts. Numeric limits bound the implementation; they are not completeness claims.", ["R", "X"])
    d.cards([("Immutable identity", "Same phase + same identity → same content\nRetry → fresh invocation ID"), ("Validated structure", "Versioned schemas\nExact receipt hashes\nStrict JSON and bounded ZIP"), ("Preserved failures", "Missing phase stays missing\nCollection quarantines conflicts\nIndependent job evidence survives")], h=3.3)
    d.takeaway("16 MiB bundle • 10,000 receipts • 256 KiB per receipt • one bundle.json ZIP member")

    d.base("APPENDIX / ARITHMETIC", "The small set of equations behind the pictures", "Translate each expression aloud. Allocation times duration gives one person's contribution when the required within-window allocation is observed. Retained credit follows the frozen score, timely-acceptance rule and observed retained fraction, plus the prespecified scope policy. Under the default exclusion rule, a material scope change can set protocol credit to zero. Cohort productivity divides summed output by summed labour, with positive labour required. The interval bound moves known mass outside or inside the target as far as capacity allows. These are conditional accounting or identification statements, not equations that infer unobserved people from repository activity.", ["P", "L"])
    d.table(["Question", "Rule", "Read it as"], [
        ["Human contribution", "duration × allocation", "Time assigned to the included work"],
        ["Timely credit, eligible scope", "frozen score × retained fraction", "Credit under the frozen rubric and scope rule"],
        ["Cohort productivity", "sum(output) ÷ sum(labour)", "A ratio of totals; labour must be positive"],
        ["Clipped interval", "[max(0, m − v), min(m, u)]", "Known mass m; u inside, v outside"],
    ], [2.65, 4.45, 5.12], row_h=0.82, fontsize=17)
    d.takeaway("For known Q ≥ 0 and 0 < Hmin ≤ H ≤ Hmax: the conditional ratio range is Q/Hmax to Q/Hmin.", y=6.38)

    d.base("APPENDIX / ESTIMATION", "Correct prediction errors with the actual sampling design", "Here M is the number of units in a complete declared frame; n is the sample size for simple random sampling without replacement. For each unit, f is a prediction frozen before evaluation sampling, y is accurately observed labour, and pi is its genuine inclusion probability. The difference estimate sums all predictions and adds weighted observed residuals. Under the stated design assumptions it is design-unbiased across repeated samples. That does not make a realised estimate exact or force it into the physical range. The repository's v2 synthetic example gives 5.5 hours despite five hours of declared frame capacity; clipping changes the estimator. The command preserves signed results and reports variance only when supported. It constructs neither ratio confidence intervals nor causal effects.", ["E"])
    d.rect(0.7, 2.35, 11.9, 1.05, PALE)
    d.text(0.95, 2.62, 11.4, 0.65, "Estimate = prediction total + sum(error ÷ selection chance)", 24, TEAL, True)
    d.cards([("Required", "Complete declared frame\nGenuine selection probabilities\nAccurate sampled labels\nPredictions frozen in advance"), ("What stays explicit", "Nonresponse is rejected\nMissing frame work is not repaired\nVariance depends on the design\nAn estimate is not exact interval mass")], y=3.75, h=2.75, colors=[TEAL, BLUE])

    d.base("APPENDIX / FORECAST RULES", "The forecast is a reproducible reference, not a capacity model", "The automatic model forms complete UTC bins for the chosen horizon and portfolio, summing repositories before fitting. At least four complete target-width bins are required. The expanding empirical method uses all supported complete bins and reports discrete empirical quantiles. Its relevance assumption is that past complete periods provide a useful reference for the next period; launch bursts, policy changes or seasonality may violate it. Forecast bundles are immutable and bound to an evidence snapshot. A later successful collection must cover the entire target period before an original forecast can be scored; merely taking a newer snapshot of stale sources does not extend observation. Retrospective reconstructions are not original issuance receipts. Queue counts describe current visible inventory, not causal bottlenecks or a maximum future merge count. Preserve the adverse two-origin results rather than selecting only a flattering statistic.", ["F", "D"])
    d.cards([("Build the reference", "Complete UTC bins\nFour bins minimum\nPortfolio counts summed first\nNo daily-to-weekly scaling"), ("Score the original", "Keep the issued distribution\nCollect through the full target\nPreserve losses and misses\nDistinguish replay from issuance"), ("Keep its scope", "Merged PR count only\nNo labour or staffing inference\nNo causal queue diagnosis\nFuture skill remains unestablished")], h=4.0)

    for title, keys in [("Sources for execution evidence", ["X", "R", "T", "G", "W", "A"]), ("Sources for measurement and forecasting", ["P", "L", "J", "E", "F", "D"])]:
        d.base("APPENDIX / SOURCES", title, "Each title on this slide is a hyperlink. Repository sources are pinned to the implementation revision recorded in the deck manifest. The external GitHub documentation describes platform semantics and may evolve. All illustrative examples are explicitly labelled; observed pilot and forecast examples retain their source scope and dates. The editable PowerPoint, PDF handout, full speaker notes and build source are stored together in the PR.", keys)
        for i, key in enumerate(keys):
            name, url = SOURCES[key]
            y = 2.28 + i * 0.65
            d.text(0.7, y, 0.55, 0.4, f"[{key}]", 19, TEAL, True)
            d.text(1.45, y, 10.9, 0.52, name, 22, INK, link=url)
        d.text(0.7, 6.45, 11.8, 0.32, "Content baseline: PR #9 implementation e252eae • 1 November 2026", 13, MUTED)
    return d


def check_facts():
    """Fail the build if the retained numerical examples no longer match the deck."""
    pilot = json.loads((REPO / "docs/execution-pilot/github-collection-verification.json").read_text())
    assert pilot["receipt_records"] == 4 and pilot["completed_invocations"] == 2
    assert pilot["repeated_delivery_receipts_unchanged"] is True
    assert len(pilot["linked_job_ids"]) == 2
    with (REPO / "docs/use-cases/engarg-eal-daily-2026-10-30/daily-counts.csv").open() as handle:
        rows = list(csv.DictReader(handle))
    expected = [17, 11, 13, 12, 8, 5]
    numeric_columns = [[row[k] for row in rows] for k in rows[0]]
    assert [str(x) for x in expected] in numeric_columns
    assert sum(expected) == 66 and sum(expected) / len(expected) == 11
    assert max(0, 40-30) == 10 and min(40, 30) == 30
    assert 40 + 2 * 4 + 2 * -2 == 44
    assert 4**2 * (1 - 2/4) * 18 / 2 == 72
    assert 0.75 / (2+2+1) == 0.15


def preflight(deck, path, output):
    reopened = Presentation(path)
    issues = []
    texts = 0
    for n, slide in enumerate(reopened.slides, 1):
        assert slide.has_notes_slide and len(slide.notes_slide.notes_text_frame.text) > 200
        for shape in slide.shapes:
            if shape.left < 0 or shape.top < 0 or shape.left + shape.width > reopened.slide_width + 3 or shape.top + shape.height > reopened.slide_height + 3:
                issues.append({"slide": n, "shape": shape.shape_id, "problem": "outside slide bounds"})
            if shape.has_text_frame and shape.text.strip():
                texts += 1
    assert not issues, issues
    result = {"source_revision": REVISION, "slide_count": len(reopened.slides), "native_text_shapes": texts,
              "all_slides_have_speaker_notes": True, "geometry_bounds_pass": True,
              "fact_checks_pass": True, "pptx_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
              "render_review": "See README for the performed PDF and visual checks; XML checks alone do not prove layout quality."}
    (output / "build-validation.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=HERE)
    args = parser.parse_args()
    check_facts()
    deck = build()
    path = deck.save(args.output_dir)
    preflight(deck, path, args.output_dir)
