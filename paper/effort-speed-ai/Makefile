PYTHON ?= python3

.PHONY: check article lean simulate figures

check:
	$(PYTHON) check_package.py
	cd formal && $(PYTHON) scripts/check_inventory.py
	cd formal && $(PYTHON) scripts/validate_formal_receipt.py formal-verification-receipt.json --base .

article:
	cd article && latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=../.build/article main.tex

lean:
	cd formal && lake build
	cd formal && lake env lean AxiomAudit.lean

simulate:
	mkdir -p .build/synthetic
	cp simulate.py .build/synthetic/simulate.py
	OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 MKL_NUM_THREADS=1 $(PYTHON) .build/synthetic/simulate.py

figures:
	mkdir -p .build/figures/evidence
	cp visualise.py .build/figures/visualise.py
	cp evidence/results.json .build/figures/evidence/results.json
	$(PYTHON) .build/figures/visualise.py
