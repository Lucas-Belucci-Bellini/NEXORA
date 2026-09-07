//! Where residency actually happens.
//!
//! `STREAMING SYSTEM.md` opens by saying what streaming is not:
//!
//! > Streaming controls which world, entity, resource and simulation data is
//! > resident. **It does not own generation or gameplay rules.**
//!
//! [`ResidencyBackend`] is that sentence expressed as a type. The manager
//! decides *what* should be resident and *when*; the backend knows how to make
//! it so. `nexora-streaming` therefore does not depend on `nexora-world` and
//! cannot generate a chunk even by accident — the same boundary
//! `nexora-physics` has for the same reason.
//!
//! ## The contract that matters most
//!
//! **Eviction must not lose state.** The manager calls [`ResidencyBackend::persist`]
//! before dropping anything the backend reports as dirty, and
//! `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md` §18 requires the target's
//! logical identity and persistent state to survive. A backend whose `evict`
//! discards an unsaved edit is broken, and the test for it is not optional.

use std::collections::BTreeMap;

use nexora_foundation::error::Result;

use crate::lod::Lod;
use crate::target::StreamTarget;

/// Something that can make targets resident and drop them again.
pub trait ResidencyBackend {
    /// Bring `target` to `lod`, returning the bytes it now occupies.
    ///
    /// Called for every promotion, including tiers the backend does not
    /// materialise — those should return `Ok(0)` and do nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when the target cannot be brought to that tier. The
    /// manager counts the failure and keeps the target at the tier it had.
    fn activate(&mut self, target: StreamTarget, lod: Lod) -> Result<u64>;

    /// Drop `target`'s resident data.
    ///
    /// Persistent state must survive; see the module documentation.
    ///
    /// # Errors
    ///
    /// Returns an error when the target cannot be released.
    fn evict(&mut self, target: StreamTarget) -> Result<()>;

    /// Write `target`'s state out. Called before eviction when it is dirty.
    ///
    /// # Errors
    ///
    /// Returns an error when the state cannot be written. The manager then
    /// **does not evict**: losing an edit is worse than holding memory.
    fn persist(&mut self, target: StreamTarget) -> Result<()>;

    /// Whether `target` holds changes that eviction would lose.
    ///
    /// Defaults to `true`, deliberately. A backend that has not thought about
    /// this gets the safe answer — persist before evicting — rather than the
    /// fast one.
    fn is_dirty(&self, target: StreamTarget) -> bool {
        let _ = target;
        true
    }
}

/// A complete in-memory backend, used for tests and tools.
///
/// A **fixture, not a mock**: it implements the whole trait correctly for the
/// world it describes — targets are resident or not, dirtiness is tracked, and
/// persisting really does move state somewhere eviction cannot touch. Nothing
/// about it is a placeholder for an unwritten implementation.
#[derive(Debug, Default, Clone)]
pub struct MemoryBackend {
    resident: BTreeMap<StreamTarget, u64>,
    persisted: BTreeMap<StreamTarget, u64>,
    dirty: BTreeMap<StreamTarget, bool>,
    bytes_per_target: u64,
    /// Targets whose activation should fail, for exercising the failure path.
    refuse: BTreeMap<StreamTarget, ()>,
    activations: u32,
    evictions: u32,
    persists: u32,
}

impl MemoryBackend {
    /// A backend where every target costs `bytes_per_target`.
    #[must_use]
    pub fn new(bytes_per_target: u64) -> Self {
        Self {
            bytes_per_target,
            ..Self::default()
        }
    }

    /// Make `target` refuse activation, to exercise the failure path.
    pub fn refuse(&mut self, target: StreamTarget) {
        self.refuse.insert(target, ());
    }

    /// Mark a resident target as holding unsaved changes.
    pub fn touch(&mut self, target: StreamTarget) {
        self.dirty.insert(target, true);
    }

    /// Whether the backend currently holds `target`.
    #[must_use]
    pub fn is_resident(&self, target: StreamTarget) -> bool {
        self.resident.contains_key(&target)
    }

    /// Whether `target`'s state was written out at some point.
    #[must_use]
    pub fn was_persisted(&self, target: StreamTarget) -> bool {
        self.persisted.contains_key(&target)
    }

