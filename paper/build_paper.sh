#!/usr/bin/env bash

pandoc --metadata=title:"BPF traffic control filter for implementing a NTP covert channel." \
  --from=markdown+abbreviations \
  --output=paper.pdf \
  --bibliography=paper.bib --csl=ieee.csl \
  --citeproc --number-sections \
  paper.md

# `--citeproc` may be necessary on newer versions of pandoc
