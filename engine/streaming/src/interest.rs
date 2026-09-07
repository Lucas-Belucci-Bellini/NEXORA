//! Interest sources and the radii that turn distance into a tier.
//!
//! `STREAMING SYSTEM.md` lists what generates interest — player, camera,
//! vehicles, AI, server visibility, prefetch and world-event relevance — and
//! requires **hysteresis** so that residency does not thrash.
//!
//! ## Why hysteresis is not optional
//!
//! Without it, an observer standing exactly on a radius boundary loads a chunk
//! on one tick and evicts it on the next, forever. Every one of those cycles
//! costs a generation or a disk read. The margin makes the boundary a band: a
//! target is promoted at the radius and demoted only past `radius + margin`, so
//! crossing back and forth inside the band changes nothing.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::ChunkCoord;

use crate::lod::Lod;

/// The largest radius an interest source may declare, in chunks.
///
/// A radius is reachable from configuration and from mods, and the area it
/// describes grows as its **square** — which matters more than it looks,
/// because every tick enumerates that area to decide tiers. At 127 the outer
/// ring is 255×255, or 65 025 columns: already far beyond anything a renderer
/// will draw, and still an enumeration a tick can afford. Bounded here so that
/// a typo is a rejected configuration rather than a frozen tick.
pub const MAX_RADIUS: u32 = 127;

/// Default chunks of overshoot before a target is demoted.
pub const DEFAULT_HYSTERESIS: u32 = 2;

/// Stable identity of something that generates interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterestId(pub u64);

/// How far each tier extends from an observer, in chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodRadii {
    full: u32,
    regional: u32,
    abstracted: u32,
    hysteresis: u32,
}

impl LodRadii {
    /// Build radii.
    ///
    /// # Errors
    ///
    /// Returns an error when the radii are not ascending, or when any exceeds
    /// [`MAX_RADIUS`]. Non-ascending radii would make a nearer tier smaller
    /// than a further one, which has no meaning and would evict what the
    /// observer is standing on.
    pub fn new(full: u32, regional: u32, abstracted: u32, hysteresis: u32) -> Result<Self> {
        for (name, value) in [
            ("full", full),
            ("regional", regional),
            ("abstract", abstracted),
            ("hysteresis", hysteresis),
        ] {
            if value > MAX_RADIUS {
                return Err(reject("radius exceeds the permitted maximum")
                    .with_context("field", name)
                    .with_context("value", value.to_string())
                    .with_context("max", MAX_RADIUS.to_string()));
            }
        }
        if full > regional || regional > abstracted {
            return Err(reject("radii must not shrink as detail decreases")
                .with_context("full", full.to_string())
                .with_context("regional", regional.to_string())
                .with_context("abstract", abstracted.to_string()));
        }
        Ok(Self {
            full,
            regional,
            abstracted,
            hysteresis,
        })
    }

    /// A simple ladder derived from one full-detail radius.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived outer radii exceed [`MAX_RADIUS`].
    pub fn around(full: u32) -> Result<Self> {
        Self::new(
            full,
            full.saturating_mul(2),
            full.saturating_mul(3),
            DEFAULT_HYSTERESIS,
        )
    }

    /// Radius of the full-detail tier.
    #[must_use]
    pub const fn full(self) -> u32 {
        self.full
    }

    /// Radius of the regional tier.
    #[must_use]
    pub const fn regional(self) -> u32 {
        self.regional
    }

    /// Radius of the abstract tier.
    #[must_use]
    pub const fn abstracted(self) -> u32 {
        self.abstracted
    }

    /// Chunks of overshoot before a target is demoted.
    #[must_use]
    pub const fn hysteresis(self) -> u32 {
        self.hysteresis
    }

    /// The outermost radius anything is tracked at.
    #[must_use]
    pub const fn outer(self) -> u32 {
        self.abstracted
    }

    /// The radius of one tier.
    #[must_use]
    pub const fn radius_of(self, tier: Lod) -> u32 {
        match tier {
            Lod::Full => self.full,
            Lod::Regional => self.regional,
            Lod::Abstract => self.abstracted,
            Lod::Unresident => 0,
        }
    }

