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
}

/// Generates an include guard string with optional language-specific modifications.
///
/// # Arguments
/// * `prefix` - A prefix string for the guard name.
/// * `suffix` - An optional suffix for the guard name.
/// * `x` - The target language (C or C++).
/// * `line_ending` - The line-ending format.
///
/// # Returns
/// A formatted include guard string.
fn generate_guard(
    prefix: String,
    suffix: Option<String>,
    x: Language,
    line_ending: LineEnding,
) -> String {
    // Generate a UUID and format it for use in the include guard.
    let uuid = uuid7::uuid7().to_string().replace('-', "_").to_uppercase();
    let mut guard = vec![prefix, uuid];

    // Append suffix if provided.
    if let Some(suffix) = suffix {
        guard.push(suffix);
    }

    // Join guard components with underscores.
    let guard = guard.join("_");

    // Define standard include guard macros.
    let ifndef = format!("#ifndef {}", guard);
    let define = format!("#define {}", guard);
    let endif = format!("#endif /* {} */", guard);

    let mut text = vec![ifndef, define];

    // If the target language is C, add `extern "C"` blocks for compatibility.
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

    // Append closing `#endif` statement.
    text.push(endif);
    text.push("".to_string());

    // Determine the newline character based on the specified line-ending format.
    let newline = match line_ending {
        LineEnding::LF => "\n",
        LineEnding::CRLF => "\r\n",
        LineEnding::None => {
            if cfg!(target_os = "windows") {
                "\r\n"
            } else {
                "\n"
            }
        }
    }
    .to_string();

    // Join all lines with the appropriate newline character.
    text.join(&newline)
}

/// Main function that parses arguments and generates the include guard.
fn main() {
    // Parse command-line arguments using `clap`.
    let args = Args::parse();

    // Generate the include guard based on user input.
    let guard = generate_guard(args.prefix, args.suffix, args.x, args.line_ending);

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
    use regex::Regex;

    fn extract_uuids(text: &str) -> Vec<String> {
        let re =
            Regex::new(r"[0-9A-F]{8}_[0-9A-F]{4}_[0-9A-F]{4}_[0-9A-F]{4}_[0-9A-F]{12}").unwrap();

        re.find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }

    #[test]
    fn test_generate_guard_default() {
        let result = generate_guard("TEST".to_string(), None, Language::None, LineEnding::LF);

        let uuids = extract_uuids(result.as_str());

        assert!(uuids.len() == 3);
        assert!(uuids[0] == uuids[1]);
        assert!(uuids[1] == uuids[2]);

        let uuid = &uuids[0];
        assert!(result.contains(format!("#ifndef TEST_{}", uuid).as_str()));
        assert!(result.contains(format!("#define TEST_{}", uuid).as_str()));
        assert!(result.contains(format!("#endif /* TEST_{} */", uuid).as_str()));
    }

    #[test]
    fn test_generate_guard_with_suffix() {
        let result = generate_guard(
            "TEST".to_string(),
            Some("SUFFIX".to_string()),
            Language::Cxx,
            LineEnding::LF,
        );

        let uuids = extract_uuids(result.as_str());

        assert!(uuids.len() == 3);
        assert!(uuids[0] == uuids[1]);
        assert!(uuids[1] == uuids[2]);

        let uuid = &uuids[0];
        assert!(result.contains(format!("#ifndef TEST_{}_SUFFIX", uuid).as_str()));
        assert!(result.contains(format!("#define TEST_{}_SUFFIX", uuid).as_str()));
        assert!(result.contains(format!("#endif /* TEST_{}_SUFFIX */", uuid).as_str()));
    }

    #[test]
    fn test_generate_guard_with_c_compatibility() {
        let result = generate_guard("TEST".to_string(), None, Language::C, LineEnding::LF);

        let uuids = extract_uuids(result.as_str());

        assert!(uuids.len() == 3);
        assert!(uuids[0] == uuids[1]);
        assert!(uuids[1] == uuids[2]);

        let uuid = &uuids[0];
        assert!(result.contains(format!("#ifndef TEST_{}", uuid).as_str()));
        assert!(result.contains(format!("#define TEST_{}", uuid).as_str()));
        assert!(result.contains(format!("#endif /* TEST_{} */", uuid).as_str()));

        assert!(result.contains("#ifdef __cplusplus"));
        assert!(result.contains("extern \"C\" {"));
        assert!(result.contains("} /* extern \"C\" */"));
    }
}
