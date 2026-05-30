// SPDX-FileCopyrightText: 2025 Daisuke Nagao
// SPDX-License-Identifier: MIT

use clap::{Parser, ValueEnum};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

/// Enum representing the target language.
/// - `None`: No language-specific modifications.
/// - `C`: Adds `extern "C"` for C compatibility.
/// - `Cxx`: No additional modifications (C++ default behavior).
#[derive(Clone, Debug, ValueEnum)]
enum Language {
    None,
    C,
    Cxx,
}

impl From<Language> for guardgen_lib::Language {
    fn from(val: Language) -> Self {
        match val {
            Language::None => guardgen_lib::Language::None,
            Language::C => guardgen_lib::Language::C,
            Language::Cxx => guardgen_lib::Language::Cxx,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
/// Enum representing line-ending styles.
/// - `None`: Uses system default.
/// - `LF`: Uses Unix-style LF.
/// - `CRLF`: Uses Windows-style CRLF.
#[derive(Clone, Debug, ValueEnum)]
enum LineEnding {
    None,
    LF,
    CRLF,
}

impl From<LineEnding> for guardgen_lib::LineEnding {
    fn from(val: LineEnding) -> Self {
        match val {
            LineEnding::None => guardgen_lib::LineEnding::None,
            LineEnding::LF => guardgen_lib::LineEnding::LF,
            LineEnding::CRLF => guardgen_lib::LineEnding::CRLF,
        }
    }
}

/// Enum selecting the UUID version used in generated include guards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum UuidVersion {
    #[value(alias = "4")]
    V4,
    #[value(alias = "7")]
    V7,
}

impl From<UuidVersion> for guardgen_lib::UuidKind {
    fn from(val: UuidVersion) -> Self {
        match val {
            UuidVersion::V4 => guardgen_lib::UuidKind::V4,
            UuidVersion::V7 => guardgen_lib::UuidKind::V7,
        }
    }
}

/// Command-line argument parser using `clap`.
#[derive(Parser, Debug)]
#[command(
    author = "Daisuke Nagao",
    version,
    about = "Generates include guards with optional UUID-based naming.",
    long_about = "This tool generates unique include guards for C/C++ header files.\n\
                  The guard name is based on a UUID and optional prefix/suffix.\n\
                  It supports different languages and line-ending formats.\n\
                  The output can be printed to stdout or written to a file."
)]
struct Args {
    /// Output filename (if omitted, prints to stdout)
    #[arg(
        short = 'o',
        long = "output",
        help = "Specify the output file. If omitted, prints to stdout."
    )]
    filename: Option<String>,

    /// Overwrite existing file if specified
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite the output file if it already exists."
    )]
    overwrite: bool,

    /// Prefix for the include guard (default: "UUID")
    #[arg(
        long = "prefix",
        default_value = "UUID",
        help = "Specify a prefix for the include guard. Default: 'UUID'."
    )]
    prefix: String,

    /// Suffix for the include guard (optional)
    #[arg(long = "suffix", default_value = None, help = "Specify an optional suffix for the include guard.")]
    suffix: Option<String>,

    /// Language format (C/C++ specific adjustments)
    #[arg(
        short,
        value_enum,
        default_value_t = Language::None,
        ignore_case = true,
        help = "Specify the language for compatibility adjustments. \
                Options: none (default), c (adds extern \"C\" blocks), cxx (no additional modification)."
    )]
    x: Language,

    /// Line-ending style (LF/CRLF)
    #[arg(
        long = "line-ending",
        value_enum,
        default_value_t = LineEnding::None,
        ignore_case = true,
        help = "Specify the line-ending style. \
                Options: none (auto-detect), lf (Unix-style LF), crlf (Windows-style CRLF)."
    )]
    line_ending: LineEnding,

    /// UUID version used for the include guard
    #[arg(
        short = 'v',
        long = "uuid-version",
        value_enum,
        default_value_t = UuidVersion::V7,
        ignore_case = true,
        help = "Specify the UUID version for the include guard. \
                Options: v7 (default), v4. Compact forms -v7 and -v4 are accepted."
    )]
    uuid_version: UuidVersion,
}

/// Main function that parses arguments and generates the include guard.
fn main() {
    // Parse command-line arguments using `clap`.
    let args = Args::parse();

    // Generate the include guard based on user input using the struct-based API.
    let mut generator = guardgen_lib::IncludeGuardGenerator::new();
    let guard = generator.generate(
        args.prefix,
        args.suffix,
        args.x.into(),
        args.line_ending.into(),
        args.uuid_version.into(),
    );

    if let Some(file_path) = &args.filename {
        // Check if the file already exists and prevent overwriting unless explicitly allowed.
        if !args.overwrite && fs::metadata(file_path).is_ok() {
            eprintln!(
                "Error: File '{}' already exists. Use --overwrite to overwrite.",
                file_path
            );
            std::process::exit(1);
        }

        // Attempt to open the file for writing.
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(args.overwrite)
            .open(file_path)
        {
            Ok(mut file) => {
                // Write the generated include guard to the file.
                if let Err(e) = file.write_all(guard.as_bytes()) {
                    eprintln!("Error writing to file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
                println!("Guard written to '{}'.", file_path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                eprintln!(
                    "Error: File '{}' already exists. Use --overwrite to overwrite.",
                    file_path
                );
                std::process::exit(1);
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("Error: Permission denied when accessing '{}'.", file_path);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error creating file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        }
    } else {
        // Print the include guard to stdout if no output file is specified.
        println!("{}", guard);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn default_uuid_version_is_v7() {
        let args = Args::parse_from(["guardgen"]);

        assert_eq!(args.uuid_version, UuidVersion::V7);
    }

    #[test]
    fn parses_compact_uuid_version_flags() {
        let args_v4 = Args::parse_from(["guardgen", "-v4"]);
        let args_v7 = Args::parse_from(["guardgen", "-v7"]);

        assert_eq!(args_v4.uuid_version, UuidVersion::V4);
        assert_eq!(args_v7.uuid_version, UuidVersion::V7);
    }

    #[test]
    fn rejects_repeated_uuid_version_selection() {
        let err = Args::try_parse_from(["guardgen", "-v4", "-v7"]).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }
}