    /// The tier a distance falls in, ignoring hysteresis.
    ///
    /// This is the **promotion** answer: what a target would be if it were
    /// arriving fresh.
    #[must_use]
    pub fn tier_at(self, distance: u64) -> Lod {
        if distance <= u64::from(self.full) {
            Lod::Full
        } else if distance <= u64::from(self.regional) {
            Lod::Regional
        } else if distance <= u64::from(self.abstracted) {
            Lod::Abstract
        } else {
            Lod::Unresident
        }
    }

    /// The tier a target already at `current` should hold at `distance`.
    ///
    /// Promotion uses the plain radius; demotion needs the target to have left
    /// the band `radius ..= radius + hysteresis`. A target inside the band
    /// keeps the tier it has, which is what stops the thrash.
    #[must_use]
    pub fn tier_for(self, current: Lod, distance: u64) -> Lod {
        let fresh = self.tier_at(distance);
        if fresh >= current {
            return fresh;
        }
        // Demotion is proposed. Only allow it once the target is past the far
        // edge of its current tier's band.
        let band = u64::from(self.radius_of(current)) + u64::from(self.hysteresis);
        if distance > band {
            fresh
        } else {
            current
        }
    }
}

impl Default for LodRadii {
    fn default() -> Self {
        Self {
            full: 2,
            regional: 4,
            abstracted: 6,
            hysteresis: DEFAULT_HYSTERESIS,
        }
    }
}

/// Something whose position drives residency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterestSource {
    /// Stable identity, so moving a source updates it rather than adding one.
    pub id: InterestId,
    /// The column the source is in.
    pub center: ChunkCoord,
    /// How far each tier reaches.
    pub radii: LodRadii,
}

impl InterestSource {
    /// An interest source with the default ladder.
    #[must_use]
    pub fn new(id: InterestId, center: ChunkCoord) -> Self {
        Self {
            id,
            center,
            radii: LodRadii::default(),
        }
    }

    /// Move the source.
    #[must_use]
    pub const fn moved_to(mut self, center: ChunkCoord) -> Self {
        self.center = center;
        self
    }

    /// Replace the radii.
    #[must_use]
    pub const fn with_radii(mut self, radii: LodRadii) -> Self {
        self.radii = radii;
        self
    }
}

