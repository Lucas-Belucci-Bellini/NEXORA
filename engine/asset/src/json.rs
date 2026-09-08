//! A strict, deterministic JSON subset.
//!
//! `Block System.md` §51 already describes data-driven content as JSON, so the
//! format is not being invented here — this crate is simply its first reader.
//! ADR-0002 forbids external dependencies, so it is written rather than pulled.
//!
//! Three properties matter more than completeness:
//!
//! * **Deterministic output.** Object keys are held in a [`BTreeMap`], so two
//!   runs that build the same document emit byte-identical text. A provenance
//!   hash over the document is meaningless otherwise.
//! * **Strict input.** No comments, no trailing commas, no unquoted keys, no
//!   `NaN`, no `Infinity`. A file that is nearly JSON is rejected rather than
//!   guessed at.
//! * **Bounded input.** `NEXORA SECURITY THREAT MODEL.md` treats content as an
//!   untrusted source, and a recursive-descent parser is exactly where that
//!   bites: nesting is capped so a hostile file cannot exhaust the stack.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Deepest nesting the parser will follow before refusing the document.
///
/// Real material documents nest three or four levels. A thousand is far past
/// anything legitimate and far below what would overflow the stack.
pub const MAX_DEPTH: usize = 64;

/// Largest document the parser will accept, in bytes.
pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

/// One JSON value.
///
/// Integers and floats are separate variants even though JSON has one number
/// type: a resolution written as `1024.0` reads as a mistake, and keeping the
/// distinction is what makes `write → read → write` produce identical bytes.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number with no fractional part, written without a decimal point.
    Integer(i64),
    /// A number with a fractional part, always written with one.
    Float(f64),
    /// A string.
    Text(String),
    /// An ordered list.
    Array(Vec<Json>),
    /// A mapping, held sorted by key so serialization is deterministic.
    Object(BTreeMap<String, Json>),
}

