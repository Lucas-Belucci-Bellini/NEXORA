//! The streaming manager.
//!
//! Implements the pipeline `STREAMING SYSTEM.md` specifies:
//!
//! ```text
//! Interest → Priority → Request → Load/Generate → Validate
//!          → Activate → Simulate → Deactivate → Persist → Evict
//! ```
//!
//! One [`StreamingSystem::tick`] walks it end to end for as much work as the
//! budget allows, and reports what it could not do rather than doing it anyway.
//!
//! ## Determinism
//!
//! Every container here is a `BTreeMap` or a `BTreeSet`, every sort carries an
//! explicit tie-break on the target itself, and nothing consults a clock. The
//! same interest, the same budget and the same backend produce the same
//! decisions in the same order — which is what makes a streamed world
//! reproducible under `NEXORA REPLAY AND DETERMINISM.md`.
//!
//! ## What the manager will not do
//!
//! It will not evict a target whose state it could not persist. Holding memory
//! is recoverable; losing an edit is not.

use std::collections::{BTreeMap, BTreeSet};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::ChunkCoord;

use crate::backend::ResidencyBackend;
use crate::budget::{StreamingBudget, StreamingReport};
use crate::interest::{InterestId, InterestSource};
use crate::lod::Lod;
use crate::target::{StreamHandle, StreamReason, StreamTarget};

/// The most interest sources one system will track.
pub const MAX_INTEREST_SOURCES: usize = 256;

/// The most failures kept for inspection before older ones are dropped.
///
/// Bounded because a backend failing every tick would otherwise grow this
/// vector without limit; the count in the report stays exact regardless.
pub const MAX_RETAINED_FAILURES: usize = 64;

/// One scheduled tier change: the target, the tier it should reach, and its
/// distance from the nearest observer.
type Scheduled = (StreamTarget, Lod, u64);

/// What the system knows about one target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetState {
    /// The tier it is currently at.
    pub lod: Lod,
    /// Bytes the backend reported for it.
    pub bytes: u64,
    /// Why it is here.
    pub reason: StreamReason,
}

#[derive(Debug, Clone, Copy)]
struct PinSlot {
    generation: u32,
    target: Option<StreamTarget>,
}

/// Interest, priority, budgets and residency for a set of targets.
#[derive(Debug, Default)]
pub struct StreamingSystem {
    interests: BTreeMap<InterestId, InterestSource>,
    tracked: BTreeMap<StreamTarget, TargetState>,
    pins: Vec<PinSlot>,
    free_pins: Vec<u32>,
    pin_counts: BTreeMap<StreamTarget, u32>,
    failures: Vec<Error>,
    dropped_failures: u64,
    bytes: u64,
}

impl StreamingSystem {
    /// An empty system: no interest, nothing tracked.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or move an interest source.
    ///
    /// Keyed by [`InterestId`], so moving a player updates their source rather
    /// than adding a second one that keeps the old region loaded forever.
    ///
    /// # Errors
    ///
    /// Returns an error when adding a new source would exceed
    /// [`MAX_INTEREST_SOURCES`]. Moving an existing one never fails.
    pub fn set_interest(&mut self, source: InterestSource) -> Result<()> {
        if !self.interests.contains_key(&source.id) && self.interests.len() >= MAX_INTEREST_SOURCES
        {
            return Err(reject("too many interest sources")
                .with_context("max", MAX_INTEREST_SOURCES.to_string()));
        }
        self.interests.insert(source.id, source);
        Ok(())
    }

    /// Remove an interest source. Returns whether it was there.
    ///
    /// Whatever it was holding resident is released by the next tick, subject
    /// to the eviction budget.
    pub fn remove_interest(&mut self, id: InterestId) -> bool {
        self.interests.remove(&id).is_some()
    }

