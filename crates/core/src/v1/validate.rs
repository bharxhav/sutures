use super::schema::Direction;
use crate::error::Error;

/// Maximum allowed regex pattern length (between backticks).
const MAX_REGEX_LEN: usize = 200;

/// Allowed characters in terminals (both JSON and struct).
/// Matches schema pattern: `[A-Za-z0-9_$.[\]:?/`^()\-+\\*|]`
fn is_terminal_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '_' | '$'
                | '.'
                | '['
                | ']'
                | ':'
                | '?'
                | '/'
                | '`'
                | '^'
                | '('
                | ')'
                | '-'
                | '+'
                | '\\'
                | '*'
                | '|'
        )
}

/// Validate a key (read side / LHS).
///
/// - Request keys are struct terminals.
/// - Response keys are JSON terminals.
/// - Regex (`` `pattern` ``) allowed on read side.
/// - Pythonic iterators (`[:]`, `[1:3]`, `[::2]`, etc.) allowed.
pub(super) fn validate_key(key: &str, direction: &Direction) -> Result<(), Error> {
    if key.is_empty() {
        return Err(Error::Suture("key must not be empty".into()));
    }
    match direction {
        Direction::Request => validate_struct_terminal(key, "request key")?,
        Direction::Response => validate_json_terminal(key, "response key")?,
    }
    validate_charset(key)?;
    validate_backticks(key)?;
    validate_brackets(key)?;
    Ok(())
}

/// Validate a value (write side / RHS).
///
/// - Request values are JSON terminals.
/// - Response values are struct terminals.
/// - Regex is **forbidden** on the write side.
/// - Pythonic iterators allowed (zipped with read-side).
pub(super) fn validate_terminal(s: &str, direction: &Direction) -> Result<(), Error> {
    if s.is_empty() {
        return Err(Error::Suture("terminal must not be empty".into()));
    }
    match direction {
        Direction::Request => validate_json_terminal(s, "request value")?,
        Direction::Response => validate_struct_terminal(s, "response value")?,
    }
    validate_charset(s)?;
    if contains_backtick(s) {
        return Err(Error::Suture(format!(
            "regex is not allowed on the write side, got: '{s}'"
        )));
    }
    validate_brackets(s)?;
    Ok(())
}

/// Validate a single constant entry value — any valid JSON is accepted.
pub(super) fn validate_constant(_val: &serde_json::Value) -> Result<(), Error> {
    Ok(())
}

// ===========================================================================
// JSON terminal
// ===========================================================================

fn validate_json_terminal(s: &str, ctx: &str) -> Result<(), Error> {
    if !s.starts_with('/') {
        return Err(Error::Suture(format!(
            "{ctx} must start with '/', got: '{s}'"
        )));
    }
    Ok(())
}

// ===========================================================================
// Struct terminal
// ===========================================================================

fn validate_struct_terminal(s: &str, ctx: &str) -> Result<(), Error> {
    if !s.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(Error::Suture(format!(
            "{ctx} must start with a letter, got: '{s}'"
        )));
    }
    if s.ends_with('.') {
        return Err(Error::Suture(format!(
            "{ctx} must not end with '.', got: '{s}'"
        )));
    }
    if s.contains("..") {
        return Err(Error::Suture(format!(
            "{ctx} must not contain consecutive dots '..', got: '{s}'"
        )));
    }
    Ok(())
}

// ===========================================================================
// Character whitelist
// ===========================================================================

fn validate_charset(s: &str) -> Result<(), Error> {
    for (i, c) in s.char_indices() {
        // Characters inside backtick-delimited regex are exempt — regex has
        // its own character rules.
        if c == '`' {
            break; // backtick validation handles the rest
        }
        if !is_terminal_char(c) {
            return Err(Error::Suture(format!(
                "invalid character '{c}' at position {i} in terminal: '{s}'"
            )));
        }
    }

    // Validate chars outside backticks only.
    let mut in_backtick = false;
    for (i, c) in s.char_indices() {
        if c == '`' {
            in_backtick = !in_backtick;
            continue;
        }
        if !in_backtick && !is_terminal_char(c) {
            return Err(Error::Suture(format!(
                "invalid character '{c}' at position {i} in terminal: '{s}'"
            )));
        }
    }
    Ok(())
}

// ===========================================================================
// Backtick / regex validation
// ===========================================================================

fn contains_backtick(s: &str) -> bool {
    s.contains('`')
}

/// Validate all backtick-delimited regex segments in a terminal.
///
/// Checks:
/// - Balanced backticks (even count).
/// - No empty patterns (`` `` is rejected).
/// - Pattern compiles as a valid regex (Rust `regex` crate flavor).
/// - No capturing groups (use `(?:...)` instead).
/// - Pattern length ≤ MAX_REGEX_LEN.
fn validate_backticks(s: &str) -> Result<(), Error> {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            let open = i;
            i += 1;

            // Find closing backtick.
            let pattern_start = i;
            while i < bytes.len() && bytes[i] != b'`' {
                i += 1;
            }
            if i >= bytes.len() {
                return Err(Error::Suture(format!(
                    "unmatched backtick at position {open} in terminal: '{s}'"
                )));
            }

            let pattern = &s[pattern_start..i];
            i += 1; // skip closing backtick

            // Empty pattern.
            if pattern.is_empty() {
                return Err(Error::Suture(format!(
                    "empty regex pattern not allowed in terminal: '{s}'"
                )));
            }

            // Length limit.
            if pattern.len() > MAX_REGEX_LEN {
                return Err(Error::Suture(format!(
                    "regex pattern exceeds max length ({MAX_REGEX_LEN} chars) in terminal: '{s}'"
                )));
            }

            // No capturing groups — reject unescaped `(` not followed by `?:`.
            validate_no_capturing_groups(pattern, s)?;

            // Must compile as valid regex.
            if let Err(e) = regex::Regex::new(&format!("^{pattern}$")) {
                return Err(Error::Suture(format!(
                    "invalid regex `{pattern}` in terminal '{s}': {e}"
                )));
            }

            continue;
        }

        i += 1;
    }

    Ok(())
}