impl Json {
    /// Build an object from key-value pairs.
    #[must_use]
    pub fn object<I, K>(fields: I) -> Self
    where
        I: IntoIterator<Item = (K, Self)>,
        K: Into<String>,
    {
        Self::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )
    }

    /// Build a string value.
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// The name of this variant, for error messages.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "boolean",
            Self::Integer(_) | Self::Float(_) => "number",
            Self::Text(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }

    /// Read a required field from an object.
    ///
    /// # Errors
    ///
    /// Returns an error when this value is not an object, or the field is
    /// absent. A missing field is refused rather than defaulted: a material
    /// that quietly acquires a default roughness is a material nobody can
    /// explain later.
    pub fn field(&self, name: &str) -> Result<&Self> {
        let Self::Object(fields) = self else {
            return Err(type_error("object", self.kind()).with_context("field", name.to_owned()));
        };
        fields
            .get(name)
            .ok_or_else(|| missing_field(name).with_context("present", present_keys(fields)))
    }

    /// Read an optional field from an object.
    ///
    /// # Errors
    ///
    /// Returns an error when this value is not an object.
    pub fn optional_field(&self, name: &str) -> Result<Option<&Self>> {
        let Self::Object(fields) = self else {
            return Err(type_error("object", self.kind()).with_context("field", name.to_owned()));
        };
        Ok(fields.get(name))
    }

    /// This value as a string.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not a string.
    pub fn as_text(&self) -> Result<&str> {
        match self {
            Self::Text(value) => Ok(value),
            other => Err(type_error("string", other.kind())),
        }
    }

    /// This value as a boolean.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not a boolean.
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(value) => Ok(*value),
            other => Err(type_error("boolean", other.kind())),
        }
    }

    /// This value as a signed integer.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an integer.
    pub fn as_integer(&self) -> Result<i64> {
        match self {
            Self::Integer(value) => Ok(*value),
            other => Err(type_error("integer", other.kind())),
        }
    }

    /// This value as an unsigned 32-bit integer.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an integer, or does not fit.
    pub fn as_u32(&self) -> Result<u32> {
        let value = self.as_integer()?;
        u32::try_from(value).map_err(|_| {
            Error::new(Domain::Content, "json", "integer is out of range")
                .with_recovery(Recovery::Reject)
                .with_context("value", value.to_string())
                .with_context("expected", "0..=4294967295")
        })
    }

    /// This value as a float, accepting an integer.
    ///
    /// A roughness written as `1` rather than `1.0` is the same number, and
    /// refusing it would be pedantry rather than strictness.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not a number.
    pub fn as_f64(&self) -> Result<f64> {
        match self {
            Self::Float(value) => Ok(*value),
            #[allow(clippy::cast_precision_loss)]
            Self::Integer(value) => Ok(*value as f64),
            other => Err(type_error("number", other.kind())),
        }
    }

    /// This value as an array.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an array.
    pub fn as_array(&self) -> Result<&[Self]> {
        match self {
            Self::Array(items) => Ok(items),
            other => Err(type_error("array", other.kind())),
        }
    }

    /// Render the value as compact JSON text.
    #[must_use]
    pub fn to_compact(&self) -> String {
        let mut out = String::new();
        self.write_compact(&mut out);
        out
    }

    /// Render the value as indented JSON text, with a trailing newline.
    ///
    /// This is the form written to disk: a material document is reviewed in a
    /// diff, and a single-line document makes a one-field change unreadable.
    #[must_use]
    pub fn to_pretty(&self) -> String {
        let mut out = String::new();
        self.write_pretty(&mut out, 0);
        out.push('\n');
        out
    }

    fn write_compact(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::Integer(value) => {
                let _ = write!(out, "{value}");
            }
            Self::Float(value) => out.push_str(&format_float(*value)),
            Self::Text(value) => write_string(out, value),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.write_compact(out);
                }
                out.push(']');
            }
            Self::Object(fields) => {
                out.push('{');
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write_string(out, key);
                    out.push(':');
                    value.write_compact(out);
                }
                out.push('}');
            }
        }
    }

    fn write_pretty(&self, out: &mut String, depth: usize) {
        match self {
            Self::Array(items) if !items.is_empty() => {
                out.push_str("[\n");
                for (index, item) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    item.write_pretty(out, depth + 1);
                    if index + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push(']');
            }
            Self::Object(fields) if !fields.is_empty() => {
                out.push_str("{\n");
                for (index, (key, value)) in fields.iter().enumerate() {
                    indent(out, depth + 1);
                    write_string(out, key);
                    out.push_str(": ");
                    value.write_pretty(out, depth + 1);
                    if index + 1 < fields.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push('}');
            }
            other => other.write_compact(out),
        }
    }
}

