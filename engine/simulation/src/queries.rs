//! The world, readable through queries.
//!
//! `NEXORA DATA OWNERSHIP AND SOURCE OF TRUTH.md` makes the block/voxel system
//! the owner of block state and the time system the owner of world time, and
//! says everyone else reads them *"by query / snapshot / public view"*. These
//! are those queries. They live here for the reason the block command handlers
//! do: this is the crate that may see the world, and `nexora-query`, like
//! `nexora-command`, must not.
//!
//! # Identifiers, never runtime ids
//!
//! `NEXORA PUBLIC API AND CONTRACTS.md`: *"runtime handles are not stable
//! persistence identifiers unless explicitly declared."* A block state id means
//! whatever the registry happened to assign in this session, so an answer
//! carrying one would be an answer a mod could cache and be wrong about after
//! the next restart. Every answer here names blocks by [`Identifier`].
//!
//! # Not resident is not empty
//!
//! A cell in a chunk that is not loaded is [`QueryFailure::Unavailable`], never
//! air — the same call the mesher and physics make, for the same reason.

use nexora_command::definition::SourcePolicy;
use nexora_command::identity::{ActorKind, Source};
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::BlockPos;
use nexora_foundation::time::WorldTime;
use nexora_foundation::version::QueryVersion;
use nexora_query::{Budget, Query, QueryDefinition, QueryFailure, QueryService};
use nexora_world::voxel::AIR;
use nexora_world::world::World;

/// `nexora:block_at` — which block is at one position.
pub const BLOCK_AT: &str = "nexora:block_at";
/// `nexora:blocks_in_region` — every non-air block in a box.
pub const BLOCKS_IN_REGION: &str = "nexora:blocks_in_region";
/// `nexora:world_time` — the authoritative world clock.
pub const WORLD_TIME: &str = "nexora:world_time";

/// The version of every world query defined here.
pub const WORLD_QUERY_VERSION: QueryVersion = QueryVersion(1);

/// The most cells one region query may scan.
///
/// A result budget bounds what comes back; this bounds what it costs to find
/// out. Without it, a region of a million air cells answers with nothing and
/// still costs a million lookups.
pub const MAX_REGION_CELLS: u64 = 32_768;

/// The result budget of a region query: a full 32³ box of solid blocks.
pub const REGION_MAX_RESULTS: usize = 32_768;

type Answer<T> = std::result::Result<T, QueryFailure>;

/// Which block is at a position.
#[derive(Debug, Clone)]
pub struct BlockAt {
    id: Identifier,
}

/// Every non-air block in an inclusive box, lowest `y`, then `z`, then `x`
/// first — an order that does not depend on how chunks happen to be stored.
#[derive(Debug, Clone)]
pub struct BlocksInRegion {
    id: Identifier,
}

/// The world's authoritative time.
#[derive(Debug, Clone)]
pub struct WorldTimeQuery {
    id: Identifier,
}

/// The world queries, constructed.
#[derive(Debug, Clone)]
pub struct WorldQueries {
    /// [`BLOCK_AT`].
    pub block_at: BlockAt,
    /// [`BLOCKS_IN_REGION`].
    pub blocks_in_region: BlocksInRegion,
    /// [`WORLD_TIME`].
    pub world_time: WorldTimeQuery,
}

impl WorldQueries {
    /// Register the world query definitions and return the queries.
    ///
    /// Who may ask: every actor that acts in the world, from every door. A
    /// player over the network may ask what block is somewhere, because the
    /// server is about to send it that block anyway; what keeps a query from
    /// becoming raw world memory is the budget, not the door.
    ///
    /// # Errors
    ///
    /// Returns an error when a definition is already registered or the
    /// service is frozen.
    pub fn register(service: &mut QueryService) -> Result<Self> {
        let access = SourcePolicy::closed()
            .allow_actor(ActorKind::Player)
            .allow_actor(ActorKind::Npc)
            .allow_actor(ActorKind::Ai)
            .allow_actor(ActorKind::Script)
            .allow_actor(ActorKind::Mod)
            .allow_actor(ActorKind::Machine)
            .allow_actor(ActorKind::Admin)
            .allow_actor(ActorKind::ServerSystem)
            .allow_source(Source::Network)
            .allow_source(Source::Local)
            .allow_source(Source::ModRuntime)
            .allow_source(Source::Console)
            .allow_source(Source::Replay);
        let owner_blocks = "nexora:system/voxel";
        service.register(
            QueryDefinition::new(BLOCK_AT, WORLD_QUERY_VERSION, owner_blocks)?
                .with_access(access.clone())
                .with_max_results(1),
        )?;
        service.register(
            QueryDefinition::new(BLOCKS_IN_REGION, WORLD_QUERY_VERSION, owner_blocks)?
                .with_access(access.clone())
                .with_max_results(REGION_MAX_RESULTS),
        )?;
        service.register(
            QueryDefinition::new(WORLD_TIME, WORLD_QUERY_VERSION, "nexora:system/time")?
                .with_access(access)
                .with_max_results(1),
        )?;
        Ok(Self {
            block_at: BlockAt {
                id: Identifier::parse(BLOCK_AT)?,
            },
            blocks_in_region: BlocksInRegion {
                id: Identifier::parse(BLOCKS_IN_REGION)?,
            },
            world_time: WorldTimeQuery {
                id: Identifier::parse(WORLD_TIME)?,
            },
        })
    }
}

