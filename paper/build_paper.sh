#!/usr/bin/env bash

pandoc --metadata=title:"A novel network hopping covert channel using BPF filters and NTP extension fields" \
  --from=markdown+abbreviations \
  --output=paper.pdf \
  --bibliography=paper.bib --csl=ieee.csl \
  --citeproc --number-sections \
  paper.md

# `--citeproc` may be necessary on newer versions of pandoc
