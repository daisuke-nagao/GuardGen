// SPDX-FileCopyrightText: 2026 Daisuke Nagao
// SPDX-License-Identifier: MIT

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::*;

/// Enum representing the target language.
/// - `None`: No language-specific modifications.
/// - `C`: Adds `extern "C"` for C compatibility.
/// - `Cxx`: No additional modifications (C++ default behavior).
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    None,
    C,
    Cxx,
}

/// Enum representing line-ending styles.
/// - `None`: Uses system default.
/// - `LF`: Uses Unix-style LF.
/// - `CRLF`: Uses Windows-style CRLF.
#[allow(clippy::upper_case_acronyms)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
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
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub fn generate_guard(
    prefix: String,
    suffix: Option<String>,
    x: Language,
    line_ending: LineEnding,
) -> String {
    // Generate a UUID and format it for use in the include guard.
    let uuid = generate().to_string().replace('-', "_").to_uppercase();
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

fn unix_time() -> (u64, u32) {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        // `js_sys::Date::now()` returns milliseconds since the epoch as an `f64`.
        // Convert to integer milliseconds, then split into seconds and nanoseconds.
        let unix_ms = js_sys::Date::now().floor() as u64;
        let seconds = unix_ms / 1000u64;
        let nanos = ((unix_ms % 1000) as u32) * 1_000_000u32;
        (seconds, nanos)
    }
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        let seconds = now.as_secs();
        // `subsec_millis()` returns the subsecond part in milliseconds; convert to nanoseconds.
        let nanos = now.subsec_millis() * 1_000_000;
        (seconds, nanos)
    }
}

fn generate() -> uuid::Uuid {
    let (seconds, millis) = unix_time();
    let ts = uuid::Timestamp::from_unix(uuid::NoContext, seconds, millis);

    uuid::Uuid::new_v7(ts)
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
