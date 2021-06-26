#!/usr/bin/env bash

pandoc --metadata=title:"Title Goes Here" \
  --from=markdown+abbreviations \
  --output=paper.pdf \
  --bibliography=paper.bib --csl=ieee.csl \
  --citeproc \
  paper.md
