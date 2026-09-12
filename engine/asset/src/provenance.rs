//! Where an asset came from, and whether it may ship.
//!
//! Implements `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md` and the release
//! gate in `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`. The vocabulary here
//! is **theirs**, not a new one: those two documents already define eight
//! provenance classes, six release states, six asset statuses and the license
//! fields, which is more than any generator needs and exactly what a release
//! review needs.
//!
//! # Why generated content still carries all of this
//!
//! The policy is explicit that *"automated generation does not bypass
//! provenance review"*. A procedurally generated texture is not exempt from
//! recording what made it — it is the case where recording is easiest and
//! therefore least excusable to skip. So [`Provenance`] is not optional on a
//! generated material: the type that carries generated pixels cannot be built
//! without one.
//!
//! # Why the timestamp is not part of the content hash
//!
//! Regenerating a material from an unchanged definition must produce identical
//! bytes — that is what makes the whole system auditable. But the record of
//! *when* it was generated legitimately differs between runs. So
//! [`Provenance::content_hash`] covers the asset and the generation trace and
//! deliberately excludes [`Provenance::recorded_at`]: two runs a year apart
//! agree on the hash and disagree on the date, which is the truth.

use std::collections::BTreeMap;
use std::fmt;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::Fnv1a64;
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::{ContentGeneratorVersion, ContentPipelineVersion};

/// A wall-clock instant, in seconds since the Unix epoch.
///
/// Deliberately **not** in `nexora_foundation::time`. That module states its own
/// invariant plainly — *"simulation reads world time, never wall-clock time. A
/// system that samples the host clock cannot be replayed"* — and a type that
/// samples the host clock does not belong inside it. Authoring provenance is
/// not simulation, and this is the one place the host clock is the right
/// answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(pub i64);

impl Timestamp {
    /// The Unix epoch.
    pub const EPOCH: Self = Self(0);

    /// Read the host clock.
    ///
    /// Returns [`Self::EPOCH`] if the host clock is set before 1970, which is a
    /// misconfigured machine rather than a condition worth an error path.
    #[must_use]
    pub fn now() -> Self {
        let elapsed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        Self(elapsed.map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        }))
    }

    /// Render as RFC 3339 UTC, e.g. `2026-09-08T14:03:21Z`.
    ///
    /// Provenance is read by people during a release review. Epoch seconds are
    /// a number nobody checks.
    #[must_use]
    pub fn to_rfc3339_utc(self) -> String {
        let (days, seconds_of_day) = (self.0.div_euclid(86_400), self.0.rem_euclid(86_400));
        let (year, month, day) = civil_from_days(days);
        let (hour, minute, second) = (
            seconds_of_day / 3600,
            (seconds_of_day % 3600) / 60,
            seconds_of_day % 60,
        );
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }
    /// Parse `YYYY-MM-DDTHH:MM:SSZ` back into epoch seconds.
    ///
    /// The inverse of [`Self::to_rfc3339_utc`], so a provenance record can be
    /// written in a form a reviewer reads and still round-trip exactly. Only
    /// the `Z` form is accepted: an offset would make two records that name the
    /// same instant compare unequal as text, and provenance is compared as
    /// text more often than as time.
    ///
    /// # Errors
    ///
    /// Returns an error when the shape is wrong or a field is out of range.
    pub fn parse_rfc3339_utc(raw: &str) -> Result<Self> {
        let bytes = raw.as_bytes();
        if bytes.len() != 20
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes[10] != b'T'
            || bytes[13] != b':'
            || bytes[16] != b':'
            || bytes[19] != b'Z'
        {
            return Err(rule("timestamp must be `YYYY-MM-DDTHH:MM:SSZ`")
                .with_context("value", raw.to_owned()));
        }
        let number = |from: usize, to: usize| -> Result<i64> {
            raw[from..to].parse::<i64>().map_err(|_| {
                rule("timestamp field is not a number").with_context("value", raw.to_owned())
            })
        };
        let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
        let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            // 60 is a leap second, which UTC has and Unix time does not.
            || second > 59
        {
            return Err(
                rule("timestamp field is out of range").with_context("value", raw.to_owned())
            );
        }
        let days = days_from_civil(year, month, day);
        Ok(Self(days * 86_400 + hour * 3600 + minute * 60 + second))
    }
}

