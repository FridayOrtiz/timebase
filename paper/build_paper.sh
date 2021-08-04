#!/usr/bin/env bash

pandoc --metadata=title:"Title Goes Here" \
  --from=markdown+abbreviations \
  --output=paper.pdf \
  --bibliography=paper.bib --csl=ieee.csl \
  --pdf-engine=xelatex \
  paper.md

# `--citeproc` may be necessary on newer versions of pandoc