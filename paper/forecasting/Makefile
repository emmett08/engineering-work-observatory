LATEXMK ?= latexmk

.PHONY: all clean

all:
	$(LATEXMK) -pdf -interaction=nonstopmode -halt-on-error -file-line-error -outdir=build main.tex
	cp build/main.pdf automatic-throughput-forecasting.pdf

clean:
	$(LATEXMK) -C -outdir=build main.tex