/// A proleptic Gregorian date to days since 1970-01-01.
///
/// The inverse of [`civil_from_days`], from the same derivation. Tested by
/// round trip rather than against a table: a table would only prove the two
/// agree with the table, while the round trip proves they agree with each
/// other over every day it is run on.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_rfc3339_utc())
    }
}

/// Days since 1970-01-01 to a proleptic Gregorian calendar date.
///
/// Howard Hinnant's `civil_from_days`, which is exact for the whole `i64` range
/// and needs no table. The shift to a 1st-of-March year origin is what removes
/// the leap-day special case: February becomes the last month, so its variable
/// length never falls in the middle of the arithmetic.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    // Shift the epoch from 1970-01-01 to 0000-03-01.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Where an asset came from.
///
/// The list from `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md`, unchanged.
/// The brief for this system proposed a shorter list of its own; a second
/// vocabulary for the same question is how "may we ship this?" becomes
/// unanswerable, so the existing one wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProvenanceClass {
    /// Authored for NEXORA from nothing.
    Original,
    /// Derived from material NEXORA already owns.
    OwnedSource,
    /// Third-party material used under a recorded license.
    LicensedThirdParty,
    /// Material with no remaining copyright restriction.
    PublicDomain,
    /// Produced by a NEXORA editor or tool from operator input.
    EditorGenerated,
    /// Produced by an algorithm from parameters — where the forge lands.
    ProceduralDerivative,
    /// Under evaluation; not a release candidate.
    Experimental,
    /// Refused. Kept so that the refusal itself stays on the record.
    Rejected,
}

/// Distribution status, from `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetStatus {
    /// Original NEXORA content.
    NexoraOriginal,
    /// Derived from NEXORA's own source material.
    NexoraDerivedFromOwnSource,
    /// Third-party content used under license.
    ThirdPartyLicensed,
    /// Usable in the editor, never packaged.
    EditorOnly,
    /// Not cleared for anything yet.
    Experimental,
    /// Must not ship.
    Blocked,
}

/// Where an asset sits in review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReleaseStatus {
    /// Produced, not yet submitted.
    Draft,
    /// Submitted, awaiting a reviewer.
    Review,
    /// Reviewed and permitted in a release artifact.
    Cleared,
    /// Permitted only in specific contexts.
    Restricted,
    /// Refused for release.
    Blocked,
    /// Withdrawn from the current build; the record stays.
    Removed,
}

impl ProvenanceClass {
    /// Every class, in a stable order.
    pub const ALL: [Self; 8] = [
        Self::Original,
        Self::OwnedSource,
        Self::LicensedThirdParty,
        Self::PublicDomain,
        Self::EditorGenerated,
        Self::ProceduralDerivative,
        Self::Experimental,
        Self::Rejected,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::OwnedSource => "owned_source",
            Self::LicensedThirdParty => "licensed_third_party",
            Self::PublicDomain => "public_domain",
            Self::EditorGenerated => "editor_generated",
            Self::ProceduralDerivative => "procedural_derivative",
            Self::Experimental => "experimental",
            Self::Rejected => "rejected",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is unknown. There is no fallback: an
    /// unrecognised provenance class is exactly the "unknown origin" the policy
    /// says to block.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|class| class.as_str() == raw)
            .ok_or_else(|| {
                rule("provenance class is not recognised").with_context("value", raw.to_owned())
            })
    }
}

impl AssetStatus {
    /// Every status, in a stable order.
    pub const ALL: [Self; 6] = [
        Self::NexoraOriginal,
        Self::NexoraDerivedFromOwnSource,
        Self::ThirdPartyLicensed,
        Self::EditorOnly,
        Self::Experimental,
        Self::Blocked,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NexoraOriginal => "nexora_original",
            Self::NexoraDerivedFromOwnSource => "nexora_derived_from_own_source",
            Self::ThirdPartyLicensed => "third_party_licensed",
            Self::EditorOnly => "editor_only",
            Self::Experimental => "experimental",
            Self::Blocked => "blocked",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is unknown.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == raw)
            .ok_or_else(|| {
                rule("asset status is not recognised").with_context("value", raw.to_owned())
            })
    }
}

