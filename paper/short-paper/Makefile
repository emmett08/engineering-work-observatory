LATEXMK ?= latexmk
PDFINFO ?= pdfinfo

.PHONY: all check clean

all:
	$(LATEXMK) -pdf -interaction=nonstopmode -halt-on-error -file-line-error -outdir=build main.tex

check: all
	@$(PDFINFO) build/main.pdf | awk '/^Pages:/ { pages = $$2 } END { if (pages > 0 && pages <= 12) { print pages " pages"; exit 0 } exit 1 }'
	@! grep -E 'Overfull|undefined references|Citation .* undefined|Reference .* undefined' build/main.log

clean:
	$(LATEXMK) -C -outdir=build main.tex