    /// How many targets are resident.
    #[must_use]
    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }

    /// Counts of the three operations, for assertions.
    #[must_use]
    pub const fn counts(&self) -> (u32, u32, u32) {
        (self.activations, self.evictions, self.persists)
    }
}

impl ResidencyBackend for MemoryBackend {
    fn activate(&mut self, target: StreamTarget, lod: Lod) -> Result<u64> {
        if self.refuse.contains_key(&target) {
            return Err(nexora_foundation::error::Error::new(
                nexora_foundation::error::Domain::World,
                "memory-backend",
                "this target is configured to refuse activation",
            )
            .with_context("target", target.to_string()));
        }
        if !lod.is_resident() {
            return Ok(0);
        }
        self.activations += 1;
        self.resident.insert(target, self.bytes_per_target);
        self.dirty.entry(target).or_insert(false);
        Ok(self.bytes_per_target)
    }

    fn evict(&mut self, target: StreamTarget) -> Result<()> {
        if self.resident.remove(&target).is_some() {
            self.evictions += 1;
        }
        Ok(())
    }

    fn persist(&mut self, target: StreamTarget) -> Result<()> {
        self.persists += 1;
        let bytes = self.resident.get(&target).copied().unwrap_or_default();
        self.persisted.insert(target, bytes);
        self.dirty.insert(target, false);
        Ok(())
    }

    fn is_dirty(&self, target: StreamTarget) -> bool {
        self.dirty.get(&target).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::spatial::ChunkCoord;

    fn chunk(x: i64, z: i64) -> StreamTarget {
        StreamTarget::Chunk(ChunkCoord::new(x, z))
    }

    #[test]
    fn activation_makes_a_target_resident_and_eviction_releases_it() {
        let mut backend = MemoryBackend::new(1_024);
        let target = chunk(0, 0);
        assert_eq!(backend.activate(target, Lod::Full).expect("ok"), 1_024);
        assert!(backend.is_resident(target));
        backend.evict(target).expect("ok");
        assert!(!backend.is_resident(target));
        assert_eq!(backend.counts(), (1, 1, 0));
    }

    #[test]
    fn a_non_resident_tier_costs_nothing_and_holds_nothing() {
        let mut backend = MemoryBackend::new(1_024);
        let target = chunk(1, 1);
        for tier in [Lod::Regional, Lod::Abstract, Lod::Unresident] {
            assert_eq!(backend.activate(target, tier).expect("ok"), 0);
            assert!(!backend.is_resident(target), "{}", tier.as_str());
        }
    }

    #[test]
    fn a_freshly_activated_target_is_not_dirty() {
        let mut backend = MemoryBackend::new(8);
        let target = chunk(2, 2);
        backend.activate(target, Lod::Full).expect("ok");
        assert!(!backend.is_dirty(target));
        backend.touch(target);
        assert!(backend.is_dirty(target));
    }

    #[test]
    fn persisting_clears_dirtiness_and_records_the_state() {
        let mut backend = MemoryBackend::new(8);
        let target = chunk(3, 3);
        backend.activate(target, Lod::Full).expect("ok");
        backend.touch(target);
        backend.persist(target).expect("ok");
        assert!(!backend.is_dirty(target));
        assert!(backend.was_persisted(target));
    }

    #[test]
    fn a_refusing_target_reports_the_failure_rather_than_pretending() {
        let mut backend = MemoryBackend::new(8);
        let target = chunk(4, 4);
        backend.refuse(target);
        assert!(backend.activate(target, Lod::Full).is_err());
        assert!(!backend.is_resident(target));
    }

    #[test]
    fn the_default_dirtiness_answer_is_the_safe_one() {
        // A backend that does not override `is_dirty` must get "yes, persist
        // it" rather than "no, just drop it".
        struct Minimal;
        impl ResidencyBackend for Minimal {
            fn activate(&mut self, _t: StreamTarget, _l: Lod) -> Result<u64> {
                Ok(0)
            }
            fn evict(&mut self, _t: StreamTarget) -> Result<()> {
                Ok(())
            }
            fn persist(&mut self, _t: StreamTarget) -> Result<()> {
                Ok(())
            }
        }
        assert!(Minimal.is_dirty(chunk(0, 0)));
    }
}