impl ReleaseStatus {
    /// Every state, in a stable order.
    pub const ALL: [Self; 6] = [
        Self::Draft,
        Self::Review,
        Self::Cleared,
        Self::Restricted,
        Self::Blocked,
        Self::Removed,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Cleared => "cleared",
            Self::Restricted => "restricted",
            Self::Blocked => "blocked",
            Self::Removed => "removed",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is unknown.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|state| state.as_str() == raw)
            .ok_or_else(|| {
                rule("release status is not recognised").with_context("value", raw.to_owned())
            })
    }
}

/// Whether the asset has been changed since it was obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modification {
    /// As produced, never edited afterwards.
    Unmodified,
    /// Obtained elsewhere and changed here.
    Modified,
    /// Produced here from another NEXORA asset, which is named.
    DerivedFrom(Identifier),
}

/// The terms an asset may be distributed under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct License {
    /// The license's name, e.g. `CC0-1.0`.
    pub name: String,
    /// Where the terms can be read.
    pub url: Option<String>,
    /// Whether a notice must accompany distribution.
    pub attribution_required: bool,
    /// Whether the asset may be redistributed at all.
    pub redistribution_allowed: bool,
    /// Whether commercial distribution is permitted.
    pub commercial_use_allowed: bool,
}

/// How the pixels are produced.
///
/// The three from the brief §4. This is recorded rather than inferred: an
/// asset's origin must be answerable without reading the generator's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Backend {
    /// An algorithm, from parameters. Reproducible from the trace alone.
    Procedural,
    /// A model. Reproducible only as far as the model is.
    Ai,
    /// A model's output shaped by an algorithm, or the reverse.
    Hybrid,
}

impl Backend {
    /// Every backend, in a stable order.
    pub const ALL: [Self; 3] = [Self::Procedural, Self::Ai, Self::Hybrid];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Procedural => "procedural",
            Self::Ai => "ai",
            Self::Hybrid => "hybrid",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not a known backend.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|backend| backend.as_str() == raw)
            .ok_or_else(|| rule("backend is not recognised").with_context("value", raw.to_owned()))
    }

    /// Whether output from this backend is reproducible from its trace alone.
    ///
    /// Only the procedural one is. Recording that difference is what stops a
    /// model's output being treated as though re-running the tool would bring
    /// it back.
    #[must_use]
    pub const fn is_reproducible_from_trace(self) -> bool {
        matches!(self, Self::Procedural)
    }
}

/// How a generated asset was produced.
///
/// Answers the question the brief marks as the important one: *"how was this
/// texture created?"* Every field here is an input; together with the generator
/// and its version they are enough to produce the same bytes again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationTrace {
    /// Which generator ran, e.g. `nexora:generator/procedural`.
    pub generator: Identifier,
    /// How it produced the pixels.
    ///
    /// A field rather than an entry in [`Self::parameters`], because
    /// [`Backend::is_reproducible_from_trace`] decides whether this record is
    /// enough to rebuild the asset, and whether the asset may ship without a
    /// person looking at it. A fact that load-bearing is not a string in a map
    /// that any generator may forget to write.
    pub backend: Backend,
    /// That generator's algorithm version.
    pub generator_version: ContentGeneratorVersion,
    /// Which pipeline transformed its output, if any.
    pub pipeline: Option<Identifier>,
    /// That pipeline's version.
    pub pipeline_version: Option<ContentPipelineVersion>,
    /// Which preset supplied the defaults, if any.
    pub preset: Option<Identifier>,
    /// The seed the generator was given. Same seed, same pixels.
    pub seed: u64,
    /// The natural-language request, when a backend takes one.
    ///
    /// Empty for the procedural generator, which takes numbers. Recorded rather
    /// than omitted so an AI-backed material can never be mistaken for one.
    pub prompt: Option<String>,
    /// Every other input, sorted so the record hashes deterministically.
    pub parameters: BTreeMap<String, String>,
    /// Assets this one was built from, if any.
    pub inputs: Vec<Identifier>,
}

impl GenerationTrace {
    /// A trace for a generator that takes parameters and no prompt.
    #[must_use]
    pub fn new(
        generator: Identifier,
        generator_version: ContentGeneratorVersion,
        seed: u64,
    ) -> Self {
        Self {
            generator,
            backend: Backend::Procedural,
            generator_version,
            pipeline: None,
            pipeline_version: None,
            preset: None,
            seed,
            prompt: None,
            parameters: BTreeMap::new(),
            inputs: Vec::new(),
        }
    }

