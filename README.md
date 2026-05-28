# databricks-tui

Terminal dashboard for Databricks — monitor clusters, jobs, pipelines, and SQL warehouses in one view.

## Install

```bash
cargo install --path .
```

## Usage

```bash
databricks-tui                      # default profile, 30s refresh
databricks-tui --profile prod       # named CLI profile
databricks-tui --refresh 10         # refresh every 10 seconds
```

## Keys

| Key | Action |
|-----|--------|
| `Tab` / `→` / `l` | Focus next panel |
| `Shift+Tab` / `←` / `h` | Focus previous panel |
| `r` | Force refresh |
| `q` / `Ctrl+C` | Quit |

## Requirements

- [Databricks CLI v0.200+](https://docs.databricks.com/dev-tools/cli/databricks-cli.html) installed and authenticated

## Release binaries

Push a `v*` tag to trigger a GitHub Actions build that publishes `.tar.gz` binaries for Linux x86, macOS x86, and macOS ARM.
