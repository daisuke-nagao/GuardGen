# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Introduced the `clap` crate (version 4.5.27) for argument parsing.
  - Added `--output`/`-o` and `--overwrite` options with `clap`.
  - Enabled `derive` feature for `clap` in `Cargo.toml`.

### Changed
- Replaced custom argument parsing logic in `src/main.rs` with `clap`.
  - Improved maintainability and readability by leveraging `clap`'s features.
- Updated `Cargo.lock` to include new dependencies related to `clap`.

### Removed
- Deprecated manual argument parsing using `std::env::args()` in favor of `clap`.

