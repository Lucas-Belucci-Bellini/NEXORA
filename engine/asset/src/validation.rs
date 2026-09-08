//! Validation verdicts: what was checked, what was found, and how bad it is.
//!
//! The Texture Forge brief §7 and §8 ask for a formal result with three
//! outcomes and *"an explanation for each"*, and state the rule that gives the
//! module its shape: **do not silently accept an invalid texture.**
//!
//! So a verdict is not a boolean and not an `Option`. It is a list of findings,
//! each naming the check that produced it, and [`TextureValidationResult::ok`]
//! turns a failing one into an [`Error`] — a caller that ignores the result has
//! to ignore a `#[must_use]` value to do it.
//!
//! # The checks live in the tool; the verdict lives here
//!
//! Deciding whether a normal map's vectors are unit length means reading
//! pixels, and reading pixels is the forge's job. But a *result* has to cross
//! the boundary — the registry refuses a material whose maps failed, and the
//! registry is engine code. So the vocabulary is here and the arithmetic is
//! not.

use std::fmt;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

use crate::texture::MapRole;

/// Which check produced a finding.
///
/// The list from the brief §8, one variant per check, so a finding can never
/// be traced back to "something went wrong somewhere".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Check {
    /// The file the material refers to is present.
    FileExists,
    /// The bytes are the format the extension claims.
    Format,
    /// The resolution is the one the material declares.
    Resolution,
    /// Width and height are legal and powers of two.
    Dimensions,
    /// The channel layout matches the map's role.
    Channels,
    /// A normal map's vectors are unit length and point outwards.
    NormalMap,
    /// Values are inside the range their role permits.
    ValueRange,
    /// The file decodes at all.
    Corruption,
    /// The document carries the metadata the schema requires.
    Metadata,
    /// Identifiers and file names follow the naming rules.
    Naming,
    /// Opposite edges match, so the texture tiles without a seam.
    Seamless,
    /// The maps in the set agree with each other and with the definition.
    MaterialIntegrity,
    /// A map the material needs is absent.
    MissingMap,
    /// The result is something the runtime can actually load.
    RuntimeCompatibility,
}

impl Check {
    /// Every check, in a stable order.
    pub const ALL: [Self; 14] = [
        Self::FileExists,
        Self::Format,
        Self::Resolution,
        Self::Dimensions,
        Self::Channels,
        Self::NormalMap,
        Self::ValueRange,
        Self::Corruption,
        Self::Metadata,
        Self::Naming,
        Self::Seamless,
        Self::MaterialIntegrity,
        Self::MissingMap,
        Self::RuntimeCompatibility,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileExists => "file_exists",
            Self::Format => "format",
            Self::Resolution => "resolution",
            Self::Dimensions => "dimensions",
            Self::Channels => "channels",
            Self::NormalMap => "normal_map",
            Self::ValueRange => "value_range",
            Self::Corruption => "corruption",
            Self::Metadata => "metadata",
            Self::Naming => "naming",
            Self::Seamless => "seamless",
            Self::MaterialIntegrity => "material_integrity",
            Self::MissingMap => "missing_map",
            Self::RuntimeCompatibility => "runtime_compatibility",
        }
    }
}

/// How serious one finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Worth knowing, changes nothing.
    Note,
    /// Usable, but somebody should look.
    Warning,
    /// Not usable.
    Failure,
}

impl Severity {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Failure => "failure",
        }
    }
}

/// The overall outcome of validating something.
///
/// `PASS`, `WARN`, `FAIL` from the brief, derived from the worst finding rather
/// than set independently — a result cannot claim to pass while holding a
/// failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verdict {
    /// Nothing worth reporting.
    Pass,
    /// Usable, with something to look at.
    Warn,
    /// Not usable.
    Fail,
}

impl Verdict {
    /// Stable uppercase name, as the brief writes it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }

