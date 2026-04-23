// SPDX-FileCopyrightText: 2026 Daisuke Nagao
// SPDX-License-Identifier: MIT

/// Enum representing the target language.
/// - `None`: No language-specific modifications.
/// - `C`: Adds `extern "C"` for C compatibility.
/// - `Cxx`: No additional modifications (C++ default behavior).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    None,
    C,
    Cxx,
}

#[allow(clippy::upper_case_acronyms)]
/// Enum representing line-ending styles.
/// - `None`: Uses system default.
/// - `LF`: Uses Unix-style LF.
/// - `CRLF`: Uses Windows-style CRLF.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineEnding {
    None,
    LF,
    CRLF,
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
pub fn generate_guard(
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
