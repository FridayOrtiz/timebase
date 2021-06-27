#!/usr/bin/env bash

pandoc --metadata=title:"Executive Summary: NTP-based Data Infiltration Channels" \
  --pdf-engine=xelatex \
  --variable monofont="DejaVu Sans Mono" \
  --from=markdown+abbreviations \
  --output=executive_summary.pdf \
  executive_summary.md
