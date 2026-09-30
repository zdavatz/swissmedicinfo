# swissmedicinfo
Helper for Swissmedic Info

## Output File Summary

| Option(s) | Filename | Format | Content |
|-----------|----------|--------|---------|
| (default) | `swissmedicinfo_DD.MM.YYYY.csv` | CSV | All records, identifier + date |
| `--larger N` | `larger_N_DD.MM.YYYY.csv` | CSV | Unique IDs > N, most recent date |
| `--today` | `today` | Text | Today's unique IDs, 5 digits, remote copy |
| `--local` | `today` | Text | Today's unique IDs, 5 digits, local copy |

### Download latest XML and process all records
cargo run --release -- --download

### Download and filter by date (records since DD.MM.YYYY)
cargo run --release -- --download --since 01.01.2025

### Download and filter by identifier threshold (IDs > N)
cargo run --release -- --download --larger 5000

### Download with both date and threshold filters
cargo run --release -- --download --since 01.01.2025 --larger 5000

### Download and extract today's unique IDs, copy to remote server
cargo run --release -- --download --today

### Download and extract today's unique IDs, copy to local directory
cargo run --release -- --download --local

### Process local XML file (all records)
cargo run --release -- AipsDownload_20260130.xml

### Process local XML with date filter
cargo run --release -- AipsDownload_20260130.xml --since 01.01.2025

### Process local XML with threshold filter
cargo run --release -- AipsDownload_20260130.xml --larger 5000

### Process local XML with both filters
cargo run --release -- AipsDownload_20260130.xml --since 01.01.2025 --larger 5000

### Process local XML for today's IDs (remote)
cargo run --release -- AipsDownload_20260130.xml --today

### Process local XML for today's IDs (local)
cargo run --release -- AipsDownload_20260130.xml --local

## Old downloads are pruned

Every `--download` saves `AipsDownload_YYYYMMDD.zip` and `.xml` (about 25 MB a day) in the **current working directory**. The daily cron job in `/etc/crontab` (04:51, `--download --local`) runs in `/home/zdavatz`, and until 30.09.2026 nothing ever removed them: 215 ZIPs and 210 XMLs, 5.1 GB since February. After each download the tool now deletes all `AipsDownload_YYYYMMDD.zip/.xml` in the working directory except the three newest dates. Other files (`today`, `AIPS_Download.xsd`, anything not named with an 8-digit date) are left alone. Test: `cargo test`.

The cron line calls `target/debug/swissmedicinfo`, so after a change run `cargo build` (not only `--release`).
