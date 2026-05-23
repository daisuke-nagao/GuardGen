// SPDX-FileCopyrightText: 2026 Daisuke Nagao
// SPDX-License-Identifier: MIT

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown", test))]
use wasm_bindgen_test::*;

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

/// Enum selecting UUID generation strategy.
///
/// - V7: Time-ordered UUID version 7 (preferred for ordered identifiers).
/// - V4: Random UUID version 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UuidKind {
    V7,
    V4,
}

/// Include guard generator struct.
///
/// @pre The `prefix` must be a non-empty string describing the guard prefix.
/// @post Calling `generate(&mut self)` returns a well-formed include-guard text.
/// @invariant The internal `v7_context` (if present) is private and used to ensure
///            monotonic UUID v7 generation for short-interval repeated calls.
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
pub struct IncludeGuardGenerator {
    // Private context used for UUID v7 generation to avoid collisions on rapid calls.
    // Stored as (last_seconds, last_nanos, counter) to provide a small monotonic
    // context without depending on internal uuid crate types.
    v7_context: Option<(u64, u32, u32)>,
}

impl IncludeGuardGenerator {
    /// Create a new `IncludeGuardGenerator`.
    ///
    /// @pre `prefix` must not be empty.
    /// @post Returns an initialized generator configured to produce UUIDs of type `uuid_kind`.
    /// Create a new `IncludeGuardGenerator` that holds only the internal context.
    ///
    /// The generator does not store prefix/suffix/language/line ending or UUID kind;
    /// those are provided per-call to `generate` to allow callers to reuse the
    /// same context while varying parameters.
    pub fn new() -> Self {
        IncludeGuardGenerator {
            v7_context: Some((0u64, 0u32, 0u32)),
        }
    }

    /// Generate the include guard string using the generator's configuration.
    ///
    /// @pre The generator was created via `new`.
    /// @post Returns a textual include guard using the selected UUID generation method.
    /// Generate the include guard string using the provided parameters.
    ///
    /// All parameters are supplied on each call so the same generator instance
    /// can be reused with different prefixes/suffixes/UUID kinds.
    pub fn generate(
        &mut self,
        prefix: String,
        suffix: Option<String>,
        language: Language,
        line_ending: LineEnding,
        uuid_kind: UuidKind,
    ) -> String {
        // Generate a UUID string according to the selected kind.
        let uuid_string = match uuid_kind {
            UuidKind::V4 => uuid::Uuid::new_v4().to_string(),
            UuidKind::V7 => {
                // Use the existing `unix_time` helper to get seconds and subsecond
                // component. Maintain a tiny local context (last timestamp + counter)
                // to avoid collisions when multiple calls occur within the same
                // millisecond.
                let (mut seconds, mut nanos) = unix_time();

                if let Some(ctx) = self.v7_context.as_mut() {
                    // ctx = (last_seconds, last_nanos, counter)
                    if ctx.0 == seconds && ctx.1 == nanos {
                        // Same timestamp as previous; bump a small counter and
                        // adjust the nanoseconds to preserve uniqueness.
                        ctx.2 = ctx.2.saturating_add(1);
                        // Add the counter value to the nanos, clamping under 1e9.
                        nanos = nanos.saturating_add(ctx.2);
                        if nanos >= 1_000_000_000 {
                            nanos = 999_999_999;
                        }
                    } else {
                        // New timestamp; reset the counter and record values.
                        ctx.0 = seconds;
                        ctx.1 = nanos;
                        ctx.2 = 0;
                    }
                }

                let ts = uuid::Timestamp::from_unix(uuid::NoContext, seconds, nanos);
                uuid::Uuid::new_v7(ts).to_string()
            }
        };

        // Format guard pieces and return the assembled include-guard text.
        let uuid = uuid_string.replace('-', "_").to_uppercase();
        let mut guard = vec![prefix, uuid.clone()];

        // If a suffix was provided, append it to the guard components.
        if let Some(s) = &suffix {
            guard.push(s.clone());
        }

        let guard = guard.join("_");

        let ifndef = format!("#ifndef {}", guard);
        let define = format!("#define {}", guard);
        let endif = format!("#endif /* {} */", guard);

        let mut text = vec![ifndef, define];

        // If the target language is C, add extern "C" compatibility blocks.
        // This branch ensures C consumers get the correct linkage annotations.
        if let Language::C = language {
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
            LineEnding::None => {
                // Qualitative explanation: pick system default line ending.
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
    // Use the new struct-based API (default to UUID v7 for compatibility).
    let mut generator = IncludeGuardGenerator::new();
    generator.generate(prefix, suffix, x, line_ending, UuidKind::V7)
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn extract_uuids(text: &str) -> Vec<String> {
        let re =
            Regex::new(r"[0-9A-F]{8}_[0-9A-F]{4}_[0-9A-F]{4}_[0-9A-F]{4}_[0-9A-F]{12}").unwrap();

        re.find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
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

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
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

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
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

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
    fn test_include_guard_generator_v7_uniqueness() {
        let mut generator = IncludeGuardGenerator::new();

        let r1 = generator.generate(
            "TEST".to_string(),
            None,
            Language::None,
            LineEnding::LF,
            UuidKind::V7,
        );
        let r2 = generator.generate(
            "TEST".to_string(),
            None,
            Language::None,
            LineEnding::LF,
            UuidKind::V7,
        );

        let u1 = extract_uuids(r1.as_str())[0].clone();
        let u2 = extract_uuids(r2.as_str())[0].clone();

        // Two successive calls should produce distinct UUIDs.
        assert_ne!(u1, u2);
    }

    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
    #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
    fn test_include_guard_generator_v4_and_short_interval() {
        // Test UUID v4 selection produces valid-looking UUIDs and uniqueness across calls.
        let mut g_v4 = IncludeGuardGenerator::new();

        let mut seen = std::collections::HashSet::new();
        for _ in 0..8 {
            let r = g_v4.generate(
                "TEST".to_string(),
                None,
                Language::None,
                LineEnding::LF,
                UuidKind::V4,
            );
            let u = extract_uuids(r.as_str())[0].clone();
            assert!(seen.insert(u), "Duplicate UUID found for v4 generator");
        }

        // Test rapid successive v7 calls are unique.
        let mut g_v7 = IncludeGuardGenerator::new();

        let mut seen_v7 = std::collections::HashSet::new();
        for _ in 0..16 {
            let r = g_v7.generate(
                "TEST".to_string(),
                None,
                Language::None,
                LineEnding::LF,
                UuidKind::V7,
            );
            let u = extract_uuids(r.as_str())[0].clone();
            assert!(seen_v7.insert(u), "Duplicate UUID found for v7 generator");
        }
    }
}
