# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2025-01-30

### Changed

- Improved help messages for the `-h` option.
  - Added `long_about` to provide a detailed overview of the tool.
  - Enhanced `help` descriptions for all command-line arguments.
  - Clarified language options (`none`, `c`, `cxx`) and line-ending styles (`none`, `lf`, `crlf`).
- Improved error handling for file operations.
  - Added specific error handling for `AlreadyExists` and `PermissionDenied` cases.
  - Improved error messages to provide clearer feedback on file operation failures.

---

## [1.1.0] - 2025-01-28

### Added

- Introduced `--prefix` option for customizing the prefix of the include guard (default: `"UUID"`).
- Introduced `--suffix` option for appending a suffix to the include guard (default: none).
- Supported combining `--prefix` and `--suffix` for full customization.
- Introduced `-x` option to specify the target language for include guard generation.
  - Supported values: `none` (default), `c`, and `cxx`.
  - Added `extern "C" {}` block when targeting C with `-x c`.
- Introduced `--line-ending` option to control line endings in the generated output.
  - Supported values: `LF`, `CRLF`, and `None` (default: system standard).
  - Automatically detects and uses the appropriate line ending if `None` is specified.
- Introduced the `clap` crate (version 4.5.27) for argument parsing.
  - Added `--output`/`-o` and `--overwrite` options with `clap`.
  - Enabled `derive` feature for `clap` in `Cargo.toml`.

### Changed

- Replaced custom argument parsing logic in `src/main.rs` with `clap`.
  - Improved maintainability and readability by leveraging `clap`'s features.
- Updated `Cargo.lock` to include new dependencies related to `clap`.

### Removed

- Deprecated manual argument parsing using `std::env::args()` in favor of `clap`.
