PYTHON ?= python3
LATEXMK ?= latexmk

.PHONY: all article baseline check performance clean

all: article

article:
	cd article && $(LATEXMK) -pdf -interaction=nonstopmode -halt-on-error -file-line-error -outdir=../build main.tex
	cp build/main.pdf ai-assisted-software-engineering.pdf

baseline:
	$(MAKE) -C effort-speed-ai article

check:
	$(MAKE) -C effort-speed-ai check
	cd performance && cargo test --locked

performance:
	cd performance && $(PYTHON) run_benchmarks.py

clean:
	cd article && $(LATEXMK) -C -outdir=../build main.tex
