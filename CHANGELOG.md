# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-11-01

### Added

- **Optional regex feature support**: Added `regex` feature flag for advanced file filtering using regular expressions
- **New CLI options**:
  - `-n, --dryrun`:              Preview what would be done without executing
  - `-i, --include <PATTERN>`:   Include files matching glob pattern(s)
  - `    --includere <PATTERN>`: Include files matching regex pattern(s)
  - `-e, --exclude <PATTERN>`:   Exclude files matching glob pattern(s)
  - `    --excludere <PATTERN>`: Exclude files matching regex pattern(s)
- **Enhanced file filtering**: Support for both glob patterns and regex patterns in file inclusion/exclusion

### Changed

- **CLI interface redesign**:
  - Now requires an explicitly designated directory to process
  - Include patterns work alongside directory walking
- **Module restructuring**:
  - Renamed `runner` module to `batch_runner`
  - Renamed `batch` module to `exec`
- **Default compression level**: Changed from 3 to 5 for better compression ratio
- **Dependency management**: Added `regex` crate v1.12.2 as optional dependency
- **Place for new options**： `--quiet` and `--verbose` flags are currently non-functional (marked as NO FUNCTION in help)

### Fixed

- **Code quality**: Fixed clippy warnings and improved code style
- **Lock file**: Now tracking `Cargo.lock` in version control for reproducible builds

---

## [0.3.3] - Previous Release

- Previous version features and changes (see git history for details)
