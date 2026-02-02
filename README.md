# swissmedicinfo
Helper for Swissmedic Info

## Output File Summary

| Option(s) | Filename | Format | Content |
|-----------|----------|--------|---------|
| (default) | `swissmedicinfo_DD.MM.YYYY.csv` | CSV | All records, identifier + date |
| `--larger N` | `larger_N_DD.MM.YYYY.csv` | CSV | Unique IDs > N, most recent date |
| `--today` | `today` | Text | Today's unique IDs, 5 digits, remote copy |
| `--local` | `today` | Text | Today's unique IDs, 5 digits, local copy |
