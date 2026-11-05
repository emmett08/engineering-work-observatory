# Automatic throughput forecasting

This companion article specifies an automatic model for forecasting how many pull requests (PRs) will merge in a fixed set of repositories visible through GitHub. The labour-identification manuscript at `paper/article/main.tex` addresses separate measurement questions about human work.

Build with `make -C paper/forecasting`. The output is `automatic-throughput-forecasting.pdf` in this directory. Intermediate LaTeX files go to `build/`.

The default model assigns probabilities according to how often each merge count occurred in completed calendar periods meeting the collection requirements. The recent-window challenger uses up to eight daily or weekly observations, or four fortnightly observations. It failed the predeclared ranked probability score (RPS) criterion for replacing the default in the public-repository comparison and remains experimental. RPS compares the predicted probabilities with the observed count, with lower scores indicating better predictions. Daily forecasts require four complete days, weekly forecasts require four complete weeks, and fortnightly forecasts require eight complete weeks. Collection requires no manual labels describing labour, AI use, task categories or acceptance. The accompanying counts and ages describe currently open PRs. Establishing which resources constrain the work requires additional evidence.

The default uses all eligible history, while the public study's expanding comparator uses only the specified 52–77 weeks of training history. The recent-window challenger's failure to improve on that comparator leaves the predictive accuracy of the full-history default unestablished. User histories are too short to establish accuracy or comparative superiority, and results from the public repository describe a separate population.

The user tables are **calculations made later from an earlier saved collection of records**. Their source cutoff states which history each calculation uses. It provides no evidence that a forecast was produced at that earlier time, and the reconstructions contain no scores from completed target periods. The separately saved [original `forecast-team` example](examples/README.md) was created at its locally declared time on 29 October 2026 at 22:18:19 UTC for 30–31 October. It uses a different source snapshot, although the five training counts and numerical prediction agree. Its score remains pending. Creation and issuance times are locally declared and lack independent verification.

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