fn reject(message: &'static str) -> Error {
    Error::new(Domain::World, "streaming-interest", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radii_must_not_shrink_as_detail_decreases() {
        assert!(LodRadii::new(4, 2, 6, 1).is_err());
        assert!(LodRadii::new(2, 6, 4, 1).is_err());
        assert!(LodRadii::new(2, 4, 6, 1).is_ok());
        // Equal radii are legal: a world may skip a tier.
        assert!(LodRadii::new(3, 3, 3, 1).is_ok());
    }

    #[test]
    fn an_absurd_radius_is_refused_rather_than_generating_forever() {
        assert!(LodRadii::new(MAX_RADIUS + 1, MAX_RADIUS + 1, MAX_RADIUS + 1, 0).is_err());
        assert!(LodRadii::around(MAX_RADIUS).is_err());
        assert!(LodRadii::around(4).is_ok());
    }

    #[test]
    fn distance_maps_onto_the_ladder() {
        let radii = LodRadii::new(2, 4, 6, 0).expect("valid");
        assert_eq!(radii.tier_at(0), Lod::Full);
        assert_eq!(radii.tier_at(2), Lod::Full);
        assert_eq!(radii.tier_at(3), Lod::Regional);
        assert_eq!(radii.tier_at(4), Lod::Regional);
        assert_eq!(radii.tier_at(5), Lod::Abstract);
        assert_eq!(radii.tier_at(6), Lod::Abstract);
        assert_eq!(radii.tier_at(7), Lod::Unresident);
        assert_eq!(radii.tier_at(u64::MAX), Lod::Unresident);
    }

    #[test]
    fn promotion_ignores_hysteresis_but_demotion_does_not() {
        let radii = LodRadii::new(2, 4, 6, 2).expect("valid");
        // Arriving from outside: the plain radius decides.
        assert_eq!(radii.tier_for(Lod::Unresident, 2), Lod::Full);
        // Leaving: still Full inside the band `full ..= full + hysteresis`,
        // which is 2..=4 here.
        assert_eq!(radii.tier_for(Lod::Full, 3), Lod::Full);
        assert_eq!(radii.tier_for(Lod::Full, 4), Lod::Full);
        // Past the band it lands on the tier the plain radii give, which at
        // distance 5 is Abstract - Regional only reaches 4.
        assert_eq!(radii.tier_for(Lod::Full, 5), Lod::Abstract);
    }

    #[test]
    fn demotion_lands_where_the_target_belongs_rather_than_stepping_one_tier() {
        // `STREAMING SYSTEM.md` lists fast travel as a test case. Stepping down
        // one tier per tick would hold a region resident for three more ticks
        // after the observer has gone somewhere else entirely, which is exactly
        // the memory the budget is trying to protect.
        let radii = LodRadii::new(2, 4, 6, 2).expect("valid");
        assert_eq!(radii.tier_for(Lod::Full, 500), Lod::Unresident);
        assert_eq!(radii.tier_for(Lod::Full, 6), Lod::Abstract);
        // And the hysteresis band still protects the tier it is leaving.
        assert_eq!(radii.tier_for(Lod::Regional, 5), Lod::Regional);
        assert_eq!(radii.tier_for(Lod::Regional, 7), Lod::Unresident);
    }

    #[test]
    fn walking_back_and_forth_across_a_boundary_changes_nothing() {
        // The whole reason hysteresis exists. Without it this loop would load
        // and evict on alternate ticks forever.
        let radii = LodRadii::new(2, 4, 6, 2).expect("valid");
        let mut tier = radii.tier_for(Lod::Unresident, 2);
        assert_eq!(tier, Lod::Full);
        for distance in [3, 2, 3, 2, 3, 2, 4, 3] {
            let next = radii.tier_for(tier, distance);
            assert_eq!(next, Lod::Full, "thrashed at distance {distance}");
            tier = next;
        }
    }

    #[test]
    fn zero_hysteresis_still_behaves_correctly_just_without_the_band() {
        let radii = LodRadii::new(2, 4, 6, 0).expect("valid");
        assert_eq!(radii.tier_for(Lod::Full, 3), Lod::Regional);
        assert_eq!(radii.tier_for(Lod::Regional, 3), Lod::Regional);
    }

    #[test]
    fn a_target_far_outside_every_band_is_demoted_immediately() {
        let radii = LodRadii::new(2, 4, 6, 2).expect("valid");
        assert_eq!(radii.tier_for(Lod::Full, 1_000), Lod::Unresident);
    }

    #[test]
    fn the_derived_ladder_is_ascending() {
        let radii = LodRadii::around(4).expect("valid");
        assert_eq!(radii.full(), 4);
        assert_eq!(radii.regional(), 8);
        assert_eq!(radii.abstracted(), 12);
        assert_eq!(radii.outer(), 12);
        assert_eq!(radii.hysteresis(), DEFAULT_HYSTERESIS);
    }

    #[test]
    fn a_source_can_be_moved_without_losing_its_identity() {
        let source = InterestSource::new(InterestId(7), ChunkCoord::new(0, 0));
        let moved = source.moved_to(ChunkCoord::new(9, 9));
        assert_eq!(moved.id, source.id);
        assert_eq!(moved.center, ChunkCoord::new(9, 9));
        assert_eq!(moved.radii, source.radii);
    }

    #[test]
    fn the_unresident_tier_has_no_radius() {
        let radii = LodRadii::default();
        assert_eq!(radii.radius_of(Lod::Unresident), 0);
        assert_eq!(radii.radius_of(Lod::Full), radii.full());
    }
}
