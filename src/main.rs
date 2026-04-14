// SPDX-FileCopyrightText: 2025 Daisuke Nagao
// SPDX-License-Identifier: MIT

use clap::Parser;
use clap::ValueEnum;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

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
        default_value_t = LanguageArg::None,
        ignore_case = true,
        help = "Specify the language for compatibility adjustments. \
                Options: none (default), c (adds extern \"C\" blocks), cxx (no additional modification)."
    )]
    x: LanguageArg,

    /// Line-ending style (LF/CRLF)
    #[arg(
        long = "line-ending",
        value_enum,
        default_value_t = LineEndingArg::None,
        ignore_case = true,
        help = "Specify the line-ending style. \
                Options: none (auto-detect), lf (Unix-style LF), crlf (Windows-style CRLF)."
    )]
    line_ending: LineEndingArg,
}

/// Main function that parses arguments and generates the include guard.
fn main() {
    // Parse command-line arguments using `clap`.
    let args = Args::parse();

    // Generate the include guard based on user input.
    // Convert CLI wrapper enums into library enums to keep the library clap-agnostic.
    let guard = guardgen::generate_guard(
        args.prefix,
        args.suffix,
        args.x.into(),
        args.line_ending.into(),
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

// Wrapper enums for CLI integration.
// These are defined in the binary crate so we can derive `clap::ValueEnum`
// without violating Rust's orphan rules. They convert into the library
// enums (`guardgen::Language` and `guardgen::LineEnding`).
#[derive(ValueEnum, Clone, Copy, Debug)]
enum LanguageArg {
    None,
    C,
    Cxx,
}

impl From<LanguageArg> for guardgen::Language {
    fn from(v: LanguageArg) -> guardgen::Language {
        match v {
            LanguageArg::None => guardgen::Language::None,
            LanguageArg::C => guardgen::Language::C,
            LanguageArg::Cxx => guardgen::Language::Cxx,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum LineEndingArg {
    None,
    LF,
    CRLF,
}

impl From<LineEndingArg> for guardgen::LineEnding {
    fn from(v: LineEndingArg) -> guardgen::LineEnding {
        match v {
            LineEndingArg::None => guardgen::LineEnding::None,
            LineEndingArg::LF => guardgen::LineEnding::LF,
            LineEndingArg::CRLF => guardgen::LineEnding::CRLF,
        }
    }
}
