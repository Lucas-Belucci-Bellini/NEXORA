//! The first visual generation, held to its own catalog.
//!
//! Regenerates everything `content/first-generation/manifest.json` lists, from
//! the checked-in definitions and recipes, and checks it against
//! `content/first-generation/CATALOG.md` byte for byte. A change to a recipe,
//! the renderer or the PNG encoder fails here, naming the asset, so the catalog
//! is updated on purpose instead of drifting away from what the forge makes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use nexora_asset::texture::{ChannelLayout, MapRole, Resolution};
use nexora_foundation::hashing::fnv1a64;
use nexora_texture_forge::batch::{Plan, Policy};
use nexora_texture_forge::forge::Forge;
use nexora_texture_forge::layout;
use nexora_texture_forge::recipe_book::RecipeBook;

fn content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "nexora-first-generation-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    root
}

/// `asset_id -> albedo hash`, read from the catalog's table.
fn catalogued() -> BTreeMap<String, u64> {
    let text = std::fs::read_to_string(content().join("first-generation/CATALOG.md"))
        .expect("the catalog is checked in");
    text.lines()
        .filter(|line| line.starts_with("| `"))
        .map(|line| {
            let cells: Vec<&str> = line
                .split('|')
                .map(str::trim)
                .filter(|cell| !cell.is_empty())
                .collect();
            let id = cells[0].trim_matches('`').to_owned();
            let hash = cells[cells.len() - 1].trim_matches('`');
            let hash = u64::from_str_radix(hash.trim_start_matches("0x"), 16)
                .unwrap_or_else(|_| panic!("{id}: `{hash}` is not a hash"));
            (id, hash)
        })
        .collect()
}

#[test]
fn the_first_generation_is_what_its_catalog_says_it_is() {
    let out = scratch("out");
    let plan =
        Plan::load(&content().join("first-generation/manifest.json")).expect("the manifest reads");
    assert!(plan.is_valid(), "{:?}", plan.problems);
    assert_eq!(
        plan.policy,
        Policy::first_generation(),
        "the first generation's manifest must carry the first generation's rule"
    );

    let forge = Forge::new(&out)
        .unwrap()
        .with_recipes(RecipeBook::at(content().join("recipes")));
    let report = plan.run(&forge, false).expect("the batch runs");
    assert!(report.succeeded(), "{report:?}");
    assert_eq!(report.tally().written, plan.entries.len());

    let catalog = catalogued();
    let listed: BTreeSet<String> = plan
        .entries
        .iter()
        .map(|entry| entry.definition.id().to_string())
        .collect();
    assert_eq!(
        catalog.keys().cloned().collect::<BTreeSet<_>>(),
        listed,
        "every generated asset is catalogued, and nothing is catalogued that is not generated"
    );

    let mut palettes: Vec<(String, BTreeSet<[u8; 4]>)> = Vec::new();
    // Collected, and checked last, so that a recipe change that also merges
    // two stones is reported as the merge rather than only as a new hash.
    let mut drifted: Vec<String> = Vec::new();
    for entry in &plan.entries {
        let id = entry.definition.id();
        // Every entry names a recipe: a first-generation asset drawn with its
        // category's default is the one-stone-many-seeds failure again.
        assert!(entry.definition.recipe().is_some(), "{id} names no recipe");

        let albedo = layout::map_file(&out, id, MapRole::Albedo);
        let bytes = std::fs::read(&albedo).unwrap();
        let hash = fnv1a64(&bytes);
        if hash != catalog[&id.to_string()] {
            drifted.push(format!("{id} is now {hash:#018x}"));
        }

        // Albedo alone: no other map file exists.
        for role in MapRole::ALL {
            assert_eq!(
                layout::map_file(&out, id, role).is_file(),
                role == MapRole::Albedo,
                "{id}: {} map",
                role.as_str()
            );
        }

        let decoded = nexora_texture_forge::decode_png(&bytes).unwrap();
        assert_eq!(decoded.resolution, Resolution::square(16).unwrap(), "{id}");
        assert_eq!(decoded.layout, ChannelLayout::Rgba, "{id}");
        let palette: BTreeSet<[u8; 4]> = decoded
            .pixels
            .chunks_exact(4)
            .map(|texel| {
                assert_eq!(
                    texel[3], 255,
                    "{id}: an opaque surface has no coverage holes"
                );
                [texel[0], texel[1], texel[2], texel[3]]
            })
            .collect();
        palettes.push((id.to_string(), palette));
    }

    // "Cada pedra visualmente distinguível", in the one way a test can hold
    // it: two members of a family may not draw from mostly the same colours.
    for (index, (a, left)) in palettes.iter().enumerate() {
        for (b, right) in &palettes[index + 1..] {
            let shared = left.intersection(right).count();
            let union = left.union(right).count();
            assert!(
                shared * 2 < union,
                "{a} and {b} share {shared} of {union} colours; they are one stone twice"
            );
        }
    }

    assert!(
        drifted.is_empty(),
        "the albedo on disk is not the one CATALOG.md records; \
         if the change is intended, update the catalog:\n{}",
        drifted.join("\n")
    );

    // And a second pass changes nothing.
    let again = plan.run(&forge, false).unwrap();
    assert_eq!(again.tally().unchanged, plan.entries.len());

    let _ = std::fs::remove_dir_all(&out);
}