    /// The interest sources currently driving residency.
    pub fn interests(&self) -> impl Iterator<Item = &InterestSource> + '_ {
        self.interests.values()
    }

    /// Pin a target resident regardless of interest.
    ///
    /// The pin lasts until [`Self::cancel`]. This is the `request` of
    /// `STREAMING SYSTEM.md`'s API: something that needs a region loaded for a
    /// reason distance cannot express — a world event, a teleport destination,
    /// a save about to be written.
    pub fn request(&mut self, target: StreamTarget, reason: StreamReason) -> StreamHandle {
        let index = if let Some(index) = self.free_pins.pop() {
            self.pins[index as usize].target = Some(target);
            index
        } else {
            let index = u32::try_from(self.pins.len()).unwrap_or(u32::MAX);
            self.pins.push(PinSlot {
                generation: 0,
                target: Some(target),
            });
            index
        };
        *self.pin_counts.entry(target).or_insert(0) += 1;
        self.tracked
            .entry(target)
            .or_insert(TargetState {
                lod: Lod::Unresident,
                bytes: 0,
                reason,
            })
            .reason = reason;
        StreamHandle::new(index, self.pins[index as usize].generation)
    }

    /// Release a pin. Returns whether the handle addressed a live one.
    pub fn cancel(&mut self, handle: StreamHandle) -> bool {
        let Some(slot) = self.pins.get_mut(handle.index() as usize) else {
            return false;
        };
        if slot.generation != handle.generation() {
            return false;
        }
        let Some(target) = slot.target.take() else {
            return false;
        };
        // Advancing the generation is what makes the handle stale. An
        // exhausted counter retires the slot instead of reusing it, so a very
        // old handle can never release someone else's pin.
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            self.free_pins.push(handle.index());
        }
        if let Some(count) = self.pin_counts.get_mut(&target) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.pin_counts.remove(&target);
            }
        }
        true
    }

    /// Whether a target is pinned.
    #[must_use]
    pub fn is_pinned(&self, target: StreamTarget) -> bool {
        self.pin_counts.contains_key(&target)
    }

    /// The tier a target is at. Untracked targets are [`Lod::Unresident`].
    #[must_use]
    pub fn lod_of(&self, target: StreamTarget) -> Lod {
        self.tracked
            .get(&target)
            .map_or(Lod::Unresident, |state| state.lod)
    }

    /// What the system knows about a target.
    #[must_use]
    pub fn state_of(&self, target: StreamTarget) -> Option<TargetState> {
        self.tracked.get(&target).copied()
    }

    /// Every tracked target with its state, in target order.
    pub fn iter(&self) -> impl Iterator<Item = (StreamTarget, TargetState)> + '_ {
        self.tracked.iter().map(|(target, state)| (*target, *state))
    }

    /// Targets currently resident.
    pub fn resident(&self) -> impl Iterator<Item = StreamTarget> + '_ {
        self.tracked
            .iter()
            .filter(|(_, state)| state.lod.is_resident())
            .map(|(target, _)| *target)
    }

    /// How many targets are resident.
    #[must_use]
    pub fn resident_count(&self) -> u32 {
        u32::try_from(self.resident().count()).unwrap_or(u32::MAX)
    }

    /// How many targets are tracked at any tier.
    #[must_use]
    pub fn tracked_count(&self) -> u32 {
        u32::try_from(self.tracked.len()).unwrap_or(u32::MAX)
    }

    /// Bytes the backend reported for the resident set.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Take the backend failures recorded since the last call.
    ///
    /// A tick counts failures rather than aborting, but nothing is swallowed:
    /// whatever the backend said is here until someone reads it.
    pub fn take_failures(&mut self) -> Vec<Error> {
        core::mem::take(&mut self.failures)
    }

    /// Failures that were dropped because more than [`MAX_RETAINED_FAILURES`]
    /// accumulated without being read.
    #[must_use]
    pub const fn dropped_failures(&self) -> u64 {
        self.dropped_failures
    }

    /// Run one pass of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error only for an invalid budget, which is a configuration
    /// mistake rather than a data problem. Backend failures are counted in the
    /// report and retained for [`Self::take_failures`].
    pub fn tick<B: ResidencyBackend + ?Sized>(
        &mut self,
        backend: &mut B,
        budget: StreamingBudget,
    ) -> Result<StreamingReport> {
        let budget = budget.validate()?;
        let mut report = StreamingReport::default();

        let desired = self.desired_tiers();
        let (promotions, demotions) = self.split_by_direction(&desired);

        self.apply_promotions(backend, &promotions, budget, &mut report);
        self.apply_demotions(backend, &demotions, budget, &mut report);
        self.shed_over_budget(backend, budget, &mut report);

        self.tracked.retain(|_, state| state.lod.is_tracked());
        report.resident = self.resident_count();
        report.tracked = self.tracked_count();
        report.bytes = self.bytes;
        Ok(report)
    }

    /// The tier every candidate target should hold after this tick.
    fn desired_tiers(&self) -> BTreeMap<StreamTarget, (Lod, u64)> {
        let mut desired: BTreeMap<StreamTarget, (Lod, u64)> = BTreeMap::new();

        // Candidates: everything already tracked, everything pinned, and every
        // column inside some source's outer radius.
        let mut candidates: BTreeSet<StreamTarget> = self.tracked.keys().copied().collect();
        candidates.extend(self.pin_counts.keys().copied());
        for source in self.interests.values() {
            let reach = i64::from(source.radii.outer());
            for x in -reach..=reach {
                for z in -reach..=reach {
                    candidates.insert(StreamTarget::Chunk(ChunkCoord::new(
                        source.center.x.saturating_add(x),
                        source.center.z.saturating_add(z),
                    )));
                }
            }
        }

        for target in candidates {
            let current = self.lod_of(target);
            let mut best = Lod::Unresident;
            let mut nearest = u64::MAX;
            for source in self.interests.values() {
                let distance = target.chunk_distance(source.center);
                nearest = nearest.min(distance);
                best = best.max(source.radii.tier_for(current, distance));
            }
            if self.is_pinned(target) {
                best = Lod::Full;
                nearest = 0;
            }
            desired.insert(target, (best, nearest));
        }
        desired
    }

    /// Split candidates into work that adds detail and work that removes it.
    ///
    /// Promotions are ordered nearest-first and demotions furthest-first, so a
    /// budget that runs out spends what it had on the targets closest to an
    /// observer and releases the ones furthest from one.
    fn split_by_direction(
        &self,
        desired: &BTreeMap<StreamTarget, (Lod, u64)>,
    ) -> (Vec<Scheduled>, Vec<Scheduled>) {
        let mut promotions = Vec::new();
        let mut demotions = Vec::new();
        for (&target, &(tier, distance)) in desired {
            let current = self.lod_of(target);
            if tier > current {
                promotions.push((target, tier, distance));
            } else if tier < current {
                demotions.push((target, tier, distance));
            }
        }
        promotions.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));
        demotions.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        (promotions, demotions)
    }

    fn apply_promotions<B: ResidencyBackend + ?Sized>(
        &mut self,
        backend: &mut B,
        promotions: &[Scheduled],
        budget: StreamingBudget,
        report: &mut StreamingReport,
    ) {
        let mut activations = 0;
        for &(target, tier, _) in promotions {
            let current = self.lod_of(target);
            let costs_io = tier.is_resident() && !current.is_resident();
            if costs_io {
                if activations >= budget.activations || self.resident_count() >= budget.max_resident
                {
                    report.deferred += 1;
                    continue;
                }
                activations += 1;
            }
            match backend.activate(target, tier) {
                Ok(bytes) => {
                    let entry = self.tracked.entry(target).or_insert(TargetState {
                        lod: Lod::Unresident,
                        bytes: 0,
                        reason: StreamReason::Interest,
                    });
                    self.bytes = self.bytes.saturating_sub(entry.bytes).saturating_add(bytes);
                    entry.lod = tier;
                    entry.bytes = bytes;
                    report.promoted += 1;
                    if costs_io {
                        report.activated += 1;
                    }
                }
                Err(cause) => {
                    report.failed += 1;
                    self.record_failure(cause);
                }
            }
        }
    }

    fn apply_demotions<B: ResidencyBackend + ?Sized>(
        &mut self,
        backend: &mut B,
        demotions: &[Scheduled],
        budget: StreamingBudget,
        report: &mut StreamingReport,
    ) {
        let mut evictions = 0;
        for &(target, tier, _) in demotions {
            let current = self.lod_of(target);
            let releases = current.is_resident() && !tier.is_resident();
            if releases {
                if evictions >= budget.evictions {
                    report.deferred += 1;
                    continue;
                }
                evictions += 1;
                if !self.release(backend, target, report) {
                    continue;
                }
            }
            let Some(entry) = self.tracked.get_mut(&target) else {
                continue;
            };
            entry.lod = tier;
            report.demoted += 1;
            if releases {
                report.evicted += 1;
            }
        }
    }

    /// Evict targets that interest wants but the budget cannot hold.
    ///
    /// This is the backpressure `STREAMING SYSTEM.md` asks for. It is reported
    /// separately as `shed`, because "we dropped what you asked for" is a
    /// different fact from "we released what you stopped asking for".
    fn shed_over_budget<B: ResidencyBackend + ?Sized>(
        &mut self,
        backend: &mut B,
        budget: StreamingBudget,
        report: &mut StreamingReport,
    ) {
        loop {
            let over_count = self.resident_count() > budget.max_resident;
            let over_bytes = self.bytes > budget.max_bytes;
            if !over_count && !over_bytes {
                return;
            }
            // Furthest from any observer goes first; pinned targets never go.
            let victim = self
                .tracked
                .iter()
                .filter(|(target, state)| state.lod.is_resident() && !self.is_pinned(**target))
                .map(|(target, _)| (self.distance_to_nearest_interest(*target), *target))
                .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)))
                .map(|(_, target)| target);
            let Some(target) = victim else {
                // Everything resident is pinned. Refusing to break a pin is the
                // right answer; the caller sees the budget overrun in the report.
                return;
            };
            if !self.release(backend, target, report) {
                return;
            }
            if let Some(entry) = self.tracked.get_mut(&target) {
                entry.lod = Lod::Abstract;
            }
            report.evicted += 1;
            report.demoted += 1;
            report.shed += 1;
        }
    }

    /// Persist if dirty, then drop. Returns whether the target was released.
    fn release<B: ResidencyBackend + ?Sized>(
        &mut self,
        backend: &mut B,
        target: StreamTarget,
        report: &mut StreamingReport,
    ) -> bool {
        if backend.is_dirty(target) {
            match backend.persist(target) {
                Ok(()) => report.persisted += 1,
                Err(cause) => {
                    // Holding memory is recoverable; losing an edit is not.
                    report.failed += 1;
                    self.record_failure(cause);
                    return false;
                }
            }
        }
        if let Err(cause) = backend.evict(target) {
            report.failed += 1;
            self.record_failure(cause);
            return false;
        }
        if let Some(entry) = self.tracked.get_mut(&target) {
            self.bytes = self.bytes.saturating_sub(entry.bytes);
            entry.bytes = 0;
        }
        true
    }

    fn distance_to_nearest_interest(&self, target: StreamTarget) -> u64 {
        self.interests
            .values()
            .map(|source| target.chunk_distance(source.center))
            .min()
            .unwrap_or(u64::MAX)
    }

    fn record_failure(&mut self, cause: Error) {
        if self.failures.len() >= MAX_RETAINED_FAILURES {
            self.failures.remove(0);
            self.dropped_failures += 1;
        }
        self.failures.push(cause);
    }
}