fn identifier_at(world: &World, at: BlockPos) -> Answer<Identifier> {
    let state = world.get_block(at).map_err(|_| {
        QueryFailure::Unavailable(format!("{},{},{} is not resident", at.x, at.y, at.z))
    })?;
    world.block_identifier(state).ok_or_else(|| {
        QueryFailure::Unavailable("a block state has no registered identifier".to_owned())
    })
}

impl Query<World> for BlockAt {
    type Input = BlockPos;
    type Output = Identifier;

    fn id(&self) -> &Identifier {
        &self.id
    }

    fn answer(&self, world: &World, at: &BlockPos, _: Budget) -> Answer<Identifier> {
        let bounds = world.descriptor().bounds;
        if !bounds.contains_y(at.y) {
            return Err(QueryFailure::Invalid(format!(
                "y {} is outside the world",
                at.y
            )));
        }
        identifier_at(world, *at)
    }

    fn result_count(_: &Identifier) -> usize {
        1
    }
}

impl Query<World> for BlocksInRegion {
    type Input = (BlockPos, BlockPos);
    type Output = Vec<(BlockPos, Identifier)>;

    fn id(&self) -> &Identifier {
        &self.id
    }

    fn answer(
        &self,
        world: &World,
        (low, high): &(BlockPos, BlockPos),
        budget: Budget,
    ) -> Answer<Vec<(BlockPos, Identifier)>> {
        if low.x > high.x || low.y > high.y || low.z > high.z {
            return Err(QueryFailure::Invalid(
                "the region's low corner is above its high corner".to_owned(),
            ));
        }
        let span = |a: i64, b: i64| u64::try_from(b - a + 1).unwrap_or(u64::MAX);
        let cells = span(low.x, high.x)
            .saturating_mul(span(low.y, high.y))
            .saturating_mul(span(low.z, high.z));
        if cells > MAX_REGION_CELLS {
            return Err(QueryFailure::Invalid(format!(
                "the region holds {cells} cells; a query may scan {MAX_REGION_CELLS}"
            )));
        }
        let bounds = world.descriptor().bounds;
        let mut found = Vec::new();
        for y in low.y.max(bounds.min_y)..=high.y.min(bounds.max_y) {
            for z in low.z..=high.z {
                for x in low.x..=high.x {
                    let at = BlockPos::new(x, y, z);
                    let state = world.get_block(at).map_err(|_| {
                        QueryFailure::Unavailable(format!("{x},{y},{z} is not resident"))
                    })?;
                    if state == AIR {
                        continue;
                    }
                    if found.len() == budget.max_results {
                        return Err(QueryFailure::OverBudget {
                            limit: budget.max_results,
                        });
                    }
                    found.push((at, identifier_at(world, at)?));
                }
            }
        }
        Ok(found)
    }

    fn result_count(output: &Vec<(BlockPos, Identifier)>) -> usize {
        output.len()
    }
}

impl Query<World> for WorldTimeQuery {
    type Input = ();
    type Output = WorldTime;

    fn id(&self) -> &Identifier {
        &self.id
    }

    fn answer(&self, world: &World, (): &(), _: Budget) -> Answer<WorldTime> {
        Ok(world.clock().now())
    }

    fn result_count(_: &WorldTime) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_command::identity::Actor;
    use nexora_foundation::spatial::ChunkCoord;
    use nexora_foundation::time::CalendarConfig;
    use nexora_query::QueryRequest;
    use nexora_world::world::WorldDescriptor;

    fn world() -> World {
        let mut world = World::create(
            WorldDescriptor::new("queries", 11).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();
        world
    }

    fn request<I>(actor: &Actor, input: I) -> QueryRequest<'_, I> {
        QueryRequest {
            actor,
            source: Source::Network,
            version: WORLD_QUERY_VERSION,
            input,
        }
    }

    fn setup() -> (QueryService, WorldQueries) {
        let mut service = QueryService::new().unwrap();
        let queries = WorldQueries::register(&mut service).unwrap();
        service.freeze();
        (service, queries)
    }

