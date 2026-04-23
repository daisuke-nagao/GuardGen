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
