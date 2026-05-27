# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2026-05-28

### Fixed

- Export `IncludeGuardGenerator` trait from the library API to allow wasm bindings to utilize the include guard generation functionality.

## [2.1.0] - 2026-05-28

### Fixed

- Export `UuidKind` enum from the library API to allow wasm bindings to specify the UUID generation kind (v4 or v7).

## [2.0.0] - 2026-05-28

### Added

- Added a reusable `guardgen_lib::IncludeGuardGenerator` API for generating include guards from library code.
- Exposed library enums for language selection, line endings, and UUID generation kind.
- Added support for selecting UUID v4 generation in the library API while keeping UUID v7 as the CLI default.
- Added wasm-friendly bindings for the library API where applicable.

### Changed

- Updated the CLI to delegate include-guard generation to the shared library implementation.
- Switched the UUID implementation from the older `uuid7` path to the `uuid` crate-based implementation.
- Kept the existing output format and defaults so the change remains compatibility-preserving for CLI users.

## [1.2.1] - 2025-02-01

### Added

- Integrated `cargo about` for license compliance.
  - Added `about.toml` to track and clarify dependency licenses.
  - Created `about.hbs` template for generating a human-readable license report.
  - Implemented license verification and documentation for third-party dependencies.

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