    /// Record one input parameter.
    #[must_use]
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Record which preset supplied the defaults.
    #[must_use]
    pub fn from_preset(mut self, preset: Identifier) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Record which pipeline transformed the generator's output.
    #[must_use]
    pub fn through_pipeline(
        mut self,
        pipeline: Identifier,
        version: ContentPipelineVersion,
    ) -> Self {
        self.pipeline = Some(pipeline);
        self.pipeline_version = Some(version);
        self
    }

    fn hash_into(&self, hasher: &mut Fnv1a64) {
        hasher.write_str(&self.generator.to_string());
        hasher.write_u64(u64::from(self.generator_version.0));
        hasher.write_str(
            &self
                .pipeline
                .as_ref()
                .map_or_else(String::new, ToString::to_string),
        );
        hasher.write_u64(u64::from(self.pipeline_version.map_or(0, |v| v.0)));
        hasher.write_str(
            &self
                .preset
                .as_ref()
                .map_or_else(String::new, ToString::to_string),
        );
        hasher.write_u64(self.seed);
        hasher.write_str(self.prompt.as_deref().unwrap_or(""));
        for (key, value) in &self.parameters {
            hasher.write_str(key);
            hasher.write_str(value);
        }
        for input in &self.inputs {
            hasher.write_str(&input.to_string());
        }
    }
}

/// The full origin record for one asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// Where it came from.
    pub class: ProvenanceClass,
    /// Whether it may be distributed.
    pub status: AssetStatus,
    /// Where it sits in review.
    pub release: ReleaseStatus,
    /// Who or what created it.
    pub author: String,
    /// The tool and version that produced it, e.g. `nexora-texture-forge@0.0.1`.
    ///
    /// Required even for procedural output. The provenance document asks for
    /// the tool *"when a tool contributes to an asset"*, and that is the field
    /// that distinguishes an AI-assisted asset from a procedural one — which is
    /// why AI generation is a value here rather than a class of its own.
    pub source_tool: String,
    /// Where the source material lives, when it came from somewhere.
    pub source_repository: Option<String>,
    /// The terms, when the asset is not NEXORA's own.
    pub license: Option<License>,
    /// Whether it has been changed since it was obtained.
    pub modification: Modification,
    /// Who reviewed it.
    pub reviewer: Option<String>,
    /// When it was reviewed.
    pub reviewed_at: Option<Timestamp>,
    /// Free-text notes for a reviewer.
    pub notes: Option<String>,
    /// How it was generated, when it was generated.
    pub generation: Option<GenerationTrace>,
    /// When the record was written. Excluded from [`Self::content_hash`].
    pub recorded_at: Timestamp,
}

impl Provenance {
    /// A record for content this forge generated procedurally.
    ///
    /// The common case, and the one with the least excuse for an incomplete
    /// record: everything it needs is already in hand at the moment of
    /// generation.
    #[must_use]
    pub fn generated(
        author: impl Into<String>,
        tool: impl Into<String>,
        trace: GenerationTrace,
    ) -> Self {
        Self {
            class: ProvenanceClass::ProceduralDerivative,
            status: AssetStatus::NexoraOriginal,
            release: ReleaseStatus::Draft,
            author: author.into(),
            source_tool: tool.into(),
            source_repository: None,
            license: None,
            modification: Modification::Unmodified,
            reviewer: None,
            reviewed_at: None,
            notes: None,
            generation: Some(trace),
            recorded_at: Timestamp::now(),
        }
    }

    /// A record for a definition a person authored, before anything ran.
    ///
    /// A material document is a recipe, not pixels: it has an author and a
    /// tool, and it has no generation trace because nothing has been generated
    /// yet. The trace arrives when the forge produces the maps, and the
    /// material is revised with the fuller record.
    #[must_use]
    pub fn authored(author: impl Into<String>, tool: impl Into<String>) -> Self {
        Self {
            class: ProvenanceClass::Original,
            status: AssetStatus::NexoraOriginal,
            release: ReleaseStatus::Draft,
            author: author.into(),
            source_tool: tool.into(),
            source_repository: None,
            license: None,
            modification: Modification::Unmodified,
            reviewer: None,
            reviewed_at: None,
            notes: None,
            generation: None,
            recorded_at: Timestamp::now(),
        }
    }

