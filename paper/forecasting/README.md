# Automatic throughput forecasting

This companion article specifies the automatic model for forecasting GitHub-visible PR merge counts in a fixed repository portfolio. The labour-identification manuscript at `paper/article/main.tex` addresses separate measurement questions. Its proofs, labour observations and historical performance claims remain unchanged.

Build with `make -C paper/forecasting`. The output is `automatic-throughput-forecasting.pdf` in this directory. Intermediate LaTeX files go to `build/`.

The default model assigns empirical probabilities using all complete calendar counts eligible under the collection and calendar requirements. The recent-window challenger uses up to eight daily or weekly observations, or four fortnightly observations. It failed the predeclared RPS criterion for replacing the default in the public-repository comparison and remains experimental. Daily forecasts require four complete days, weekly forecasts require four complete weeks, and fortnightly forecasts require eight complete weeks. Collection requires no manual labour, AI-use, task-class or acceptance labels. The accompanying open-inventory counts describe observed queue pressure and leave causal bottlenecks unidentified.

The default uses all eligible history, while the public study's expanding comparator uses its declared 52–77-week training frame. The failed recent-window challenger leaves the predictive accuracy of the default unestablished. User histories are too short to establish accuracy or comparative superiority, and results from the public repository describe a separate population.

The user tables are **offline reconstructions from an earlier retained snapshot**. Their declared source cutoff leaves issuance at that time unestablished, and the reconstructions contain no scores from completed targets. The separately retained [original `forecast-team` example](examples/README.md) was created at its locally declared time on 29 October 2026 at 22:18:19 UTC for 30–31 October. It uses a different source snapshot, although the five training counts and numerical prediction agree. Its score remains pending. Creation and issuance times are locally declared and lack independent certification.

Sources were checked in the full published Gneiting–Raftery (2007) and Gneiting–Balabdaoui–Raftery (2007) PDFs, Little's (2011) original article, the complete 2026 Queue & AI preprint, and official GitHub endpoint documentation. `references.bib` records the exact sources. These sources supply neither a coverage guarantee for the predicted percentile ranges nor a staffing forecast for the monitored repositories.

`generate_results.py` formats retained reconstruction and comparison JSON into the manuscript tables and describes the original command output separately. It checks the shared user snapshot digest, the original example's compressed and uncompressed file hashes, and the comparison's predeclared 78/52/26-week boundaries. It recomputes aggregate scores from retained rows and preserves every fixed comparator. Inputs and their file hashes are retained under `generated/`. From the repository root, regenerate the supplied tables with:

```sh
python paper/forecasting/generate_results.py \
  --user-day paper/forecasting/generated/user-day.json \
  --user-week paper/forecasting/generated/user-week.json \
  --reference paper/forecasting/generated/reference-weekly.json
```

For a new comparison report, also pass `--reference-snapshot path/to/public-snapshot.json`. The generator verifies the stable repository identity and matching snapshot digest, then retains a compact source-metadata record. Subsequent formatting can reuse that record. The full source inventory is retained separately.

The independent [reference checker](check_reference_results.py) uses only the Python standard library and reconstructs all 130 scores from [the compact event ledger](generated/reference-events.json):

```sh
python paper/forecasting/check_reference_results.py --self-test
```

This ledger is **derived from the collected source data**. It retains stable event identities, timestamps and digests linking it to the raw source snapshot. Raw GitHub responses and an archive establishing when historical observations became available remain separate requirements. Hashes detect changed bytes while leaving collection time and historical visibility unverified. Where the full matching raw snapshot is available, check the projection independently with:

```sh
python paper/forecasting/check_reference_results.py \
  --snapshot path/to/public-snapshot.json --self-test
```

The original issued example includes its own public source payloads. Follow [its verification commands](examples/README.md) to check that separate command output and pending score. These checks establish whether the recorded inputs reproduce the forecast. Predictive accuracy requires scoring outcomes after their targets have elapsed.
