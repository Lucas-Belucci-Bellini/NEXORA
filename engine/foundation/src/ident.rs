//! Namespaced identifiers.
//!
//! Implements `NEXORA NAMING AND TERMINOLOGY.md`: logical IDs never depend on
//! file names, official content lives under `nexora:`, and every mod owns its
//! own namespace. The canonical text form is `namespace:path`, e.g.
//! `nexora:block/stone`.

use core::fmt;

use crate::error::{Domain, Error, Recovery, Result};

/// Namespace reserved for first-party NEXORA content.
pub const NEXORA_NAMESPACE: &str = "nexora";

/// Longest accepted namespace, in bytes.
pub const MAX_NAMESPACE_LEN: usize = 64;

/// Longest accepted path, in bytes.
pub const MAX_PATH_LEN: usize = 192;

/// An owner of identifiers: first-party content, a mod, or the engine itself.
///
/// Namespaces are lowercase ASCII so that identifiers compare and sort
/// identically on every platform and in every save file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace(String);

impl Namespace {
    /// Parse and validate a namespace.
    ///
    /// # Errors
    ///
    /// Returns an error when the namespace is empty, too long, or contains
    /// anything other than `[a-z0-9_]`.
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.is_empty() {
            return Err(invalid("namespace", raw, "namespace must not be empty"));
        }
        if raw.len() > MAX_NAMESPACE_LEN {
            return Err(invalid("namespace", raw, "namespace is too long"));
        }
        if let Some(bad) = raw
            .chars()
            .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_'))
        {
            return Err(invalid("namespace", raw, "namespace allows only [a-z0-9_]")
                .with_context("character", bad.to_string()));
        }
        Ok(Self(raw.to_owned()))
    }

    /// The first-party namespace.
    #[must_use]
    pub fn nexora() -> Self {
        Self(NEXORA_NAMESPACE.to_owned())
    }

    /// The namespace text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this is the first-party namespace.
    ///
    /// Content ownership checks use this: a mod must not register into
    /// `nexora:` (`Registry System.md` §33).
    #[must_use]
    pub fn is_first_party(&self) -> bool {
        self.0 == NEXORA_NAMESPACE
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A fully qualified logical identifier, `namespace:path`.
///
/// Identifiers are stable across runs and across saves. They are the only
/// legitimate way to name content; array indices and runtime IDs are not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    namespace: Namespace,
    path: String,
}

impl Identifier {
    /// Build an identifier from an already-validated namespace and a raw path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty, too long, contains characters
    /// outside `[a-z0-9_/.-]`, or uses `/` in a way that yields an empty segment.
    pub fn new(namespace: Namespace, path: &str) -> Result<Self> {
        validate_path(path)?;
        Ok(Self {
            namespace,
            path: path.to_owned(),
        })
    }

    /// Parse the canonical `namespace:path` text form.
    ///
    /// # Errors
    ///
    /// Returns an error when the separator is missing or either half is invalid.
    pub fn parse(raw: &str) -> Result<Self> {
        let Some((namespace, path)) = raw.split_once(':') else {
            return Err(invalid(
                "identifier",
                raw,
                "identifier must be `namespace:path`",
            ));
        };
        // Guard against `a:b:c`: a second colon means the caller built the string wrong.
        if path.contains(':') {
            return Err(invalid(
                "identifier",
                raw,
                "identifier must contain exactly one `:`",
            ));
        }
        Ok(Self {
            namespace: Namespace::parse(namespace)?,
            path: {
                validate_path(path)?;
                path.to_owned()
            },
        })
    }

    /// Build a first-party `nexora:` identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is invalid.
    pub fn nexora(path: &str) -> Result<Self> {
        Self::new(Namespace::nexora(), path)
    }

    /// The owning namespace.
    #[must_use]
    pub const fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// The path within the namespace.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(invalid("path", path, "path must not be empty"));
    }
    if path.len() > MAX_PATH_LEN {
        return Err(invalid("path", path, "path is too long"));
    }
    if let Some(bad) = path
        .chars()
        .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '_' | '/' | '.' | '-'))
    {
        return Err(invalid("path", path, "path allows only [a-z0-9_/.-]")
            .with_context("character", bad.to_string()));
    }
    if path.split('/').any(str::is_empty) {
        return Err(invalid(
            "path",
            path,
            "path must not contain an empty segment",
        ));
    }
    Ok(())
}

fn invalid(what: &'static str, raw: &str, message: &'static str) -> Error {
    Error::new(Domain::Content, "identifier", message)
        .with_recovery(Recovery::Reject)
        .with_context("kind", what)
        .with_context("input", raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_form_round_trip() {
        let id = Identifier::parse("nexora:block/stone").expect("valid identifier");
        assert_eq!(id.namespace().as_str(), "nexora");
        assert_eq!(id.path(), "block/stone");
        assert_eq!(id.to_string(), "nexora:block/stone");
    }

    #[test]
    fn mod_namespaces_are_not_first_party() {
        let id = Identifier::parse("example:block/ruby_ore").expect("valid identifier");
        assert!(!id.namespace().is_first_party());
        assert!(Identifier::nexora("block/stone")
            .unwrap()
            .namespace()
            .is_first_party());
    }

    #[test]
    fn rejects_malformed_identifiers() {
        for raw in [
            "",
            "nostone",
            ":stone",
            "nexora:",
            "NEXORA:stone",
            "nexora:Stone",
            "nexora:block//stone",
            "nexora:block stone",
            "nexora:block:stone",
        ] {
            let err = Identifier::parse(raw).expect_err("must reject {raw}");
            assert_eq!(err.recovery(), Recovery::Reject, "input {raw:?}");
        }
    }

    #[test]
    fn enforces_length_limits() {
        let long_path = "a".repeat(MAX_PATH_LEN + 1);
        assert!(Identifier::nexora(&long_path).is_err());
        let ok_path = "a".repeat(MAX_PATH_LEN);
        assert!(Identifier::nexora(&ok_path).is_ok());

        let long_ns = "n".repeat(MAX_NAMESPACE_LEN + 1);
        assert!(Namespace::parse(&long_ns).is_err());
    }

    #[test]
    fn ordering_is_stable_and_lexicographic() {
        let mut ids = [
            Identifier::parse("nexora:block/stone").unwrap(),
            Identifier::parse("example:block/ruby").unwrap(),
            Identifier::parse("nexora:block/dirt").unwrap(),
        ];
        ids.sort();
        let rendered: Vec<String> = ids.iter().map(ToString::to_string).collect();
        assert_eq!(
            rendered,
            [
                "example:block/ruby",
                "nexora:block/dirt",
                "nexora:block/stone"
            ]
        );
    }
}