/// Reject capturing groups — only non-capturing `(?:...)` allowed.
fn validate_no_capturing_groups(pattern: &str, full: &str) -> Result<(), Error> {
    let bytes = pattern.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            // Skip escaped character.
            i += 2;
            continue;
        }
        if bytes[i] == b'(' {
            // Check if followed by `?` (non-capturing / assertion).
            if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                // `(?:...)`, `(?=...)`, `(?!...)`, etc. — allowed.
                i += 2;
                continue;
            }
            return Err(Error::Suture(format!(
                "capturing groups not allowed in regex, use (?:...) instead, in terminal: '{full}'"
            )));
        }
        i += 1;
    }

    Ok(())
}

// ===========================================================================
// Bracket / pythonic iterator validation
//
// Valid forms inside `[...]`:
//   [N]              index      — [0], [-1], [42]
//   [start:end]      slice      — [1:3], [:3], [1:], [:]
//   [start:end:step] extended   — [::2], [1::2], [1:3:2], [::-1]
//
// Rules:
//   - Empty brackets `[]` are invalid.
//   - Parts are optional integers (may be negative).
//   - Step must not be 0.
//   - No whitespace inside brackets.
//   - Max 2 colons (3 parts).
//   - Brackets inside backtick-delimited regex are skipped.
// ===========================================================================

fn validate_brackets(s: &str) -> Result<(), Error> {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip backtick-delimited regex — don't validate brackets inside.
        if bytes[i] == b'`' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'`' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // skip closing backtick
            }
            continue;
        }

        if bytes[i] == b'[' {
            i += 1;
            let bracket_start = i;

            // Find closing bracket — no nesting allowed.
            while i < bytes.len() && bytes[i] != b']' {
                if bytes[i] == b'[' {
                    return Err(Error::Suture(format!(
                        "nested brackets not allowed in terminal: '{s}'"
                    )));
                }
                i += 1;
            }
            if i >= bytes.len() {
                return Err(Error::Suture(format!("unclosed '[' in terminal: '{s}'")));
            }

            let inner = &s[bracket_start..i];
            i += 1; // skip ']'

            validate_bracket_inner(inner, s)?;
            continue;
        }

        // Stray closing bracket.
        if bytes[i] == b']' {
            return Err(Error::Suture(format!(
                "unexpected ']' without matching '[' in terminal: '{s}'"
            )));
        }

        i += 1;
    }

    Ok(())
}

/// Validate the contents between `[` and `]`.
fn validate_bracket_inner(inner: &str, full: &str) -> Result<(), Error> {
    // Empty brackets.
    if inner.is_empty() {
        return Err(Error::Suture(format!(
            "empty brackets '[]' not allowed in terminal: '{full}'"
        )));
    }

    // No whitespace.
    if inner.contains(|c: char| c.is_ascii_whitespace()) {
        return Err(Error::Suture(format!(
            "whitespace not allowed inside brackets '[{inner}]' in terminal: '{full}'"
        )));
    }

    let parts: Vec<&str> = inner.split(':').collect();

    match parts.len() {
        // [N] — single index.
        1 => {
            validate_int_part(parts[0], full, "index")?;
        }
        // [start:end] — slice.
        2 => {
            if !parts[0].is_empty() {
                validate_int_part(parts[0], full, "start")?;
            }
            if !parts[1].is_empty() {
                validate_int_part(parts[1], full, "end")?;
            }
        }
        // [start:end:step] — extended slice.
        3 => {
            if !parts[0].is_empty() {
                validate_int_part(parts[0], full, "start")?;
            }
            if !parts[1].is_empty() {
                validate_int_part(parts[1], full, "end")?;
            }
            if !parts[2].is_empty() {
                let step = validate_int_part(parts[2], full, "step")?;
                if step == 0 {
                    return Err(Error::Suture(format!(
                        "step cannot be 0 in bracket expression '[{inner}]' in terminal: '{full}'"
                    )));
                }
            }
        }
        _ => {
            return Err(Error::Suture(format!(
                "too many ':' in bracket expression '[{inner}]' in terminal: '{full}'"
            )));
        }
    }

    Ok(())
}

/// Validate that a slice part is a valid integer. Returns the parsed value.
fn validate_int_part(part: &str, full: &str, label: &str) -> Result<i64, Error> {
    part.parse::<i64>().map_err(|_| {
        Error::Suture(format!(
            "invalid {label} '{part}' in bracket expression, expected integer, in terminal: '{full}'"
        ))
    })
}
