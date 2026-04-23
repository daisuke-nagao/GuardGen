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