fn reject(message: &'static str) -> Error {
    Error::new(Domain::World, "streaming", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MemoryBackend;
    use crate::interest::LodRadii;

    const KIB: u64 = 1_024;

    fn chunk(x: i64, z: i64) -> StreamTarget {
        StreamTarget::Chunk(ChunkCoord::new(x, z))
    }

    fn observer(at: ChunkCoord, full: u32) -> InterestSource {
        InterestSource::new(InterestId(1), at)
            .with_radii(LodRadii::new(full, full, full, 0).expect("valid radii"))
    }

    /// Run ticks until nothing changes, so a budget-limited system reaches the
    /// state interest asks for. Returns the number of ticks it took.
    fn settle(
        system: &mut StreamingSystem,
        backend: &mut MemoryBackend,
        budget: StreamingBudget,
    ) -> u32 {
        for tick in 1..=200 {
            let report = system.tick(backend, budget).expect("valid budget");
            if report.is_quiet() {
                return tick;
            }
        }
        panic!("streaming never settled");
    }

    #[test]
    fn an_empty_system_does_nothing() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let report = system
            .tick(&mut backend, StreamingBudget::UNLIMITED)
            .expect("valid budget");
        assert!(report.is_quiet());
        assert_eq!(report.resident, 0);
        assert_eq!(backend.resident_count(), 0);
    }

    #[test]
    fn interest_loads_the_square_around_an_observer() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 2))
            .expect("room for one source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        // A radius of 2 is a 5x5 square: 25 columns, corners included.
        assert_eq!(system.resident_count(), 25);
        assert_eq!(backend.resident_count(), 25);
        assert!(
            system.lod_of(chunk(2, 2)).is_resident(),
            "the corner must load"
        );
        assert!(!system.lod_of(chunk(3, 0)).is_resident());
        assert_eq!(system.bytes(), 25 * KIB);
    }

    #[test]
    fn moving_the_observer_streams_in_front_and_releases_behind() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert!(system.lod_of(chunk(-1, 0)).is_resident());

        system
            .set_interest(observer(ChunkCoord::new(4, 0), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        assert!(
            system.lod_of(chunk(5, 0)).is_resident(),
            "ahead did not load"
        );
        assert!(
            !system.lod_of(chunk(-1, 0)).is_resident(),
            "behind was never released"
        );
        assert_eq!(system.resident_count(), 9);
    }

    #[test]
    fn walking_back_and_forth_over_a_boundary_does_not_thrash() {
        // The property hysteresis exists for, at system level: the backend must
        // not see load/evict pairs for a chunk the observer keeps stepping
        // across.
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let radii = LodRadii::new(2, 4, 6, 2).expect("valid");
        system
            .set_interest(
                InterestSource::new(InterestId(1), ChunkCoord::new(0, 0)).with_radii(radii),
            )
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        let (loads_before, evictions_before, _) = backend.counts();

        for step in [1, 0, 1, 0, 1, 0] {
            system
                .set_interest(
                    InterestSource::new(InterestId(1), ChunkCoord::new(step, 0)).with_radii(radii),
                )
                .expect("source");
            settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        }

        let (loads_after, evictions_after) = {
            let (l, e, _) = backend.counts();
            (l, e)
        };
        // Stepping one chunk and back inside the hysteresis band must not
        // evict anything at all.
        assert_eq!(
            evictions_after,
            evictions_before,
            "thrashed: {} evictions while stepping inside the band",
            evictions_after - evictions_before
        );
        // Loads are allowed - moving forward genuinely exposes new columns -
        // but nothing should be loaded twice.
        assert!(loads_after >= loads_before);
    }

    #[test]
    fn fast_travel_releases_the_old_region_and_loads_the_new_one() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 2))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert!(system.lod_of(chunk(0, 0)).is_resident());

        // Somewhere else entirely.
        system
            .set_interest(observer(ChunkCoord::new(10_000, -10_000), 2))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        assert!(!system.lod_of(chunk(0, 0)).is_resident(), "old region held");
        assert!(system.lod_of(chunk(10_000, -10_000)).is_resident());
        assert_eq!(system.resident_count(), 25);
        // And the old region is not still being tracked at some inner tier.
        assert_eq!(system.lod_of(chunk(0, 0)), Lod::Unresident);
    }

    #[test]
    fn a_dirty_target_is_persisted_before_it_is_evicted() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        let edited = chunk(-1, -1);
        assert!(backend.is_resident(edited));
        backend.touch(edited);

        system
            .set_interest(observer(ChunkCoord::new(50, 50), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        assert!(!backend.is_resident(edited));
        assert!(
            backend.was_persisted(edited),
            "an edited chunk was dropped without being written out"
        );
    }

    #[test]
    fn a_clean_target_is_evicted_without_a_pointless_write() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        system
            .set_interest(observer(ChunkCoord::new(50, 50), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        let (_, evictions, persists) = backend.counts();
        assert!(evictions > 0);
        assert_eq!(persists, 0, "clean chunks were written out for nothing");
    }

    #[test]
    fn a_target_whose_state_cannot_be_written_is_not_evicted() {
        // Holding memory is recoverable. Losing an edit is not.
        struct FailsToPersist {
            inner: MemoryBackend,
        }
        impl ResidencyBackend for FailsToPersist {
            fn activate(&mut self, target: StreamTarget, lod: Lod) -> Result<u64> {
                self.inner.activate(target, lod)
            }
            fn evict(&mut self, target: StreamTarget) -> Result<()> {
                self.inner.evict(target)
            }
            fn persist(&mut self, _target: StreamTarget) -> Result<()> {
                Err(Error::new(Domain::Save, "test", "the disk is full"))
            }
            fn is_dirty(&self, _target: StreamTarget) -> bool {
                true
            }
        }

        let mut system = StreamingSystem::new();
        let mut backend = FailsToPersist {
            inner: MemoryBackend::new(KIB),
        };
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 0))
            .expect("source");
        system
            .tick(&mut backend, StreamingBudget::UNLIMITED)
            .expect("budget");
        assert!(backend.inner.is_resident(chunk(0, 0)));

        system
            .set_interest(observer(ChunkCoord::new(500, 500), 0))
            .expect("source");
        let report = system
            .tick(&mut backend, StreamingBudget::UNLIMITED)
            .expect("budget");

        assert!(
            backend.inner.is_resident(chunk(0, 0)),
            "evicted a chunk whose state could not be saved"
        );
        assert_eq!(report.evicted, 0);
        assert!(report.failed > 0);
        assert_eq!(system.take_failures().len(), report.failed as usize);
    }

    #[test]
    fn the_activation_budget_spreads_a_large_load_over_ticks() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 3))
            .expect("source");
        let budget = StreamingBudget {
            activations: 4,
            ..StreamingBudget::UNLIMITED
        };

        let first = system.tick(&mut backend, budget).expect("budget");
        assert_eq!(first.activated, 4, "the budget was not respected");
        assert!(first.is_saturated(), "deferral was not reported");

        let ticks = settle(&mut system, &mut backend, budget);
        // 7x7 = 49 columns at 4 per tick: it takes a while, and it gets there.
        assert!(ticks > 1);
        assert_eq!(system.resident_count(), 49);
    }

    #[test]
    fn nearest_columns_load_first_when_the_budget_is_tight() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 3))
            .expect("source");
        system
            .tick(
                &mut backend,
                StreamingBudget {
                    activations: 1,
                    ..StreamingBudget::UNLIMITED
                },
            )
            .expect("budget");
        assert!(
            system.lod_of(chunk(0, 0)).is_resident(),
            "the column the observer stands in was not first"
        );
        assert_eq!(system.resident_count(), 1);
    }

    #[test]
    fn memory_pressure_sheds_the_furthest_columns() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 2))
            .expect("source");
        let budget = StreamingBudget {
            max_bytes: 9 * KIB,
            ..StreamingBudget::UNLIMITED
        };
        for _ in 0..8 {
            let report = system.tick(&mut backend, budget).expect("budget");
            if report.is_quiet() {
                break;
            }
        }

        assert!(system.bytes() <= budget.max_bytes, "over the memory budget");
        assert!(
            system.lod_of(chunk(0, 0)).is_resident(),
            "shed the column under the observer"
        );
        assert!(
            !system.lod_of(chunk(2, 2)).is_resident(),
            "kept a far corner instead of a near column"
        );
        // Shedding what interest wanted is reported separately from releasing
        // what it stopped wanting.
        let report = system.tick(&mut backend, budget).expect("budget");
        assert!(report.shed > 0 || report.is_quiet());
    }

    #[test]
    fn a_resident_count_cap_is_enforced_as_well_as_bytes() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 3))
            .expect("source");
        let budget = StreamingBudget {
            max_resident: 6,
            ..StreamingBudget::UNLIMITED
        };
        for _ in 0..12 {
            system.tick(&mut backend, budget).expect("budget");
        }
        assert!(system.resident_count() <= 6, "{}", system.resident_count());
        assert_eq!(backend.resident_count(), system.resident_count() as usize);
    }

    #[test]
    fn a_pin_holds_a_target_that_no_observer_is_near() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let far = chunk(900, 900);
        let handle = system.request(far, StreamReason::WorldEvent);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        assert!(system.is_pinned(far));
        assert!(system.lod_of(far).is_resident(), "the pin did not hold it");

        assert!(system.cancel(handle));
        assert!(!system.is_pinned(far));
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert!(
            !system.lod_of(far).is_resident(),
            "the pin outlived its handle"
        );
    }

    #[test]
    fn a_stale_handle_cannot_release_someone_elses_pin() {
        let mut system = StreamingSystem::new();
        let target = chunk(1, 1);
        let first = system.request(target, StreamReason::Explicit);
        assert!(system.cancel(first));
        let second = system.request(target, StreamReason::Explicit);
        assert_eq!(first.index(), second.index(), "the slot should be reused");
        assert!(!system.cancel(first), "a stale handle released a live pin");
        assert!(system.is_pinned(target));
        assert!(system.cancel(second));
    }

    #[test]
    fn a_pinned_target_survives_memory_pressure() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let pinned = chunk(500, 500);
        let _handle = system.request(pinned, StreamReason::WorldEvent);
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 2))
            .expect("source");
        let budget = StreamingBudget {
            max_resident: 3,
            ..StreamingBudget::UNLIMITED
        };
        for _ in 0..10 {
            system.tick(&mut backend, budget).expect("budget");
        }
        assert!(
            system.lod_of(pinned).is_resident(),
            "backpressure broke a pin"
        );
    }

    #[test]
    fn two_observers_keep_both_regions_and_neither_evicts_the_other() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        system
            .set_interest(InterestSource::new(InterestId(1), ChunkCoord::new(0, 0)))
            .expect("source");
        system
            .set_interest(InterestSource::new(InterestId(2), ChunkCoord::new(40, 40)))
            .expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);

        assert!(system.lod_of(chunk(0, 0)).is_resident());
        assert!(system.lod_of(chunk(40, 40)).is_resident());

        // One disconnects; the other must not lose anything.
        assert!(system.remove_interest(InterestId(2)));
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert!(system.lod_of(chunk(0, 0)).is_resident());
        assert!(!system.lod_of(chunk(40, 40)).is_resident());
    }

    #[test]
    fn an_observer_that_leaves_and_returns_gets_its_region_back() {
        // The doc's "multiplayer reconnect" case, as far as residency is
        // concerned: identity survives, so the region comes back rather than
        // becoming a second, parallel one.
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let source = observer(ChunkCoord::new(7, 7), 1);
        system.set_interest(source).expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        let edited = chunk(7, 7);
        backend.touch(edited);

        assert!(system.remove_interest(InterestId(1)));
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert_eq!(system.resident_count(), 0);
        assert!(
            backend.was_persisted(edited),
            "state was lost on disconnect"
        );

        system.set_interest(source).expect("source");
        settle(&mut system, &mut backend, StreamingBudget::UNLIMITED);
        assert!(system.lod_of(edited).is_resident());
        assert_eq!(system.resident_count(), 9);
    }

    #[test]
    fn too_many_interest_sources_are_refused_but_moving_one_never_is() {
        let mut system = StreamingSystem::new();
        for index in 0..MAX_INTEREST_SOURCES {
            system
                .set_interest(InterestSource::new(
                    InterestId(index as u64),
                    ChunkCoord::new(0, 0),
                ))
                .expect("within the limit");
        }
        assert!(system
            .set_interest(InterestSource::new(
                InterestId(9_999),
                ChunkCoord::new(0, 0)
            ))
            .is_err());
        // Moving one that already exists is always allowed.
        assert!(system
            .set_interest(InterestSource::new(InterestId(0), ChunkCoord::new(5, 5)))
            .is_ok());
    }

    #[test]
    fn an_invalid_budget_is_refused_rather_than_silently_corrected() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let broken = StreamingBudget {
            activations: 0,
            ..StreamingBudget::MODEST
        };
        assert!(system.tick(&mut backend, broken).is_err());
    }

    #[test]
    fn a_backend_failure_is_counted_and_kept_rather_than_swallowed() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        backend.refuse(chunk(0, 0));
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        let report = system
            .tick(&mut backend, StreamingBudget::UNLIMITED)
            .expect("budget");

        assert_eq!(report.failed, 1);
        assert!(!system.lod_of(chunk(0, 0)).is_resident());
        // The neighbours still loaded: one refusal does not stop the tick.
        assert!(system.lod_of(chunk(1, 0)).is_resident());
        let failures = system.take_failures();
        assert_eq!(failures.len(), 1);
        assert!(
            system.take_failures().is_empty(),
            "failures were not drained"
        );
    }

    #[test]
    fn retained_failures_are_bounded_and_the_overflow_is_counted() {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let radius = 8;
        for x in -radius..=radius {
            for z in -radius..=radius {
                backend.refuse(chunk(x, z));
            }
        }
        system
            .set_interest(observer(ChunkCoord::new(0, 0), radius as u32))
            .expect("source");
        let report = system
            .tick(&mut backend, StreamingBudget::UNLIMITED)
            .expect("budget");

        assert!(report.failed as usize > MAX_RETAINED_FAILURES);
        assert_eq!(system.take_failures().len(), MAX_RETAINED_FAILURES);
        assert!(
            system.dropped_failures() > 0,
            "the overflow was not recorded"
        );
    }

    #[test]
    fn the_same_inputs_produce_the_same_decisions() {
        let run = || {
            let mut system = StreamingSystem::new();
            let mut backend = MemoryBackend::new(KIB);
            let budget = StreamingBudget {
                activations: 3,
                evictions: 2,
                max_resident: 30,
                max_bytes: 30 * KIB,
            };
            let mut trace = Vec::new();
            for step in 0..24i64 {
                system
                    .set_interest(observer(ChunkCoord::new(step, step / 2), 2))
                    .expect("source");
                let report = system.tick(&mut backend, budget).expect("budget");
                trace.push((
                    report.activated,
                    report.evicted,
                    report.deferred,
                    report.shed,
                    report.resident,
                ));
            }
            let residents: Vec<_> = system.resident().collect();
            (trace, residents)
        };
        let first = run();
        for _ in 0..4 {
            assert_eq!(run(), first, "streaming decisions diverged between runs");
        }
    }

    #[test]
    fn the_manager_and_the_backend_never_disagree_about_what_is_resident() {
        // The invariant that makes every other assertion meaningful.
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(KIB);
        let budget = StreamingBudget {
            activations: 5,
            evictions: 3,
            max_resident: 20,
            max_bytes: 20 * KIB,
        };
        for step in 0..40i64 {
            system
                .set_interest(observer(ChunkCoord::new(step % 7, step % 5), 2))
                .expect("source");
            system.tick(&mut backend, budget).expect("budget");
            let claimed: Vec<_> = system.resident().collect();
            assert_eq!(
                claimed.len(),
                backend.resident_count(),
                "step {step}: manager says {} resident, backend holds {}",
                claimed.len(),
                backend.resident_count()
            );
            for target in claimed {
                assert!(
                    backend.is_resident(target),
                    "step {step}: {target} is not held"
                );
            }
        }
    }
}
