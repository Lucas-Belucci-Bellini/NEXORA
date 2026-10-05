//! What of the world the client holds: the columns the scene draws and the
//! ring around them, following the player (ADR-0038).
//!
//! The client does not decide residency itself. It names one interest — the
//! player's column, at the scene's radius plus the ring a drawn edge is
//! meshed against — and the engine's [`StreamingSystem`] activates and evicts
//! through [`WorldResidency`], the one place voxel chunks and streaming meet
//! (ADR-0008). That is what keeps an edit alive across eviction: an unedited
//! column is dropped and generated again identically, an edited one is held
//! in [`RetainedChunks`] and given back when the player returns. A column the
//! client generated itself, around the backend, would come back from the seed
//! with its edits gone.
//!
//! The budget is unlimited, and a move settles before the scene is meshed
//! again: the scene draws a full square or nothing, because the reference ray
//! cast walks one box (ADR-0030). What that costs a frame is measured and
//! reported (`streaming` in the report), not hidden.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::ChunkCoord;
use nexora_simulation::{RetainedChunks, WorldResidency};
use nexora_streaming::budget::StreamingBudget;
use nexora_streaming::interest::{InterestId, InterestSource, LodRadii};
use nexora_streaming::system::StreamingSystem;
use nexora_world::world::World;

/// The player's interest: the only one the client has.
const PLAYER: InterestId = InterestId(1);

/// Columns of overshoot before one is evicted: stepping back and forth over a
/// column's edge does not generate the same columns again and again.
pub const HYSTERESIS: u32 = 1;

/// Ticks a move may take to settle. With an unlimited budget one tick
/// activates and evicts everything; the bound only turns a manager that never
/// settles into an error instead of a hang.
const SETTLE_TICKS: u32 = 8;

/// What residency has done so far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResidencyTally {
    /// Columns activated: generated, or given back from retention.
    pub activated: u64,
    /// Columns evicted from the world.
    pub evicted: u64,
    /// Evicted columns that were edited, and so retained rather than dropped.
    pub persisted: u64,
}

/// The client's residency: the streaming manager, and the edited columns it
/// evicted.
#[derive(Debug)]
pub struct ClientResidency {
    system: StreamingSystem,
    retained: RetainedChunks,
    radii: LodRadii,
    tally: ResidencyTally,
}

impl ClientResidency {
    /// Residency for a scene of `radius` columns around the player: that and
    /// one ring more are kept, every one of them at full detail.
    ///
    /// # Errors
    ///
    /// The radius is negative or beyond what streaming tracks.
    pub fn new(radius: i64) -> Result<Self> {
        let reach = u32::try_from(radius + 1).map_err(|_| {
            Error::new(
                Domain::World,
                "client-residency",
                "the radius must not be negative",
            )
            .with_recovery(Recovery::Reject)
            .with_context("radius", radius.to_string())
        })?;
        Ok(Self {
            system: StreamingSystem::new(),
            retained: RetainedChunks::new(),
            radii: LodRadii::new(reach, reach, reach, HYSTERESIS)?,
            tally: ResidencyTally::default(),
        })
    }

    /// Make the columns around `centre` resident, and let go of those too far
    /// from it, before returning.
    ///
    /// # Errors
    ///
    /// A column failed to activate, evict or persist, or the manager did not
    /// settle in [`SETTLE_TICKS`].
    pub fn settle(&mut self, world: &mut World, centre: ChunkCoord) -> Result<()> {
        self.system
            .set_interest(InterestSource::new(PLAYER, centre).with_radii(self.radii))?;
        for _ in 0..SETTLE_TICKS {
            let mut backend = WorldResidency::new(world, &mut self.retained);
            let report = self.system.tick(&mut backend, StreamingBudget::UNLIMITED)?;
            if let Some(failure) = self.system.take_failures().into_iter().next() {
                return Err(failure);
            }
            self.tally.activated += u64::from(report.activated);
            self.tally.evicted += u64::from(report.evicted);
            self.tally.persisted += u64::from(report.persisted);
            if report.is_quiet() {
                return Ok(());
            }
        }
        Err(Error::new(
            Domain::World,
            "client-residency",
            "streaming did not settle around the player",
        )
        .with_recovery(Recovery::Manual)
        .with_context("centre", format!("{},{}", centre.x, centre.z))
        .with_context("ticks", SETTLE_TICKS.to_string()))
    }

