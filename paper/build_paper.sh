#!/usr/bin/env bash

pandoc --metadata=title:"A novel network hopping covert channel using BPF filters and NTP extension fields" \
  --from=markdown+abbreviations \
  --bibliography=paper.bib \
  --csl=ieee.csl \
  --citeproc \
  --number-sections \
  --output=paper.pdf \
  paper.md

# `--citeproc` may be necessary on newer versions of pandoc
#  --trace \
#  --verbose \
