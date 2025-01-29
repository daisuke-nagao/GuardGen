/*
 * SPDX-FileCopyrightText: 2025 Daisuke Nagao
 *
 * SPDX-License-Identifier: MIT
 */

use clap::{Parser, ValueEnum};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Clone, Debug, ValueEnum)]
enum Language {
    None,
    C,
    Cxx,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, ValueEnum)]
enum LineEnding {
    None,
    LF,
    CRLF,
}

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
}

fn generate_guard(
    prefix: String,
    suffix: Option<String>,
    x: Language,
    line_ending: LineEnding,
) -> String {
    let uuid = uuid7::uuid7().to_string().replace('-', "_").to_uppercase();
    let mut guard = vec![prefix, uuid];
    if let Some(suffix) = suffix {
        guard.push(suffix);
    }

    let guard = guard.join("_");

    let ifndef = format!("#ifndef {}", guard);
    let define = format!("#define {}", guard);
    let endif = format!("#endif /* {} */", guard);

    let mut text = vec![ifndef, define];

    if let Language::C = x {
        let extern_c: Vec<String> = vec![
            "".to_string(), // blank line
            "#ifdef __cplusplus".to_string(),
            "extern \"C\" {".to_string(),
            "#endif /* __cplusplus */".to_string(),
            "".to_string(), // blank line
            "#ifdef __cplusplus".to_string(),
            "} /* extern \"C\" */".to_string(),
            "#endif /* __cplusplus */".to_string(),
            "".to_string(), // blank line
        ];
        text.extend(extern_c);
    }

    text.push(endif);
    text.push("".to_string());

    let newline = match line_ending {
        LineEnding::LF => "\n",
        LineEnding::CRLF => "\r\n",
        _ => {
            if cfg!(target_os = "windows") {
                "\r\n"
            } else {
                "\n"
            }
        }
    }
    .to_string();

    text.join(&newline)
}

fn main() {
    let args = Args::parse();

    let guard = generate_guard(args.prefix, args.suffix, args.x, args.line_ending);

    if let Some(file_path) = &args.filename {
        if !args.overwrite && fs::metadata(file_path).is_ok() {
            eprintln!(
                "Error: File '{}' already exists. Use --overwrite to overwrite.",
                file_path
            );
            std::process::exit(1);
        }

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(args.overwrite)
            .open(file_path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(guard.as_bytes()) {
                    eprintln!("Error writing to file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
                println!("Guard written to '{}'.", file_path);
            }
            Err(e) => {
                eprintln!("Error creating file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", guard);
    }
}
