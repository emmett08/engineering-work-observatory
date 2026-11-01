PYTHON ?= python3
LATEXMK ?= latexmk

.PHONY: all article figure forecasting baseline check performance clean

all: article

article:
	cd article && $(LATEXMK) -pdf -interaction=nonstopmode -halt-on-error -file-line-error -outdir=../build main.tex
	cp build/main.pdf ai-assisted-software-engineering.pdf

figure:
	$(PYTHON) figures/shared-capacity/calculate.py --check
	$(PYTHON) figures/shared-capacity/render.py --source figures/shared-capacity/tikz.tex --output figures/shared-capacity/figure.pdf --spec shared-capacity.spec.json --audit figures/shared-capacity/figure.audit.json --preview-dir build/shared-capacity-preview
	$(PYTHON) figures/shared-capacity/verify.py


forecasting:
	$(MAKE) -C forecasting

baseline:
	$(MAKE) -C effort-speed-ai article

check:
	$(MAKE) -C effort-speed-ai check
	$(PYTHON) figures/shared-capacity/verify.py --review
	$(PYTHON) check_measurement_examples.py
	$(PYTHON) check_sharp_identification.py
	$(PYTHON) forecasting/check_reference_results.py --self-test
	cd performance && cargo test --locked
	cd performance && $(PYTHON) -m unittest discover -s tests -v

performance:
	cd performance && $(PYTHON) collect_benchmarks.py

clean:
	cd article && $(LATEXMK) -C -outdir=../build main.tex