    /// Whether this asset is permitted in a release artifact.
    ///
    /// The gate from the policy document, as one function. `Cleared` alone is
    /// not enough: a blocked or rejected asset that someone marked cleared is a
    /// contradiction, and it resolves against shipping.
    #[must_use]
    pub const fn may_ship(&self) -> bool {
        matches!(self.release, ReleaseStatus::Cleared)
            && !matches!(self.status, AssetStatus::Blocked | AssetStatus::EditorOnly)
            && !matches!(
                self.class,
                ProvenanceClass::Rejected | ProvenanceClass::Experimental
            )
    }

    /// Check the record against the registry's own rules.
    ///
    /// # Errors
    ///
    /// Returns an error naming the rule that failed. The rules are the ones
    /// written in `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md`: no unknown
    /// origin, a license on third-party material, a named reviewer behind any
    /// clearance, and traceable modification.
    pub fn validate(&self) -> Result<()> {
        if self.author.trim().is_empty() {
            return Err(rule("an asset must name its author or creator"));
        }
        if self.source_tool.trim().is_empty() {
            return Err(rule("an asset must name the tool that produced it"));
        }

        let third_party = matches!(self.class, ProvenanceClass::LicensedThirdParty)
            || matches!(self.status, AssetStatus::ThirdPartyLicensed);
        if third_party && self.license.is_none() {
            return Err(rule("third-party material must record its license")
                .with_context("class", format!("{:?}", self.class)));
        }
        if let Some(license) = &self.license {
            if license.name.trim().is_empty() {
                return Err(rule("a recorded license must be named"));
            }
        }

        if matches!(self.release, ReleaseStatus::Cleared) {
            if self.reviewer.is_none() || self.reviewed_at.is_none() {
                return Err(rule("clearance requires a named reviewer and a date"));
            }
            if matches!(self.status, AssetStatus::Blocked)
                || matches!(self.class, ProvenanceClass::Rejected)
            {
                return Err(rule("a blocked or rejected asset cannot also be cleared"));
            }
        }

        if matches!(self.modification, Modification::Modified) && self.source_repository.is_none() {
            return Err(rule(
                "modified third-party material must name where it came from",
            ));
        }

        if matches!(self.class, ProvenanceClass::ProceduralDerivative) && self.generation.is_none()
        {
            return Err(rule(
                "procedural content must carry the trace that reproduces it",
            ));
        }

        Ok(())
    }

    /// A stable hash of the origin, excluding when the record was written.
    ///
    /// Two runs of the same definition, a year apart, produce the same hash and
    /// different timestamps — which is exactly what an audit wants to see.
    #[must_use]
    pub fn content_hash(&self) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str(&format!("{:?}", self.class));
        hasher.write_str(&format!("{:?}", self.status));
        hasher.write_str(&self.author);
        hasher.write_str(&self.source_tool);
        hasher.write_str(self.source_repository.as_deref().unwrap_or(""));
        hasher.write_str(&format!("{:?}", self.modification));
        if let Some(license) = &self.license {
            hasher.write_str(&license.name);
        }
        if let Some(trace) = &self.generation {
            trace.hash_into(&mut hasher);
        }
        hasher.finish()
    }
}