    /// Put every retained column back into `world`: before any save, or the
    /// save is missing every edit made in an evicted column
    /// ([`RetainedChunks::flush_into`]).
    ///
    /// # Errors
    ///
    /// A column could not be read back.
    pub fn flush_into(&mut self, world: &mut World) -> Result<usize> {
        self.retained.flush_into(world)
    }

    /// Edited columns held out of the world right now.
    #[must_use]
    pub fn retained(&self) -> usize {
        self.retained.len()
    }

    /// Columns resident by the manager's account.
    #[must_use]
    pub fn resident(&self) -> u32 {
        self.system.resident_count()
    }

    /// What residency has done so far.
    #[must_use]
    pub const fn tally(&self) -> ResidencyTally {
        self.tally
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::spatial::BlockPos;
    use nexora_foundation::time::CalendarConfig;
    use nexora_world::world::WorldDescriptor;

    fn world() -> World {
        let mut world = World::create(
            WorldDescriptor::new("client-residency", 11).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        world
    }

    #[test]
    fn the_scene_and_its_ring_are_resident_and_nothing_more() {
        let mut world = world();
        let mut residency = ClientResidency::new(1).unwrap();
        residency.settle(&mut world, ChunkCoord::new(0, 0)).unwrap();
        assert_eq!(world.chunk_count(), 25);
        assert_eq!(residency.resident(), 25);
        for x in -2..=2 {
            for z in -2..=2 {
                assert!(world.chunk(ChunkCoord::new(x, z)).is_some(), "({x},{z})");
            }
        }
    }

    /// Walking away evicts what falls behind — past the hysteresis — and an
    /// edited column comes back with its edit when the player returns.
    #[test]
    fn an_edit_survives_walking_away_and_back() {
        let mut world = world();
        let mut residency = ClientResidency::new(1).unwrap();
        residency.settle(&mut world, ChunkCoord::new(0, 0)).unwrap();
        let stone = world
            .block_id(&Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        let top = world.surface_height(-20, 3);
        let cell = BlockPos::new(-20, top + 3, 3);
        world.set_block(cell, stone).unwrap();

        residency.settle(&mut world, ChunkCoord::new(4, 0)).unwrap();
        assert!(world.chunk(ChunkCoord::new(-2, 0)).is_none(), "evicted");
        assert_eq!(residency.retained(), 1, "the edited column is retained");
        assert!(residency.tally().evicted > 0);
        assert_eq!(residency.tally().persisted, 1);

        residency.settle(&mut world, ChunkCoord::new(0, 0)).unwrap();
        assert_eq!(world.get_block(cell).unwrap(), stone, "the edit came back");
        assert_eq!(residency.retained(), 0);
    }

    /// Before a save, retained columns go back into the world.
    #[test]
    fn a_flush_puts_retained_columns_back_for_the_save() {
        let mut world = world();
        let mut residency = ClientResidency::new(1).unwrap();
        residency.settle(&mut world, ChunkCoord::new(0, 0)).unwrap();
        let stone = world
            .block_id(&Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        let cell = BlockPos::new(-20, world.surface_height(-20, 3) + 3, 3);
        world.set_block(cell, stone).unwrap();
        residency.settle(&mut world, ChunkCoord::new(4, 0)).unwrap();
        assert!(world.chunk(ChunkCoord::new(-2, 0)).is_none());
        assert_eq!(residency.flush_into(&mut world).unwrap(), 1);
        assert_eq!(world.get_block(cell).unwrap(), stone);
    }

    #[test]
    fn a_negative_radius_is_refused() {
        assert!(ClientResidency::new(-2).is_err());
    }
}
