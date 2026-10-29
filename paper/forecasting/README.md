# Automatic throughput forecasting

This companion article specifies the operational merge-count model. It is separate from `paper/article/main.tex` and the existing labour-identification PDF. It does not alter or extend their proofs, labour observations or historical performance claims.

Build with `make -C paper/forecasting`. The output is `automatic-throughput-forecasting.pdf` in this directory. Intermediate LaTeX files go to `build/`.

The model forecasts GitHub-visible PR merge counts in a fixed repository portfolio. The operational reference is an empirical distribution over all supported complete calendar counts. The recent-window challenger uses up to eight daily/weekly observations or four fortnightly observations. It failed the predeclared public-reference RPS promotion screen and remains experimental. Daily output needs four complete days; weekly output needs four complete weeks; fortnightly output needs eight complete weeks. No manual labour, AI, task-class or acceptance labels are required. Current open-inventory observations do not identify causal bottlenecks. The deployed full-history reference is not the same observation frame as the bounded public study's expanding comparator, and neither that study nor the default choice validates user-repository accuracy.

The user tables in the publication are **offline replays from an earlier retained snapshot**. Their declared source cut is not evidence that those forecast files existed then. They are not mature outcome scores. The separately retained [original `forecast-team` example](examples/README.md) was created at its locally declared time on 29 October 2026 at 22:18:19 UTC for 30–31 October. Its source snapshot differs from the replay's, although the five training counts and numerical prediction agree. Its supplied score is pending. Receipt times are declarations, not independently trusted timestamps.

Successful software checks remain distinct from predictive skill. Short user histories cannot establish accuracy or comparative superiority. Reference-repository results describe a separate population.

Sources were checked in the full published Gneiting--Raftery (2007) and Gneiting--Balabdaoui--Raftery (2007) PDFs, Little's (2011) original article, the complete 2026 Queue & AI preprint, and official GitHub endpoint documentation. `references.bib` records the exact sources. No prediction-interval coverage guarantee or staffing forecast is inferred from those references.

`generate_results.py` formats retained replay and comparison JSON into the manuscript tables, and describes the original command example separately. It checks the shared user snapshot digest, the original example's compressed/uncompressed file hashes, and the reference exercise's predeclared 78/52/26-week boundaries. It recomputes aggregate scores from retained rows and preserves every fixed comparator. Inputs and their file hashes are retained under `generated/`. From the repository root, regenerate the supplied tables with:

```sh
python paper/forecasting/generate_results.py \
  --user-day paper/forecasting/generated/user-day.json \
  --user-week paper/forecasting/generated/user-week.json \
  --reference paper/forecasting/generated/reference-weekly.json
```

When generating from a new reference report, also pass `--reference-snapshot path/to/public-snapshot.json`. The generator verifies the stable reference repository identity and matching snapshot digest, then retains a compact source-metadata record. Subsequent formatting can reuse that record; the full source inventory is retained separately rather than duplicated in the manuscript directory. Reference results remain separate from evidence about the user repositories.

The independent [reference checker](check_reference_results.py) uses only the Python standard library and reconstructs all 130 scores from [the compact event ledger](generated/reference-events.json), rather than trusting the table summaries:

```sh
python paper/forecasting/check_reference_results.py --self-test
```

This ledger is **derived evidence**, not raw GitHub responses or a historical observed-at archive. It retains stable event identities, timestamps and digests linking it to the raw source snapshot. Hashes detect changed bytes; they do not prove collection time or historical visibility. Where the full matching raw snapshot is available, check the projection independently with:

```sh
python paper/forecasting/check_reference_results.py \
  --snapshot path/to/public-snapshot.json --self-test
```

The original issued example includes its own public source payloads; follow [its verification commands](examples/README.md) to check that separate receipt and pending score. Neither command establishes predictive accuracy from an unelapsed target.