fn rule(message: &'static str) -> Error {
    Error::new(Domain::Content, "provenance", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn trace() -> GenerationTrace {
        GenerationTrace::new(
            id("nexora:generator/procedural"),
            ContentGeneratorVersion(1),
            0xABCD,
        )
        .with_parameter("style", "plank")
        .from_preset(id("nexora:preset/wood"))
    }

    fn generated() -> Provenance {
        Provenance::generated("NEXORA", "nexora-texture-forge@0.0.1", trace())
    }

    #[test]
    fn epoch_seconds_render_as_readable_utc() {
        assert_eq!(Timestamp::EPOCH.to_rfc3339_utc(), "1970-01-01T00:00:00Z");
        assert_eq!(
            Timestamp(1_000_000_000).to_rfc3339_utc(),
            "2001-09-09T01:46:40Z"
        );
        // A leap day, which is where a naive conversion goes wrong.
        assert_eq!(
            Timestamp(1_709_164_800).to_rfc3339_utc(),
            "2024-02-29T00:00:00Z"
        );
        // Before the epoch, which is where a truncating division goes wrong.
        assert_eq!(Timestamp(-1).to_rfc3339_utc(), "1969-12-31T23:59:59Z");
        assert_eq!(Timestamp(0).to_string(), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn every_timestamp_survives_being_written_and_read_back() {
        // Round trip rather than a table: this proves the two conversions agree
        // with each other, which is the property the document format needs.
        let mut day = -25_000i64; // 1901
        while day < 25_000 {
            // 2038
            let stamp = Timestamp(day * 86_400 + 45_296); // 12:34:56
            let text = stamp.to_rfc3339_utc();
            assert_eq!(
                Timestamp::parse_rfc3339_utc(&text).unwrap(),
                stamp,
                "failed on {text}"
            );
            day += 97; // a stride that is coprime with 4, 100 and 400
        }
        assert_eq!(
            Timestamp::parse_rfc3339_utc("1970-01-01T00:00:00Z").unwrap(),
            Timestamp::EPOCH
        );
    }

    #[test]
    fn a_malformed_timestamp_is_refused() {
        for bad in [
            "",
            "1970-01-01",
            "1970-01-01T00:00:00",
            "1970-01-01T00:00:00+01:00",
            "1970-13-01T00:00:00Z",
            "1970-01-32T00:00:00Z",
            "1970-01-01T24:00:00Z",
            "1970-01-01T00:60:00Z",
            "1970-01-01T00:00:60Z",
            "197x-01-01T00:00:00Z",
        ] {
            assert!(
                Timestamp::parse_rfc3339_utc(bad).is_err(),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn every_vocabulary_name_round_trips_and_is_distinct() {
        for class in ProvenanceClass::ALL {
            assert_eq!(ProvenanceClass::parse(class.as_str()).unwrap(), class);
        }
        for status in AssetStatus::ALL {
            assert_eq!(AssetStatus::parse(status.as_str()).unwrap(), status);
        }
        for state in ReleaseStatus::ALL {
            assert_eq!(ReleaseStatus::parse(state.as_str()).unwrap(), state);
        }

        // An unknown class does not fall back to anything: unknown origin blocks.
        assert!(ProvenanceClass::parse("ai_generated").is_err());
        assert!(AssetStatus::parse("fine_probably").is_err());
        assert!(ReleaseStatus::parse("shipped").is_err());

        let mut names: Vec<&str> = ProvenanceClass::ALL
            .iter()
            .map(|class| class.as_str())
            .collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn the_host_clock_is_read_only_here_and_lands_in_this_century() {
        // 2020-01-01. A machine reporting earlier is misconfigured, not a bug.
        assert!(Timestamp::now().0 > 1_577_836_800);
    }

    #[test]
    fn a_generated_record_satisfies_the_registry_rules() {
        let record = generated();
        record.validate().expect("a generated record must be valid");
        assert_eq!(record.class, ProvenanceClass::ProceduralDerivative);
        assert_eq!(record.status, AssetStatus::NexoraOriginal);
        // Draft, not cleared: generating something does not clear it.
        assert_eq!(record.release, ReleaseStatus::Draft);
        assert!(!record.may_ship());
    }

    #[test]
    fn an_authored_definition_is_valid_before_anything_has_been_generated() {
        let record = Provenance::authored("operator", "hand-written");
        record.validate().expect("a recipe needs no trace");
        assert_eq!(record.class, ProvenanceClass::Original);
        assert!(record.generation.is_none());
        // A recipe is not a cleared asset either.
        assert!(!record.may_ship());
    }

    #[test]
    fn procedural_content_without_a_trace_is_refused() {
        let mut record = generated();
        record.generation = None;
        let err = record
            .validate()
            .expect_err("a trace is what reproduces it");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("reproduces"), "{err}");
    }

    #[test]
    fn third_party_material_without_a_license_is_refused() {
        let mut record = generated();
        record.class = ProvenanceClass::LicensedThirdParty;
        let err = record.validate().expect_err("a license is mandatory here");
        assert!(err.to_string().contains("license"), "{err}");

        record.license = Some(License {
            name: "CC0-1.0".to_owned(),
            url: None,
            attribution_required: false,
            redistribution_allowed: true,
            commercial_use_allowed: true,
        });
        record
            .validate()
            .expect("a named license satisfies the rule");

        // An unnamed license is not a license.
        record.license.as_mut().unwrap().name = "  ".to_owned();
        assert!(record.validate().is_err());
    }

    #[test]
    fn clearance_needs_a_named_reviewer_and_a_date() {
        let mut record = generated();
        record.release = ReleaseStatus::Cleared;
        let err = record.validate().expect_err("clearance needs a reviewer");
        assert!(err.to_string().contains("reviewer"), "{err}");

        record.reviewer = Some("operator".to_owned());
        record.reviewed_at = Some(Timestamp(1_700_000_000));
        record.validate().expect("a reviewed record clears");
        assert!(record.may_ship());
    }

    #[test]
    fn a_blocked_asset_cannot_be_cleared_by_saying_so() {
        let mut record = generated();
        record.release = ReleaseStatus::Cleared;
        record.reviewer = Some("operator".to_owned());
        record.reviewed_at = Some(Timestamp(1_700_000_000));
        record.status = AssetStatus::Blocked;

        assert!(!record.may_ship());
        let err = record
            .validate()
            .expect_err("the contradiction must surface");
        assert!(err.to_string().contains("cannot also be cleared"), "{err}");
    }

    #[test]
    fn editor_only_and_experimental_assets_never_ship() {
        for (status, class) in [
            (AssetStatus::EditorOnly, ProvenanceClass::Original),
            (AssetStatus::NexoraOriginal, ProvenanceClass::Experimental),
            (AssetStatus::NexoraOriginal, ProvenanceClass::Rejected),
        ] {
            let mut record = generated();
            record.release = ReleaseStatus::Cleared;
            record.status = status;
            record.class = class;
            assert!(!record.may_ship(), "{status:?} / {class:?} must not ship");
        }
    }

    #[test]
    fn an_asset_must_name_its_author_and_its_tool() {
        let mut record = generated();
        record.author = "   ".to_owned();
        assert!(record.validate().is_err());

        let mut record = generated();
        record.source_tool = String::new();
        let err = record.validate().expect_err("the tool must be recorded");
        assert!(err.to_string().contains("tool"), "{err}");
    }

    #[test]
    fn modified_third_party_material_must_say_where_it_came_from() {
        let mut record = generated();
        record.modification = Modification::Modified;
        assert!(record.validate().is_err());

        record.source_repository = Some("https://example.invalid/pack".to_owned());
        record
            .validate()
            .expect("a named source satisfies the rule");
    }

    #[test]
    fn the_content_hash_ignores_when_but_not_what() {
        let early = generated();
        let mut later = early.clone();
        later.recorded_at = Timestamp(early.recorded_at.0 + 31_536_000);
        assert_eq!(
            early.content_hash(),
            later.content_hash(),
            "a year of waiting must not change the asset"
        );

        // Every input that changes the pixels changes the hash.
        let mut different_seed = early.clone();
        different_seed.generation.as_mut().unwrap().seed = 1;
        assert_ne!(early.content_hash(), different_seed.content_hash());

        let mut different_parameter = early.clone();
        different_parameter
            .generation
            .as_mut()
            .unwrap()
            .parameters
            .insert("style".to_owned(), "bark".to_owned());
        assert_ne!(early.content_hash(), different_parameter.content_hash());

        let mut different_version = early.clone();
        different_version
            .generation
            .as_mut()
            .unwrap()
            .generator_version = ContentGeneratorVersion(2);
        assert_ne!(early.content_hash(), different_version.content_hash());
    }

    #[test]
    fn a_prompt_is_recorded_so_ai_output_cannot_pass_as_procedural() {
        let mut record = generated();
        record.generation.as_mut().unwrap().prompt = Some("weathered oak planks".to_owned());
        record.source_tool = "some-image-model@1".to_owned();

        assert_ne!(record.content_hash(), generated().content_hash());
        assert_eq!(
            record.generation.as_ref().unwrap().prompt.as_deref(),
            Some("weathered oak planks")
        );
    }

    #[test]
    fn a_pipeline_is_recorded_alongside_the_generator() {
        let record = Provenance::generated(
            "NEXORA",
            "nexora-texture-forge@0.0.1",
            trace().through_pipeline(id("nexora:pipeline/pbr"), ContentPipelineVersion(1)),
        );
        record.validate().expect("valid");
        let generation = record.generation.as_ref().unwrap();
        assert_eq!(generation.pipeline_version, Some(ContentPipelineVersion(1)));
        assert_ne!(record.content_hash(), generated().content_hash());
    }
}