    /// Whether this verdict permits the thing to be used.
    #[must_use]
    pub const fn is_usable(self) -> bool {
        !matches!(self, Self::Fail)
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One thing a validator noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Which check noticed it.
    pub check: Check,
    /// How serious it is.
    pub severity: Severity,
    /// Which map it concerns, when it concerns one.
    pub map: Option<MapRole>,
    /// What is wrong, in words a person can act on.
    ///
    /// The brief asks for an explanation with every verdict. A finding whose
    /// message is "invalid" is a finding that costs an hour to investigate.
    pub message: String,
}

impl Finding {
    /// A failing finding.
    #[must_use]
    pub fn failure(check: Check, message: impl Into<String>) -> Self {
        Self {
            check,
            severity: Severity::Failure,
            map: None,
            message: message.into(),
        }
    }

    /// A warning finding.
    #[must_use]
    pub fn warning(check: Check, message: impl Into<String>) -> Self {
        Self {
            check,
            severity: Severity::Warning,
            map: None,
            message: message.into(),
        }
    }

    /// A note.
    #[must_use]
    pub fn note(check: Check, message: impl Into<String>) -> Self {
        Self {
            check,
            severity: Severity::Note,
            map: None,
            message: message.into(),
        }
    }

    /// Attach the map this finding concerns.
    #[must_use]
    pub const fn about(mut self, map: MapRole) -> Self {
        self.map = Some(map);
        self
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.check.as_str(), self.message)?;
        if let Some(map) = self.map {
            write!(f, " (map: {})", map.as_str())?;
        }
        Ok(())
    }
}

/// Everything one validation run found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[must_use = "a validation result that is discarded is a texture that was never validated"]
pub struct TextureValidationResult {
    findings: Vec<Finding>,
}

impl TextureValidationResult {
    /// A result with nothing in it yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a finding.
    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Record a finding, chaining.
    pub fn with(mut self, finding: Finding) -> Self {
        self.push(finding);
        self
    }

    /// Absorb another run's findings.
    pub fn merge(&mut self, other: Self) {
        self.findings.extend(other.findings);
    }

    /// Everything found, in the order it was found.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The overall outcome.
    #[must_use]
    pub fn verdict(&self) -> Verdict {
        match self.findings.iter().map(|f| f.severity).max() {
            Some(Severity::Failure) => Verdict::Fail,
            Some(Severity::Warning) => Verdict::Warn,
            // A note is not a warning: it exists to be read, not to be acted on.
            Some(Severity::Note) | None => Verdict::Pass,
        }
    }

    /// Whether nothing failed.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.verdict().is_usable()
    }

    /// Only the findings at or above a severity.
    pub fn at_least(&self, severity: Severity) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(move |finding| finding.severity >= severity)
    }

    /// Turn a failing result into an error.
    ///
    /// This is the rule *"do not silently accept an invalid texture"* expressed
    /// as a type: the only way to proceed past a failure is to write the call
    /// that ignores this, which is visible in review.
    ///
    /// # Errors
    ///
    /// Returns an error listing every failure when the verdict is
    /// [`Verdict::Fail`].
    pub fn ok(&self, subject: &str) -> Result<()> {
        if self.is_usable() {
            return Ok(());
        }
        let failures: Vec<String> = self
            .at_least(Severity::Failure)
            .map(ToString::to_string)
            .collect();
        Err(
            Error::new(Domain::Content, "validation", "the asset failed validation")
                .with_recovery(Recovery::Reject)
                .with_context("subject", subject.to_owned())
                .with_context("failures", failures.len().to_string())
                .with_context("detail", failures.join("; ")),
        )
    }
}

