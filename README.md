# swissmedicinfo
Helper for Swissmedic Info

## Output File Summary

| Option(s) | Filename | Format | Content |
|-----------|----------|--------|---------|
| (default) | `swissmedicinfo_DD.MM.YYYY.csv` | CSV | All records, identifier + date |
| `--larger N` | `larger_N_DD.MM.YYYY.csv` | CSV | Unique IDs > N, most recent date |
| `--today` | `today` | Text | Today's unique IDs, 5 digits, remote copy |
| `--local` | `today` | Text | Today's unique IDs, 5 digits, local copy |

# Download latest XML and process all records
cargo run --release -- --download

# Download and filter by date (records since DD.MM.YYYY)
cargo run --release -- --download --since 01.01.2025

# Download and filter by identifier threshold (IDs > N)
cargo run --release -- --download --larger 5000

# Download with both date and threshold filters
cargo run --release -- --download --since 01.01.2025 --larger 5000

# Download and extract today's unique IDs, copy to remote server
cargo run --release -- --download --today

# Download and extract today's unique IDs, copy to local directory
cargo run --release -- --download --local

# Process local XML file (all records)
cargo run --release -- AipsDownload_20260130.xml

# Process local XML with date filter
cargo run --release -- AipsDownload_20260130.xml --since 01.01.2025

# Process local XML with threshold filter
cargo run --release -- AipsDownload_20260130.xml --larger 5000

# Process local XML with both filters
cargo run --release -- AipsDownload_20260130.xml --since 01.01.2025 --larger 5000

# Process local XML for today's IDs (remote)
cargo run --release -- AipsDownload_20260130.xml --today

# Process local XML for today's IDs (local)
cargo run --release -- AipsDownload_20260130.xml --local
