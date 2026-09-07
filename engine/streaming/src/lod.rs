//! Level-of-detail tiers.
//!
//! `STREAMING SYSTEM.md` defines the ladder `FULL → REGIONAL → ABSTRACT →
//! UNRESIDENT` and one invariant that outranks all of it:
//!
//! > Logical identity and persistent state survive eviction.
//!
//! A tier is therefore about *how much of a thing is loaded*, never about
//! whether the thing still exists. `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md`
//! §18 lists what each transition has to preserve — settlement identity,
//! economic state, active historical events — and none of that is allowed to
//! depend on the player standing nearby.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// How much of a target is loaded.
///
/// Ordered from most to least detailed, so `Full > Regional > Abstract >
/// Unresident` compares the way the ladder reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Lod {
    /// Not loaded. Persistent state still exists; it is simply not here.
    #[default]
    Unresident,
    /// A statistical or event-driven summary.
    Abstract,
    /// Reduced detail: the region simulates, but not every cell.
    Regional,
    /// Everything resident and simulating.
    Full,
}

impl Lod {
    /// The ladder, most detailed first.
    pub const LADDER: [Self; 4] = [Self::Full, Self::Regional, Self::Abstract, Self::Unresident];

    /// Stable lowercase name, safe to emit in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Regional => "regional",
            Self::Abstract => "abstract",
            Self::Unresident => "unresident",
        }
    }

    /// Whether data has to be held in memory at this tier.
    ///
    /// Only [`Lod::Full`] materialises voxel data in this phase. The tiers
    /// between it and [`Lod::Unresident`] are real states of the manager — a
    /// target at `Regional` is tracked, keeps its identity, and is not resident
    /// — but no backend holds distinct data for them yet, because the regional
    /// simulation that would fill them does not exist. That gap is recorded as
    /// debt rather than papered over with a tier that silently means nothing.
    #[must_use]
    pub const fn is_resident(self) -> bool {
        matches!(self, Self::Full)
    }

    /// Whether the target is tracked at all.
    #[must_use]
    pub const fn is_tracked(self) -> bool {
        !matches!(self, Self::Unresident)
    }

    /// One step less detailed.
    #[must_use]
    pub const fn demoted(self) -> Self {
        match self {
            Self::Full => Self::Regional,
            Self::Regional => Self::Abstract,
            Self::Abstract | Self::Unresident => Self::Unresident,
        }
    }

    /// One step more detailed.
    #[must_use]
    pub const fn promoted(self) -> Self {
        match self {
            Self::Unresident => Self::Abstract,
            Self::Abstract => Self::Regional,
            Self::Regional | Self::Full => Self::Full,
        }
    }

    /// Parse a tier from its stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not one of the four tiers. Refused
    /// rather than defaulted: silently reading an unknown tier as `Unresident`
    /// would evict data because of a typo.
    pub fn parse(name: &str) -> Result<Self> {
        Self::LADDER
            .into_iter()
            .find(|tier| tier.as_str() == name)
            .ok_or_else(|| {
                Error::new(
                    Domain::World,
                    "streaming-lod",
                    "unknown level-of-detail tier",
                )
                .with_recovery(Recovery::Reject)
                .with_context("name", name.to_owned())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ladder_orders_from_most_to_least_detailed() {
        assert!(Lod::Full > Lod::Regional);
        assert!(Lod::Regional > Lod::Abstract);
        assert!(Lod::Abstract > Lod::Unresident);
        assert_eq!(Lod::LADDER[0], Lod::Full);
        assert_eq!(Lod::LADDER[3], Lod::Unresident);
    }

    #[test]
    fn the_default_tier_is_unresident_so_nothing_loads_by_accident() {
        assert_eq!(Lod::default(), Lod::Unresident);
        assert!(!Lod::default().is_resident());
        assert!(!Lod::default().is_tracked());
    }

    #[test]
    fn stepping_up_and_down_saturates_at_the_ends() {
        assert_eq!(Lod::Full.promoted(), Lod::Full);
        assert_eq!(Lod::Unresident.demoted(), Lod::Unresident);
        assert_eq!(Lod::Full.demoted(), Lod::Regional);
        assert_eq!(Lod::Unresident.promoted(), Lod::Abstract);
    }

    #[test]
    fn a_full_walk_down_the_ladder_reaches_every_tier() {
        let mut tier = Lod::Full;
        let mut seen = vec![tier];
        for _ in 0..3 {
            tier = tier.demoted();
            seen.push(tier);
        }
        assert_eq!(seen, Lod::LADDER.to_vec());
    }

    #[test]
    fn only_full_materialises_data_in_this_phase() {
        assert!(Lod::Full.is_resident());
        for tier in [Lod::Regional, Lod::Abstract, Lod::Unresident] {
            assert!(!tier.is_resident(), "{}", tier.as_str());
        }
        // But the tiers between still count as tracked: identity survives.
        assert!(Lod::Regional.is_tracked());
        assert!(Lod::Abstract.is_tracked());
    }

    #[test]
    fn names_round_trip_and_an_unknown_name_is_refused() {
        for tier in Lod::LADDER {
            assert_eq!(Lod::parse(tier.as_str()).expect("known"), tier);
        }
        assert!(Lod::parse("full ").is_err());
        assert!(Lod::parse("").is_err());
    }
}