impl fmt::Display for TextureValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.verdict())?;
        for finding in &self.findings {
            write!(f, "\n  {finding}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_result_passes() {
        let result = TextureValidationResult::new();
        assert_eq!(result.verdict(), Verdict::Pass);
        assert!(result.is_usable());
        result.ok("nexora:material/stone").expect("nothing failed");
        assert!(result.findings().is_empty());
    }

    #[test]
    fn the_verdict_is_the_worst_finding_not_the_last_one() {
        let result = TextureValidationResult::new()
            .with(Finding::failure(
                Check::Seamless,
                "left edge does not meet right",
            ))
            .with(Finding::warning(
                Check::ValueRange,
                "roughness is nearly flat",
            ))
            .with(Finding::note(Check::Metadata, "no notes recorded"));
        // The failure came first and a note came last; the verdict is still FAIL.
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(!result.is_usable());
    }

    #[test]
    fn a_note_alone_does_not_lower_the_verdict() {
        let result =
            TextureValidationResult::new().with(Finding::note(Check::Metadata, "generated today"));
        assert_eq!(result.verdict(), Verdict::Pass);

        let warned =
            TextureValidationResult::new().with(Finding::warning(Check::Naming, "unusual name"));
        assert_eq!(warned.verdict(), Verdict::Warn);
        assert!(warned.is_usable());
    }

    #[test]
    fn a_failing_result_becomes_an_error_that_names_every_failure() {
        let result = TextureValidationResult::new()
            .with(
                Finding::failure(Check::Dimensions, "height is 100, not a power of two")
                    .about(MapRole::Albedo),
            )
            .with(Finding::failure(Check::MissingMap, "albedo is absent"))
            .with(Finding::warning(Check::Seamless, "faint vertical seam"));

        let err = result
            .ok("nexora:material/stone")
            .expect_err("a failing result must not be silently usable");
        assert_eq!(err.recovery(), Recovery::Reject);
        let rendered = err.to_string();
        assert!(rendered.contains("nexora:material/stone"), "{rendered}");
        assert!(rendered.contains("power of two"), "{rendered}");
        assert!(rendered.contains("albedo is absent"), "{rendered}");
        // Two failures, and the warning is not counted among them.
        assert!(rendered.contains('2'), "{rendered}");
    }

    #[test]
    fn a_finding_says_which_check_and_which_map() {
        let finding = Finding::failure(Check::NormalMap, "vectors are not unit length")
            .about(MapRole::Normal);
        let rendered = finding.to_string();
        assert!(rendered.contains("[normal_map]"), "{rendered}");
        assert!(rendered.contains("unit length"), "{rendered}");
        assert!(rendered.contains("map: normal"), "{rendered}");
        assert_eq!(finding.severity, Severity::Failure);
    }

    #[test]
    fn merging_keeps_every_finding_and_the_worst_verdict() {
        let mut first =
            TextureValidationResult::new().with(Finding::warning(Check::Naming, "odd name"));
        let second =
            TextureValidationResult::new().with(Finding::failure(Check::Corruption, "truncated"));

        first.merge(second);
        assert_eq!(first.findings().len(), 2);
        assert_eq!(first.verdict(), Verdict::Fail);
        assert_eq!(first.at_least(Severity::Failure).count(), 1);
        assert_eq!(first.at_least(Severity::Warning).count(), 2);
        assert_eq!(first.at_least(Severity::Note).count(), 2);
    }

    #[test]
    fn every_check_and_severity_has_a_distinct_stable_name() {
        let mut names: Vec<&str> = Check::ALL.iter().map(|check| check.as_str()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "check names must be distinct");

        assert_eq!(Verdict::Pass.as_str(), "PASS");
        assert_eq!(Verdict::Fail.to_string(), "FAIL");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert!(Verdict::Warn.is_usable());
        assert!(!Verdict::Fail.is_usable());
    }

    #[test]
    fn the_display_form_shows_the_verdict_and_then_the_detail() {
        let result = TextureValidationResult::new().with(Finding::warning(
            Check::Seamless,
            "faint seam on the v axis",
        ));
        let rendered = result.to_string();
        assert!(rendered.starts_with("WARN"), "{rendered}");
        assert!(rendered.contains("faint seam"), "{rendered}");
    }
}
