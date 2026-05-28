# Changelog

## [Unreleased]

## [0.2.0] - 2026-05-28

### Fixed
- Jobs and warehouses fetchers now handle plain array responses from the CLI
- `IDLE`, `DELETED` states now map to Stopped; `DELETING` maps to Pending
- Status labels show real text (e.g. `IDLE`) instead of `UNKNOWN`
- CI release job now has correct `contents: write` permission

### Changed
- Warehouses panel switched from table to list view with cluster size shown as detail
- All list items now render their detail field dimmed on the right

## [0.1.0] - 2026-05-28

### Added
- Initial scaffold: clusters, jobs, pipelines, warehouses panels
- Auto-refresh with configurable interval (`--refresh`)
- Multi-profile support (`--profile`)
- CI workflow with binary releases on git tags
