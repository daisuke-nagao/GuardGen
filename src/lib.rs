// SPDX-FileCopyrightText: 2026 Daisuke Nagao
// SPDX-License-Identifier: MIT

use wasm_bindgen::prelude::*;

/// Enum representing the target language.
/// - `None`: No language-specific modifications.
/// - `C`: Adds `extern "C"` for C compatibility.
/// - `Cxx`: No additional modifications (C++ default behavior).
#[derive(Copy, Clone, Debug)]
pub enum Language {
    None,
    C,
    Cxx,
}

impl Language {
    /// Parses a JavaScript-friendly language specifier.
    ///
    /// Preconditions:
    /// - `value` is expected to represent one of: `none`, `c`, or `cxx`.
    ///
    /// Postconditions:
    /// - Returns `Ok(Language)` when the value is recognized.
    /// - Returns `Err(String)` with a human-readable reason when invalid.
    ///
    /// Invariants:
    /// - The mapping is case-insensitive.
    fn parse_for_wasm(value: &str) -> Result<Self, String> {
        let normalized = value.trim().to_ascii_lowercase();
        let parsed = match normalized.as_str() {
            // Branch: explicit no-op language mode.
            "none" => Ok(Self::None),
            // Branch: C compatibility mode inserts extern "C" block.
            "c" => Ok(Self::C),
            // Branch: C++ mode uses plain include guard output.
            "cxx" | "cpp" | "c++" => Ok(Self::Cxx),
            // Branch: unsupported keyword from JavaScript caller.
            _ => Err(format!(
                "invalid language '{}'; expected one of: none, c, cxx",
                value
            )),
        };

        parsed
    }
}

#[allow(clippy::upper_case_acronyms)]
/// Enum representing line-ending styles.
/// - `None`: Uses system default.
/// - `LF`: Uses Unix-style LF.
/// - `CRLF`: Uses Windows-style CRLF.
#[derive(Copy, Clone, Debug)]
pub enum LineEnding {
    None,
    LF,
    CRLF,
}

impl LineEnding {
    /// Parses a JavaScript-friendly line-ending specifier.
    ///
    /// Preconditions:
    /// - `value` is expected to represent one of: `none`, `lf`, or `crlf`.
    ///
    /// Postconditions:
    /// - Returns `Ok(LineEnding)` when the value is recognized.
    /// - Returns `Err(String)` with a human-readable reason when invalid.
    ///
    /// Invariants:
    /// - The mapping is case-insensitive.
    fn parse_for_wasm(value: &str) -> Result<Self, String> {
        let normalized = value.trim().to_ascii_lowercase();
        let parsed = match normalized.as_str() {
            // Branch: auto-detects line ending based on runtime target.
            "none" | "auto" => Ok(Self::None),
            // Branch: enforces LF newline style.
            "lf" => Ok(Self::LF),
            // Branch: enforces CRLF newline style.
            "crlf" => Ok(Self::CRLF),
            // Branch: unsupported keyword from JavaScript caller.
            _ => Err(format!(
                "invalid line ending '{}'; expected one of: none, lf, crlf",
                value
            )),
        };

        parsed
    }
}

/// WebAssembly-exported wrapper for JavaScript/browser callers.
///
/// Preconditions:
/// - `prefix` should be a non-empty identifier-like string.
/// - `language` should be one of: `none`, `c`, or `cxx`.
/// - `line_ending` should be one of: `none`, `lf`, or `crlf`.
///
/// Postconditions:
/// - Returns the generated include guard text on success.
/// - Returns a JavaScript exception (`JsValue`) if inputs are invalid.
///
/// Invariants:
/// - Delegates guard construction to `generate_guard` so output format remains consistent.
#[wasm_bindgen]
pub fn generate_guard_wasm(
    prefix: &str,
    suffix: Option<String>,
    language: &str,
    line_ending: &str,
) -> Result<String, JsValue> {
    // Install panic hook so Rust panic messages appear in browser devtools console.
    // This statement is compiled only on wasm32 targets where the crate is available.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Preconditions: reject empty prefixes early because they produce malformed guard names.
    if prefix.trim().is_empty() {
        return Err(JsValue::from_str("prefix must not be empty"));
    }

    let language_value = Language::parse_for_wasm(language)
        .map_err(|err| JsValue::from_str(format!("language error: {}", err).as_str()))?;
    let line_ending_value = LineEnding::parse_for_wasm(line_ending)
        .map_err(|err| JsValue::from_str(format!("line ending error: {}", err).as_str()))?;

    // Branch: suffix is optional and passed through unchanged to core logic.
    let generated = generate_guard(
        prefix.to_string(),
        suffix,
        language_value,
        line_ending_value,
    );

    Ok(generated)
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
    // Generate a UUID-like identifier for use in the include guard.
    // Use `uuid7` on native targets. For `wasm32`, instantiate a `V7Generator`
    // with a JS-backed `TimeSource` (Date.now()) and a `RandSource` that uses
    // `getrandom` so we avoid calls to `std::time` which panic on wasm.
    #[cfg(target_arch = "wasm32")]
    mod wasm_uuid7 {
        use js_sys::Date;
        use once_cell::sync::Lazy;
        use std::sync::Mutex;
        use uuid7::generator::{RandSource, TimeSource};
        use uuid7::V7Generator;

        pub struct WasmTimeSource;
        impl TimeSource for WasmTimeSource {
            fn unix_ts_ms(&mut self) -> u64 {
                // Date::now() returns milliseconds as f64; cast to u64.
                Date::now() as u64
            }
        }

        use web_sys::window;

        pub struct GetRandomSource;
        impl RandSource for GetRandomSource {
            fn next_u32(&mut self) -> u32 {
                let mut b = [0u8; 4];
                let win = window().expect("no window");
                let crypto = win.crypto().expect("no crypto available");
                // Fill via get_random_values into a Uint8Array backed by our buffer
                crypto
                    .get_random_values_with_u8_array(&mut b)
                    .expect("get_random_values failed");
                u32::from_be_bytes(b)
            }

            fn next_u64(&mut self) -> u64 {
                let mut b = [0u8; 8];
                let win = window().expect("no window");
                let crypto = win.crypto().expect("no crypto available");
                crypto
                    .get_random_values_with_u8_array(&mut b)
                    .expect("get_random_values failed");
                u64::from_be_bytes(b)
            }
        }

        static GLOBAL_GEN: Lazy<Mutex<V7Generator<GetRandomSource, WasmTimeSource>>> =
            Lazy::new(|| {
                Mutex::new(V7Generator::with_rand_and_time_sources(
                    GetRandomSource,
                    WasmTimeSource,
                ))
            });

        pub fn generate_uuid_string() -> String {
            let mut g = GLOBAL_GEN.lock().unwrap();
            g.generate().to_string()
        }
    }

    let uuid = {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_uuid7::generate_uuid_string()
                .replace('-', "_")
                .to_uppercase()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            uuid7::uuid7().to_string().replace('-', "_").to_uppercase()
        }
    };
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
