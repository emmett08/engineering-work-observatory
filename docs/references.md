# References and source status

The project bibliography is [references.bib](../references.bib). It includes the
user-supplied book under the requested key `vanderpost2024datascience`, preserving
the supplied author, title, publisher, year and note.

## Candidate Rust data-science resource

Hayden Van Der Post, *Data Science with Rust: The Speed & Safety Revolution*,
Reactive Publishing, 2024. **Source status: user-supplied; exact edition and
contents not independently verified.**

A search-indexed entry in the [Federal University of Technology Akure computing
library catalogue](https://soclibrary.futa.edu.ng/userstudent/bookdept/6) lists
*Data Science with Rust* and Hayden Van Der Post. It does not establish the
supplied subtitle or year. The publisher field is spelled "Reative Publishing";
this does not independently verify the supplied publisher metadata. Direct access
to the catalogue returned HTTP 403 during the 29 October 2026 check.

No authorised publisher preview or contents for this exact edition was obtained.
Results naming other subtitles or editions were not treated as the same book.
No ISBN, DOI, publisher URL, chapter attribution or library recommendation has
been inferred. The bibliography comment records that boundary without silently
replacing the user's citation.

| Proposed use or claim | Applicable source | Evidence strength | Caveat |
|---|---|---|---|
| Candidate background reading on Rust data science | `vanderpost2024datascience` | User-nominated educational resource | Exact edition and contents remain unverified; suitability needs inspection. |
| Observatory feature priorities | Inspected [report export](../src/report.rs), [analysis](../src/analysis.rs) and [model](../src/model.rs), with the supplied measurement paper | Project-specific engineering rationale | These are proposals, not findings attributed to the book or measured benefits. |
| A Rust CSV writer can serialize ordinary Rust records and handle CSV quoting | Maintainer [csv 1.4.0 Writer documentation](https://docs.rs/csv/1.4.0/csv/struct.Writer.html) | Primary API documentation | A library handles serialization, not the observatory's null/status, provenance or spreadsheet-safety contract. |
| A future Parquet exporter can write Arrow record batches | Apache [Arrow Rust Parquet documentation](https://arrow.apache.org/rust/parquet/arrow/arrow_writer/struct.ArrowWriter.html) | Primary API documentation | This supports technical feasibility, not a performance advantage for this dataset; API versions must be pinned when implementing. |

For implementation decisions, prefer the measurement paper and tested project
contracts, then relevant primary API documentation. Retain this book as a
candidate supplementary learning resource. It is not evidence that a calculation
is statistically justified, an implementation is correct, or AI assistance has a
particular effect. The [roadmap](roadmap.md) gives concrete follow-up work grounded
in current code, with acceptance criteria independent of access to the book.
