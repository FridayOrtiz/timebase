#!/usr/bin/env bash

pandoc --metadata=title:"Executive Summary: NTP-based Data Infiltration Channels" \
  --from=markdown+abbreviations \
  --output=executive_summary.pdf \
  executive_summary.md
