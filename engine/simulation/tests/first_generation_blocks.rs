//! The first visual generation, inside a world.
//!
//! `content/first-generation/blocks.json` adds the sixteen stones of issue #5
//! as blocks. This loads it the way any content would be loaded, builds a world
//! with it, and checks that every stone reaches the mesher through its own
//! surface — the `integration_status` the catalog records.

use std::collections::BTreeSet;
use std::path::Path;

use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::CalendarConfig;
use nexora_mesh::view::VoxelView;
use nexora_simulation::{BlockContent, WorldSurfaces, UNMAPPED_SURFACE};
use nexora_world::persist;
use nexora_world::world::{World, WorldDescriptor};

fn document() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/first-generation/blocks.json")
}

#[test]
fn every_first_generation_stone_is_a_block_with_its_own_surface() {
    let content = BlockContent::load(&document()).expect("the document loads");
    assert_eq!(content.blocks().len(), 16, "the sixteen stones of issue #5");

    // The policy the catalog promises, checked on the definitions the game
    // actually registers rather than only on the files the forge wrote.
    for material in content.materials() {
        assert_eq!(material.resolution().width, 16, "{}", material.id());
        assert_eq!(material.resolution().height, 16, "{}", material.id());
        assert!(material.wanted_maps().is_empty(), "{}", material.id());
        assert!(material.recipe().is_some(), "{}", material.id());
    }

    let materials = content
        .material_registry()
        .expect("every material registers");
    let mut world = World::create_with(
        WorldDescriptor::new("first-generation", 5).unwrap(),
        CalendarConfig::earthlike(),
        &content.block_definitions(),
    )
    .expect("the blocks register");
    world.bring_online().unwrap();
    world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();

    let table = content
        .surface_table(&world, &materials)
        .expect("every block resolves");

    // Place every stone in a row above the terrain, and read it back through
    // the view the mesher uses.
    let mut placed = Vec::new();
    for (index, block) in content.blocks().iter().enumerate() {
        let state = world.block_id(&block.id).unwrap();
        let at = BlockPos::new(index as i64 % 16, 200, 0);
        world.set_block(at, state).unwrap();
        placed.push((block.id.clone(), at));
    }
    let view = WorldSurfaces::new(&world, table);
    let mut surfaces = BTreeSet::new();
    for (block, at) in &placed {
        let surface = view.surface_at(*at).expect("a stone shows something");
        assert_ne!(surface, UNMAPPED_SURFACE, "{block} has no material");
        assert!(view.occludes(*at), "{block} is opaque stone");
        surfaces.insert(surface);
    }
    assert_eq!(surfaces.len(), 16, "sixteen stones, sixteen surfaces");

    // A world holding them saves and loads with the content present, and a
    // build without it refuses the save by name rather than loading holes.
    let container = persist::save(&world).unwrap();
    let definitions = content.block_definitions();
    let restored = persist::load_with(&container, &definitions).expect("loads with its content");
    for (block, at) in &placed {
        assert_eq!(
            restored.get_block(*at).unwrap(),
            restored.block_id(block).unwrap(),
            "{block}"
        );
    }
    let err = persist::load(&container).expect_err("no content, no stones");
    assert!(err.to_string().contains("nexora:block/stone/"), "{err}");
}
