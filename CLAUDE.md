# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust CLI utility that downloads, parses, and filters medicinal product data (XML) from Swissmedic (Swiss regulatory authority). Single-file application in `src/main.rs` (~500 lines).

## Build and Run Commands

```bash
cargo build --release
cargo run --release -- --download                          # Download latest XML and output CSV
cargo run --release -- --download --since 01.01.2025       # Filter by date (DD.MM.YYYY format)
cargo run --release -- --download --larger 5000            # Filter IDs > threshold
cargo run --release -- --download --today                  # Extract today's unique IDs, scp to remote
cargo run --release -- --download --local                  # Extract today's unique IDs to /var/www/oddb.org/data/txt/today
cargo run --release -- AipsDownload_20260130.xml           # Process existing local XML file
```

There are no tests, lints, or CI configured.

## Architecture

**Download flow:** `download_latest_xml()` fetches https://download.swissmedicinfo.ch/, scrapes ASP.NET ViewState from the HTML form, POSTs to trigger a ZIP download, then extracts the XML from the ZIP.

**XML parsing:** `parse_xml()` uses quick-xml event-driven parsing with a state machine to extract 5-digit identifiers from `RegulatedAuthorization` elements and associated dates (YYYY-MM-DD).

**Filtering and output:**
- `--since` / `--larger` flags filter records by date or ID threshold
- Default output: CSV file (`swissmedicinfo_DD.MM.YYYY.csv`)
- `--today` / `--local`: writes unique IDs for today to a `today` file, optionally copies via `scp`

## Key Dependencies

- `quick-xml` — event-based XML parsing
- `reqwest` (blocking, cookies) — HTTP client for download
- `scraper` — HTML parsing to extract ASP.NET form values
- `csv` — CSV output
- `chrono` — date handling
- `zip` — ZIP extraction