/// Parse a JSON document.
///
/// # Errors
///
/// Returns an error when the text is too long, nested too deeply, or is not
/// valid JSON. Every error carries the byte offset where parsing stopped.
pub fn parse(text: &str) -> Result<Json> {
    if text.len() > MAX_DOCUMENT_BYTES {
        return Err(
            Error::new(Domain::Content, "json", "document exceeds the size limit")
                .with_recovery(Recovery::Reject)
                .with_context("bytes", text.len().to_string())
                .with_context("limit", MAX_DOCUMENT_BYTES.to_string()),
        );
    }
    let mut parser = Parser {
        bytes: text.as_bytes(),
        at: 0,
    };
    parser.skip_whitespace();
    let value = parser.value(0)?;
    parser.skip_whitespace();
    if parser.at < parser.bytes.len() {
        return Err(parser.error("trailing content after the document"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn error(&self, message: &'static str) -> Error {
        Error::new(Domain::Content, "json", message)
            .with_recovery(Recovery::Reject)
            .with_context("offset", self.at.to_string())
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn expect(&mut self, byte: u8, message: &'static str) -> Result<()> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn literal(&mut self, word: &[u8], value: Json) -> Result<Json> {
        if self.bytes[self.at..].starts_with(word) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(self.error("unrecognised literal"))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json> {
        if depth > MAX_DEPTH {
            return Err(self
                .error("document is nested too deeply")
                .with_context("limit", MAX_DEPTH.to_string()));
        }
        match self.peek() {
            None => Err(self.error("unexpected end of document")),
            Some(b'n') => self.literal(b"null", Json::Null),
            Some(b't') => self.literal(b"true", Json::Bool(true)),
            Some(b'f') => self.literal(b"false", Json::Bool(false)),
            Some(b'"') => self.string().map(Json::Text),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.error("a value cannot start here")),
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json> {
        self.expect(b'[', "expected `[`")?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json> {
        self.expect(b'{', "expected `{`")?;
        let mut fields = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':', "expected `:` after a key")?;
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            // A duplicate key means the document says two things; picking one
            // silently is how a material ends up with a roughness nobody wrote.
            if fields.insert(key.clone(), value).is_some() {
                return Err(self
                    .error("duplicate key in an object")
                    .with_context("key", key));
            }
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Json::Object(fields));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn number(&mut self) -> Result<Json> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        let integer_start = self.at;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
        if self.at == integer_start {
            return Err(self.error("a number needs at least one digit"));
        }
        // JSON forbids a leading zero. Accepting `01` would also mean accepting
        // `0755` as 755, which is the kind of near-miss this parser exists to
        // refuse rather than reinterpret.
        if self.at - integer_start > 1 && self.bytes[integer_start] == b'0' {
            return Err(self.error("a number must not have a leading zero"));
        }
        let mut fractional = false;
        if self.peek() == Some(b'.') {
            fractional = true;
            self.at += 1;
            let digits = self.at;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.at += 1;
            }
            if self.at == digits {
                return Err(self.error("a decimal point needs digits after it"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            fractional = true;
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            let digits = self.at;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.at += 1;
            }
            if self.at == digits {
                return Err(self.error("an exponent needs digits"));
            }
        }
        let text = core::str::from_utf8(&self.bytes[start..self.at])
            .map_err(|_| self.error("number is not valid text"))?;
        if !fractional {
            if let Ok(value) = text.parse::<i64>() {
                return Ok(Json::Integer(value));
            }
        }
        let value: f64 = text
            .parse()
            .map_err(|_| self.error("number cannot be represented"))?;
        if !value.is_finite() {
            return Err(self.error("number is not finite"));
        }
        Ok(Json::Float(value))
    }

    fn string(&mut self) -> Result<String> {
        self.expect(b'"', "expected a string")?;
        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.error("unterminated string"));
            };
            match byte {
                b'"' => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.at += 1;
                    self.escape(&mut out)?;
                }
                // JSON forbids raw control characters inside a string.
                0x00..=0x1F => return Err(self.error("raw control character in a string")),
                _ => {
                    let rest = core::str::from_utf8(&self.bytes[self.at..])
                        .map_err(|_| self.error("string is not valid UTF-8"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("unterminated string"))?;
                    out.push(character);
                    self.at += character.len_utf8();
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<()> {
        let Some(byte) = self.peek() else {
            return Err(self.error("unterminated escape"));
        };
        self.at += 1;
        let simple = match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode_escape(out),
            _ => return Err(self.error("unrecognised escape")),
        };
        out.push(simple);
        Ok(())
    }

    fn unicode_escape(&mut self, out: &mut String) -> Result<()> {
        let first = self.hex4()?;
        // A high surrogate is only half a character; the low half must follow,
        // or the document is claiming a code point that does not exist.
        let code = if (0xD800..0xDC00).contains(&first) {
            if !self.bytes[self.at..].starts_with(b"\\u") {
                return Err(self.error("high surrogate without a low surrogate"));
            }
            self.at += 2;
            let second = self.hex4()?;
            if !(0xDC00..0xE000).contains(&second) {
                return Err(self.error("high surrogate followed by a non-surrogate"));
            }
            0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00)
        } else if (0xDC00..0xE000).contains(&first) {
            return Err(self.error("low surrogate without a high surrogate"));
        } else {
            first
        };
        let character =
            char::from_u32(code).ok_or_else(|| self.error("escape names no code point"))?;
        out.push(character);
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32> {
        let end = self.at + 4;
        if end > self.bytes.len() {
            return Err(self.error("truncated `\\u` escape"));
        }
        let mut value = 0u32;
        for &byte in &self.bytes[self.at..end] {
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                b'a'..=b'f' => u32::from(byte - b'a') + 10,
                b'A'..=b'F' => u32::from(byte - b'A') + 10,
                _ => return Err(self.error("`\\u` escape needs four hex digits")),
            };
            value = (value << 4) | digit;
        }
        self.at = end;
        Ok(value)
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            // Everything else goes out as UTF-8. JSON permits it, and escaping
            // accented text would make a human-reviewed document unreadable.
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Format a float so that reading it back yields a float again.
///
/// `{}` renders `1.0` as `1`, which would parse back as an integer and break
/// the round trip that every document test depends on.
fn format_float(value: f64) -> String {
    let rendered = format!("{value}");
    if rendered.contains(['.', 'e', 'E']) {
        rendered
    } else {
        format!("{rendered}.0")
    }
}

fn type_error(expected: &'static str, found: &'static str) -> Error {
    Error::new(Domain::Content, "json", "value has the wrong type")
        .with_recovery(Recovery::Reject)
        .with_context("expected", expected)
        .with_context("found", found)
}

fn missing_field(name: &str) -> Error {
    Error::new(Domain::Content, "json", "a required field is absent")
        .with_recovery(Recovery::Reject)
        .with_context("field", name.to_owned())
}

fn present_keys(fields: &BTreeMap<String, Json>) -> String {
    fields.keys().cloned().collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(text: &str) -> String {
        parse(text).expect("valid JSON").to_compact()
    }

    #[test]
    fn parses_and_re_emits_every_value_kind() {
        assert_eq!(round_trip("null"), "null");
        assert_eq!(round_trip("true"), "true");
        assert_eq!(round_trip(" false "), "false");
        assert_eq!(round_trip("-17"), "-17");
        assert_eq!(round_trip("0.5"), "0.5");
        assert_eq!(round_trip("\"text\""), "\"text\"");
        assert_eq!(round_trip("[1,2,3]"), "[1,2,3]");
        assert_eq!(round_trip("{\"a\":1}"), "{\"a\":1}");
        assert_eq!(round_trip("[]"), "[]");
        assert_eq!(round_trip("{}"), "{}");
    }

    #[test]
    fn object_keys_are_emitted_in_a_stable_order() {
        // Same content, opposite input order: the bytes must match, because a
        // provenance hash is taken over these bytes.
        let forward = round_trip(r#"{"alpha":1,"beta":2,"gamma":3}"#);
        let reverse = round_trip(r#"{"gamma":3,"beta":2,"alpha":1}"#);
        assert_eq!(forward, reverse);
        assert_eq!(forward, r#"{"alpha":1,"beta":2,"gamma":3}"#);
    }

    #[test]
    fn a_whole_number_float_survives_the_round_trip_as_a_float() {
        // `{}` would render this as `1`, which parses back as an integer.
        let value = Json::Float(1.0);
        assert_eq!(value.to_compact(), "1.0");
        assert_eq!(parse("1.0").unwrap(), Json::Float(1.0));
        assert_eq!(parse("1").unwrap(), Json::Integer(1));
    }

    #[test]
    fn pretty_output_reparses_to_the_same_value() {
        let source = r#"{"b":[1,2,{"c":true}],"a":"x"}"#;
        let value = parse(source).unwrap();
        let pretty = value.to_pretty();
        assert!(pretty.ends_with('\n'));
        assert!(pretty.contains("\n  \"a\""), "{pretty}");
        assert_eq!(parse(&pretty).unwrap(), value);
    }

    #[test]
    fn strings_handle_escapes_including_surrogate_pairs() {
        assert_eq!(parse(r#""a\nb""#).unwrap(), Json::text("a\nb"));
        assert_eq!(parse(r#""\u0041""#).unwrap(), Json::text("A"));
        // U+1F5FF, expressed as a surrogate pair.
        assert_eq!(parse(r#""\ud83d\uddff""#).unwrap(), Json::text("\u{1F5FF}"));
        // Non-ASCII goes out raw rather than escaped, so a review stays readable.
        assert_eq!(Json::text("pedra áspera").to_compact(), "\"pedra áspera\"");
    }

    #[test]
    fn malformed_documents_are_refused_not_guessed_at() {
        for bad in [
            "",
            "{",
            "[1,]",
            "{\"a\":1,}",
            "{a:1}",
            "'text'",
            "01",
            "1.",
            "1e",
            "nul",
            "NaN",
            "Infinity",
            "\"unterminated",
            "[1] junk",
            "// comment\n1",
            "\"\\q\"",
            "\"\\u00\"",
            "\"\\ud83d\"",
            "\"\\udc00\"",
            "\"raw\ttab\"",
        ] {
            let err = parse(bad).expect_err("must reject {bad}");
            assert_eq!(err.recovery(), Recovery::Reject, "input {bad:?}");
        }
    }

    #[test]
    fn a_duplicate_key_is_an_error_rather_than_a_silent_winner() {
        let err = parse(r#"{"a":1,"a":2}"#).expect_err("duplicate keys must be refused");
        assert!(err.to_string().contains('a'), "{err}");
    }

    #[test]
    fn nesting_is_bounded_so_a_hostile_document_cannot_exhaust_the_stack() {
        let deep = format!(
            "{}1{}",
            "[".repeat(MAX_DEPTH + 8),
            "]".repeat(MAX_DEPTH + 8)
        );
        let err = parse(&deep).expect_err("must refuse to recurse this far");
        assert!(err.to_string().contains("deep"), "{err}");

        // One below the limit still parses, so the cap is not off by an order.
        let shallow = format!(
            "{}1{}",
            "[".repeat(MAX_DEPTH - 1),
            "]".repeat(MAX_DEPTH - 1)
        );
        assert!(parse(&shallow).is_ok());
    }

    #[test]
    fn oversized_documents_are_refused_before_parsing() {
        let big = format!("\"{}\"", "x".repeat(MAX_DOCUMENT_BYTES));
        let err = parse(&big).expect_err("must refuse an oversized document");
        assert!(err.to_string().contains("size limit"), "{err}");
    }

    #[test]
    fn accessors_report_what_they_wanted_and_what_they_found() {
        let value = parse(r#"{"name":"stone","size":16,"rough":0.8,"on":true}"#).unwrap();
        assert_eq!(value.field("name").unwrap().as_text().unwrap(), "stone");
        assert_eq!(value.field("size").unwrap().as_u32().unwrap(), 16);
        assert!((value.field("rough").unwrap().as_f64().unwrap() - 0.8).abs() < f64::EPSILON);
        assert!(value.field("on").unwrap().as_bool().unwrap());
        // An integer is an acceptable float; a float is not an acceptable integer.
        assert!((parse("2").unwrap().as_f64().unwrap() - 2.0).abs() < f64::EPSILON);
        assert!(parse("2.5").unwrap().as_integer().is_err());

        let err = value
            .field("absent")
            .expect_err("missing field must be named");
        assert!(err.to_string().contains("absent"), "{err}");
        // The error lists what was there, which is what makes a typo findable.
        assert!(err.to_string().contains("name"), "{err}");

        assert!(value.optional_field("absent").unwrap().is_none());
        assert!(value.optional_field("name").unwrap().is_some());

        let err = value.field("name").unwrap().as_u32().unwrap_err();
        assert!(err.to_string().contains("string"), "{err}");
    }

    #[test]
    fn negative_and_out_of_range_integers_do_not_become_u32() {
        assert!(parse("-1").unwrap().as_u32().is_err());
        assert!(parse("5000000000").unwrap().as_u32().is_err());
        assert_eq!(parse("4294967295").unwrap().as_u32().unwrap(), u32::MAX);
    }

    #[test]
    fn the_object_helper_builds_what_the_parser_would_have() {
        let built = Json::object([
            ("id", Json::text("nexora:material/stone")),
            ("resolution", Json::Integer(64)),
        ]);
        assert_eq!(
            built,
            parse(r#"{"id":"nexora:material/stone","resolution":64}"#).unwrap()
        );
        assert_eq!(built.kind(), "object");
    }
}