    #[test]
    fn a_block_is_answered_by_its_identifier() {
        let (service, queries) = setup();
        let mut world = world();
        let stone = world
            .block_id(&Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        let at = BlockPos::new(3, 150, 3);
        world.set_block(at, stone).unwrap();

        let player = Actor::player(7);
        let answer = service.ask(&queries.block_at, &world, &request(&player, at));
        assert_eq!(answer.unwrap().to_string(), "nexora:block/stone");
        let air = service.ask(
            &queries.block_at,
            &world,
            &request(&player, BlockPos::new(3, 151, 3)),
        );
        assert_eq!(air.unwrap().to_string(), "nexora:block/air");
    }

    #[test]
    fn a_cell_that_is_not_resident_is_unavailable_never_air() {
        let (service, queries) = setup();
        let world = world();
        let far = BlockPos::new(10_000, 70, 10_000);
        let answer = service.ask(&queries.block_at, &world, &request(&Actor::server(), far));
        assert!(
            matches!(answer, Err(QueryFailure::Unavailable(_))),
            "{answer:?}"
        );
        let outside = BlockPos::new(0, world.descriptor().bounds.max_y + 1, 0);
        let answer = service.ask(
            &queries.block_at,
            &world,
            &request(&Actor::server(), outside),
        );
        assert!(
            matches!(answer, Err(QueryFailure::Invalid(_))),
            "{answer:?}"
        );
    }

    #[test]
    fn a_region_answers_in_a_stable_order_and_within_its_cost() {
        let (service, queries) = setup();
        let mut world = world();
        let dirt = world
            .block_id(&Identifier::parse("nexora:block/dirt").unwrap())
            .unwrap();
        for at in [
            BlockPos::new(2, 200, 1),
            BlockPos::new(1, 200, 1),
            BlockPos::new(1, 201, 0),
        ] {
            world.set_block(at, dirt).unwrap();
        }
        let player = Actor::player(1);
        let found = service
            .ask(
                &queries.blocks_in_region,
                &world,
                &request(
                    &player,
                    (BlockPos::new(0, 200, 0), BlockPos::new(3, 201, 3)),
                ),
            )
            .unwrap();
        let positions: Vec<BlockPos> = found.iter().map(|(at, _)| *at).collect();
        assert_eq!(
            positions,
            [
                BlockPos::new(1, 200, 1),
                BlockPos::new(2, 200, 1),
                BlockPos::new(1, 201, 0)
            ],
            "y, then z, then x"
        );
        assert!(found
            .iter()
            .all(|(_, id)| id.to_string() == "nexora:block/dirt"));

        // Too large to scan, whatever it holds.
        let huge = service.ask(
            &queries.blocks_in_region,
            &world,
            &request(&player, (BlockPos::new(0, 0, 0), BlockPos::new(63, 63, 63))),
        );
        assert!(
            matches!(huge, Err(QueryFailure::Invalid(ref why)) if why.contains("262144")),
            "{huge:?}"
        );

        // Inverted corners.
        let inverted = service.ask(
            &queries.blocks_in_region,
            &world,
            &request(&player, (BlockPos::new(3, 3, 3), BlockPos::new(0, 0, 0))),
        );
        assert!(matches!(inverted, Err(QueryFailure::Invalid(_))));
    }

    #[test]
    fn a_region_of_solid_rock_fills_its_budget_and_no_further() {
        let (service, queries) = setup();
        let mut world = world();
        for (x, z) in [(1, 0), (0, 1), (1, 1)] {
            world.load_or_generate(ChunkCoord::new(x, z)).unwrap();
        }
        // Deep underground the generator fills every cell with stone: a 32³
        // box is exactly the budget, and one cell more is refused by cost.
        let low = BlockPos::new(0, world.descriptor().bounds.min_y, 0);
        let high = BlockPos::new(31, low.y + 31, 31);
        let found = service
            .ask(
                &queries.blocks_in_region,
                &world,
                &request(&Actor::server(), (low, high)),
            )
            .expect("exactly the budget");
        assert_eq!(found.len(), REGION_MAX_RESULTS);
        let wider = BlockPos::new(high.x + 1, high.y, high.z);
        let refused = service.ask(
            &queries.blocks_in_region,
            &world,
            &request(&Actor::server(), (low, wider)),
        );
        assert!(
            matches!(refused, Err(QueryFailure::Invalid(_))),
            "{refused:?}"
        );
    }

    #[test]
    fn world_time_is_the_clock() {
        let (service, queries) = setup();
        let world = world();
        let now = service
            .ask(
                &queries.world_time,
                &world,
                &request(&Actor::of(ActorKind::Script), ()),
            )
            .unwrap();
        assert_eq!(now, world.clock().now());
    }
}
