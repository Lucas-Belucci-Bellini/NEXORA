//! What a surface material *is*, before anyone draws it.
//!
//! A material is treated as an entity rather than as an image, which is the
//! central demand of the Texture Forge brief §2: it has identity, a category, a
//! physical scale, PBR parameters, a revision and an origin. The pixels are
//! deliberately **not** here.
//!
//! # Why the definition carries no pixels
//!
//! `RESOURCE AND ASSET SYSTEM.md` describes the runtime path as
//! `ResourceID → Manifest → Resolver → Loader → Cache → Runtime Handle`: the
//! thing that is registered is an identity, and the bytes arrive later, through
//! a cache that can evict them. A registry holding pixels would make that
//! impossible — freezing the material registry would pin every texture in
//! memory forever. So [`SurfaceMaterial`] is small, cheap to clone and
//! registrable, and `crate::generator::GeneratedMaterial` is what carries maps.
//!
//! # Why this is not called `Material`
//!
//! `nexora_physics::material::MaterialId` already means friction, restitution,
//! density and drag. `NEXORA NAMING AND TERMINOLOGY.md` forbids one name
//! covering two concepts. A block may have both; they never meet.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::Fnv1a64;
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::MaterialSchemaVersion;

use crate::provenance::Provenance;
use crate::texture::{MapRole, Resolution};

/// Schema version this build writes and reads.
pub const MATERIAL_SCHEMA_VERSION: MaterialSchemaVersion = MaterialSchemaVersion(1);

/// Path prefix a material identifier conventionally uses.
pub const MATERIAL_PATH_PREFIX: &str = "material/";

/// Path prefix the derived texture asset identifiers use.
pub const TEXTURE_PATH_PREFIX: &str = "texture/";

/// Which family a material belongs to.
///
/// The list from the Texture Forge brief §11. Classification only: what stone
/// *looks like* is a preset, and presets live in the tool, because a crate the
/// runtime links has no business holding art direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialCategory {
    /// Rock, in all its forms.
    Stone,
    /// Cut timber and bark.
    Wood,
    /// Worked metal.
    Metal,
    /// Earth and loam.
    Soil,
    /// Loose granular mineral.
    Sand,
    /// Cast aggregate.
    Concrete,
    /// Fired clay masonry.
    Brick,
    /// Transparent amorphous solid.
    Glass,
    /// Fired clay and porcelain.
    Ceramic,
    /// Synthetic polymer.
    Plastic,
    /// Woven or felted textile.
    Fabric,
    /// Living or once-living tissue.
    Organic,
    /// Crystalline ore and gem.
    Mineral,
    /// Leaf, stem and foliage.
    Vegetation,
    /// Anything the list above does not describe.
    Custom,
}

impl MaterialCategory {
    /// Every category, in a stable order.
    pub const ALL: [Self; 15] = [
        Self::Stone,
        Self::Wood,
        Self::Metal,
        Self::Soil,
        Self::Sand,
        Self::Concrete,
        Self::Brick,
        Self::Glass,
        Self::Ceramic,
        Self::Plastic,
        Self::Fabric,
        Self::Organic,
        Self::Mineral,
        Self::Vegetation,
        Self::Custom,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stone => "stone",
            Self::Wood => "wood",
            Self::Metal => "metal",
            Self::Soil => "soil",
            Self::Sand => "sand",
            Self::Concrete => "concrete",
            Self::Brick => "brick",
            Self::Glass => "glass",
            Self::Ceramic => "ceramic",
            Self::Plastic => "plastic",
            Self::Fabric => "fabric",
            Self::Organic => "organic",
            Self::Mineral => "mineral",
            Self::Vegetation => "vegetation",
            Self::Custom => "custom",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not a known category. Unknown does not
    /// fall back to [`Self::Custom`]: a typo that silently becomes "custom" is
    /// a material that quietly loses its family.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == raw)
            .ok_or_else(|| {
                invalid("material category is not recognised").with_context("value", raw.to_owned())
            })
    }
}

/// How a surface is composited.
///
/// The list from `RENDERER and GRAPHICS.md` RENDER-13. This is the field
/// DEBT-0026 records as missing: the mesher today assumes every block occludes
/// what is behind it because nothing in the engine can say otherwise. A
/// material that declares itself transparent is what lets that assumption go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    /// Fully hides what is behind it.
    Opaque,
    /// Fully opaque or fully absent per texel, decided by an alpha threshold.
    Cutout,
    /// Blended against what is behind it.
    Transparent,
    /// Emits light of its own.
    Emissive,
    /// Transmits light diffusely.
    Translucent,
}

impl BlendMode {
    /// Every mode, in a stable order.
    pub const ALL: [Self; 5] = [
        Self::Opaque,
        Self::Cutout,
        Self::Transparent,
        Self::Emissive,
        Self::Translucent,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::Cutout => "cutout",
            Self::Transparent => "transparent",
            Self::Emissive => "emissive",
            Self::Translucent => "translucent",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not a known mode.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|mode| mode.as_str() == raw)
            .ok_or_else(|| {
                invalid("blend mode is not recognised").with_context("value", raw.to_owned())
            })
    }

    /// Whether a face of this material hides the face behind it.
    ///
    /// The question `nexora_mesh::view::VoxelView::occludes` asks. Only an
    /// opaque surface may answer yes: a cutout surface has holes, and the other
    /// three are seen through by definition.
    #[must_use]
    pub const fn occludes(self) -> bool {
        matches!(self, Self::Opaque)
    }
}

/// How large one tile of the material is in the world.
///
/// A texture with no declared scale tiles at whatever size the mesh happens to
/// give it, which is how a brick wall ends up with bricks the size of a door.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalScale {
    /// Metres covered by one tile of the texture.
    pub metres_per_tile: f64,
}

impl PhysicalScale {
    /// One tile per block, the voxel default.
    pub const PER_BLOCK: Self = Self {
        metres_per_tile: 1.0,
    };

    /// A scale in metres per tile.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not finite or not positive.
    pub fn new(metres_per_tile: f64) -> Result<Self> {
        if !metres_per_tile.is_finite() || metres_per_tile <= 0.0 {
            return Err(invalid("physical scale must be finite and positive")
                .with_context("metres_per_tile", metres_per_tile.to_string()));
        }
        Ok(Self { metres_per_tile })
    }
}

/// The physically based parameters a shader reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PbrParameters {
    /// Conductor fraction, `0.0` dielectric to `1.0` metal.
    pub metallic: f64,
    /// Microfacet roughness, `0.0` mirror to `1.0` fully diffuse.
    pub roughness: f64,
    /// Multiplier on the normal map's deviation from flat.
    pub normal_strength: f64,
    /// Multiplier on the height map's displacement.
    pub height_strength: f64,
}

/// Largest multiplier accepted for normal and height strength.
///
/// Past this the surface reads as noise rather than as relief; the ceiling
/// exists so a mistyped `40` fails instead of shipping.
pub const MAX_STRENGTH: f64 = 8.0;

impl PbrParameters {
    /// A neutral dielectric: fully rough, no relief exaggeration.
    pub const DEFAULT: Self = Self {
        metallic: 0.0,
        roughness: 1.0,
        normal_strength: 1.0,
        height_strength: 1.0,
    };

    /// Check every coefficient against its range.
    ///
    /// # Errors
    ///
    /// Returns an error naming the coefficient, its value and its range.
    pub fn validate(self) -> Result<Self> {
        let checks: [(&'static str, f64, f64); 4] = [
            ("metallic", self.metallic, 1.0),
            ("roughness", self.roughness, 1.0),
            ("normal_strength", self.normal_strength, MAX_STRENGTH),
            ("height_strength", self.height_strength, MAX_STRENGTH),
        ];
        for (name, value, limit) in checks {
            if !value.is_finite() || value < 0.0 || value > limit {
                return Err(invalid("PBR coefficient is outside its range")
                    .with_context("coefficient", name)
                    .with_context("value", value.to_string())
                    .with_context("range", format!("0.0..={limit}")));
            }
        }
        Ok(self)
    }
}

/// A material's revision number.
///
/// The brief §15 asks for `v1`, `v2`, `v3` and for changes to be traceable.
/// Distinct from [`MaterialSchemaVersion`], which versions the *document
/// layout*: a material can reach revision nine without the schema ever moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(pub u32);

impl Revision {
    /// The first revision of any material.
    pub const FIRST: Self = Self(1);

    /// The next revision.
    ///
    /// # Errors
    ///
    /// Returns an error when the counter would overflow.
    pub fn next(self) -> Result<Self> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| invalid("revision counter is exhausted"))
    }
}

impl core::fmt::Display for Revision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// The definition of one surface material.
///
/// Built through [`SurfaceMaterial::builder`], which validates on the way out;
/// there is no way to hold an invalid one.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceMaterial {
    id: Identifier,
    name: String,
    category: MaterialCategory,
    schema: MaterialSchemaVersion,
    revision: Revision,
    resolution: Resolution,
    physical_scale: PhysicalScale,
    seamless: bool,
    blend: BlendMode,
    pbr: PbrParameters,
    wanted_maps: Vec<MapRole>,
    provenance: Provenance,
}

impl SurfaceMaterial {
    /// Start building a material.
    #[must_use]
    pub fn builder(
        id: Identifier,
        category: MaterialCategory,
        resolution: Resolution,
        provenance: Provenance,
    ) -> SurfaceMaterialBuilder {
        SurfaceMaterialBuilder {
            name: None,
            id,
            category,
            revision: Revision::FIRST,
            resolution,
            physical_scale: PhysicalScale::PER_BLOCK,
            seamless: true,
            blend: BlendMode::Opaque,
            pbr: PbrParameters::DEFAULT,
            wanted_maps: Vec::new(),
            provenance,
        }
    }

    /// The stable identifier, e.g. `nexora:material/stone_rough`.
    #[must_use]
    pub const fn id(&self) -> &Identifier {
        &self.id
    }

    /// The human-readable name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The family this material belongs to.
    #[must_use]
    pub const fn category(&self) -> MaterialCategory {
        self.category
    }

    /// The document schema this material was written against.
    #[must_use]
    pub const fn schema(&self) -> MaterialSchemaVersion {
        self.schema
    }

    /// Which revision of this material it is.
    #[must_use]
    pub const fn revision(&self) -> Revision {
        self.revision
    }

    /// The dimensions its maps are generated at.
    #[must_use]
    pub const fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// How large one tile is in the world.
    #[must_use]
    pub const fn physical_scale(&self) -> PhysicalScale {
        self.physical_scale
    }

    /// Whether the material is meant to tile without a visible seam.
    #[must_use]
    pub const fn is_seamless(&self) -> bool {
        self.seamless
    }

    /// How the surface is composited.
    #[must_use]
    pub const fn blend(&self) -> BlendMode {
        self.blend
    }

    /// The shader parameters.
    #[must_use]
    pub const fn pbr(&self) -> PbrParameters {
        self.pbr
    }

    /// Which optional maps this material wants, beyond the required ones.
    #[must_use]
    pub fn wanted_maps(&self) -> &[MapRole] {
        &self.wanted_maps
    }

    /// Where it came from.
    #[must_use]
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Mutable access to the origin record, for a reviewer marking clearance.
    pub fn provenance_mut(&mut self) -> &mut Provenance {
        &mut self.provenance
    }

    /// The asset identifier of one of this material's maps.
    ///
    /// Derived, never stored: a texture path written by hand is a path that
    /// drifts from the material it belongs to. `nexora:material/stone` yields
    /// `nexora:texture/stone/albedo`; a material whose path does not start with
    /// `material/` keeps its path whole, so the mapping is still one-to-one.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived path is too long for an identifier.
    pub fn map_asset_id(&self, role: MapRole) -> Result<Identifier> {
        let stem = self
            .id
            .path()
            .strip_prefix(MATERIAL_PATH_PREFIX)
            .unwrap_or_else(|| self.id.path());
        Identifier::new(
            self.id.namespace().clone(),
            &format!("{TEXTURE_PATH_PREFIX}{stem}/{}", role.as_str()),
        )
    }

    /// A revision of this material with a fresh origin record.
    ///
    /// The brief §15 forbids silently overwriting a material. Producing the
    /// next revision is therefore an operation that returns a *new* value: the
    /// previous one is still whatever it was.
    ///
    /// # Errors
    ///
    /// Returns an error when the revision counter is exhausted.
    pub fn revised(&self, provenance: Provenance) -> Result<Self> {
        Ok(Self {
            revision: self.revision.next()?,
            provenance,
            ..self.clone()
        })
    }

    /// A hash over every field that decides what the material looks like.
    ///
    /// Excludes the revision and the origin record: two revisions that describe
    /// the same surface should hash the same, because that is how a "repair"
    /// can tell it changed nothing.
    #[must_use]
    pub fn appearance_hash(&self) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str(&self.id.to_string());
        hasher.write_str(self.category.as_str());
        hasher.write_str(self.blend.as_str());
        hasher.write_u64(u64::from(self.resolution.width));
        hasher.write_u64(u64::from(self.resolution.height));
        hasher.write_u64(self.physical_scale.metres_per_tile.to_bits());
        hasher.write_u64(u64::from(self.seamless));
        hasher.write_u64(self.pbr.metallic.to_bits());
        hasher.write_u64(self.pbr.roughness.to_bits());
        hasher.write_u64(self.pbr.normal_strength.to_bits());
        hasher.write_u64(self.pbr.height_strength.to_bits());
        for role in &self.wanted_maps {
            hasher.write_str(role.as_str());
        }
        hasher.finish()
    }
}

/// Builds a [`SurfaceMaterial`], validating at the end.
#[derive(Debug, Clone)]
pub struct SurfaceMaterialBuilder {
    id: Identifier,
    name: Option<String>,
    category: MaterialCategory,
    revision: Revision,
    resolution: Resolution,
    physical_scale: PhysicalScale,
    seamless: bool,
    blend: BlendMode,
    pbr: PbrParameters,
    wanted_maps: Vec<MapRole>,
    provenance: Provenance,
}

impl SurfaceMaterialBuilder {
    /// Set the human-readable name. Defaults to the identifier's path.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the revision. Defaults to [`Revision::FIRST`].
    #[must_use]
    pub const fn revision(mut self, revision: Revision) -> Self {
        self.revision = revision;
        self
    }

    /// Set the world scale. Defaults to one tile per block.
    #[must_use]
    pub const fn scaled(mut self, scale: PhysicalScale) -> Self {
        self.physical_scale = scale;
        self
    }

    /// Declare whether the material must tile seamlessly. Defaults to `true`.
    #[must_use]
    pub const fn seamless(mut self, seamless: bool) -> Self {
        self.seamless = seamless;
        self
    }

    /// Set the blend mode. Defaults to [`BlendMode::Opaque`].
    #[must_use]
    pub const fn blended(mut self, blend: BlendMode) -> Self {
        self.blend = blend;
        self
    }

    /// Set the PBR parameters. Defaults to [`PbrParameters::DEFAULT`].
    #[must_use]
    pub const fn pbr(mut self, pbr: PbrParameters) -> Self {
        self.pbr = pbr;
        self
    }

    /// Ask for an optional map.
    #[must_use]
    pub fn wants(mut self, role: MapRole) -> Self {
        if !self.wanted_maps.contains(&role) {
            self.wanted_maps.push(role);
        }
        self
    }

    /// Finish, validating everything.
    ///
    /// # Errors
    ///
    /// Returns an error when the PBR coefficients are out of range, the origin
    /// record breaks a registry rule, or the derived texture identifiers would
    /// be invalid. Validation happens here rather than at use so that an
    /// invalid material cannot be held at all.
    pub fn build(self) -> Result<SurfaceMaterial> {
        let pbr = self.pbr.validate()?;
        self.provenance.validate()?;

        let name = self.name.unwrap_or_else(|| {
            // From the stem, matching `map_asset_id`: a material called
            // "material dark oak plank" reads like a bug, because it is one.
            self.id
                .path()
                .strip_prefix(MATERIAL_PATH_PREFIX)
                .unwrap_or_else(|| self.id.path())
                .replace(['/', '_'], " ")
        });
        if name.trim().is_empty() {
            return Err(invalid("a material needs a name"));
        }

        let mut wanted_maps = self.wanted_maps;
        wanted_maps.sort_unstable();
        wanted_maps.dedup();

        let material = SurfaceMaterial {
            id: self.id,
            name,
            category: self.category,
            schema: MATERIAL_SCHEMA_VERSION,
            revision: self.revision,
            resolution: self.resolution,
            physical_scale: self.physical_scale,
            seamless: self.seamless,
            blend: self.blend,
            pbr,
            wanted_maps,
            provenance: self.provenance,
        };

        // Prove every derived identifier is constructible now, rather than
        // discovering at write time that one map cannot be named.
        for role in MapRole::ALL {
            material.map_asset_id(role)?;
        }
        Ok(material)
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "material", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{GenerationTrace, Provenance};
    use nexora_foundation::version::ContentGeneratorVersion;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn provenance() -> Provenance {
        Provenance::generated(
            "NEXORA",
            "nexora-texture-forge@0.0.1",
            GenerationTrace::new(
                id("nexora:generator/procedural"),
                ContentGeneratorVersion(1),
                7,
            ),
        )
    }

    fn material() -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id("nexora:material/stone_rough"),
            MaterialCategory::Stone,
            Resolution::square(64).unwrap(),
            provenance(),
        )
        .named("Rough Stone")
        .wants(MapRole::Roughness)
        .build()
        .expect("a well-formed material")
    }

    #[test]
    fn a_material_carries_identity_not_pixels() {
        let material = material();
        assert_eq!(material.id().to_string(), "nexora:material/stone_rough");
        assert_eq!(material.name(), "Rough Stone");
        assert_eq!(material.category(), MaterialCategory::Stone);
        assert_eq!(material.revision(), Revision::FIRST);
        assert_eq!(material.schema(), MATERIAL_SCHEMA_VERSION);
        assert!(material.is_seamless());
        assert_eq!(material.blend(), BlendMode::Opaque);
    }

    #[test]
    fn texture_identifiers_are_derived_from_the_material_not_written_by_hand() {
        let material = material();
        assert_eq!(
            material.map_asset_id(MapRole::Albedo).unwrap().to_string(),
            "nexora:texture/stone_rough/albedo"
        );
        assert_eq!(
            material
                .map_asset_id(MapRole::AmbientOcclusion)
                .unwrap()
                .to_string(),
            "nexora:texture/stone_rough/ambient_occlusion"
        );

        // A material whose path does not use the conventional prefix keeps it,
        // so the mapping stays one-to-one.
        let odd = SurfaceMaterial::builder(
            id("example:custom/thing"),
            MaterialCategory::Custom,
            Resolution::square(16).unwrap(),
            provenance(),
        )
        .build()
        .unwrap();
        assert_eq!(
            odd.map_asset_id(MapRole::Albedo).unwrap().to_string(),
            "example:texture/custom/thing/albedo"
        );
    }

    #[test]
    fn out_of_range_pbr_values_never_reach_a_material() {
        for bad in [
            PbrParameters {
                metallic: 1.5,
                ..PbrParameters::DEFAULT
            },
            PbrParameters {
                roughness: -0.1,
                ..PbrParameters::DEFAULT
            },
            PbrParameters {
                normal_strength: f64::NAN,
                ..PbrParameters::DEFAULT
            },
            PbrParameters {
                height_strength: MAX_STRENGTH + 1.0,
                ..PbrParameters::DEFAULT
            },
        ] {
            let err = SurfaceMaterial::builder(
                id("nexora:material/bad"),
                MaterialCategory::Stone,
                Resolution::square(16).unwrap(),
                provenance(),
            )
            .pbr(bad)
            .build()
            .expect_err("out-of-range coefficients must be refused");
            assert_eq!(err.recovery(), Recovery::Reject);
        }
        PbrParameters::DEFAULT
            .validate()
            .expect("the default is valid");
    }

    #[test]
    fn an_invalid_origin_record_stops_the_material_being_built() {
        let mut broken = provenance();
        broken.generation = None;
        let err = SurfaceMaterial::builder(
            id("nexora:material/stone"),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            broken,
        )
        .build()
        .expect_err("provenance is not optional");
        assert!(err.to_string().contains("reproduces"), "{err}");
    }

    #[test]
    fn revising_produces_a_new_material_and_leaves_the_old_one_alone() {
        let first = material();
        let second = first.revised(provenance()).expect("revision two");

        assert_eq!(first.revision(), Revision(1));
        assert_eq!(second.revision(), Revision(2));
        assert_eq!(second.revision().to_string(), "v2");
        // The surface itself did not change, and the hash says so.
        assert_eq!(first.appearance_hash(), second.appearance_hash());
        assert!(Revision(u32::MAX).next().is_err());
    }

    #[test]
    fn the_appearance_hash_moves_when_the_surface_does() {
        let base = material();
        let rougher = SurfaceMaterial::builder(
            id("nexora:material/stone_rough"),
            MaterialCategory::Stone,
            Resolution::square(64).unwrap(),
            provenance(),
        )
        .named("Rough Stone")
        .wants(MapRole::Roughness)
        .pbr(PbrParameters {
            roughness: 0.4,
            ..PbrParameters::DEFAULT
        })
        .build()
        .unwrap();
        assert_ne!(base.appearance_hash(), rougher.appearance_hash());

        let bigger = SurfaceMaterial::builder(
            id("nexora:material/stone_rough"),
            MaterialCategory::Stone,
            Resolution::square(128).unwrap(),
            provenance(),
        )
        .named("Rough Stone")
        .wants(MapRole::Roughness)
        .build()
        .unwrap();
        assert_ne!(base.appearance_hash(), bigger.appearance_hash());
    }

    #[test]
    fn only_opaque_surfaces_hide_what_is_behind_them() {
        assert!(BlendMode::Opaque.occludes());
        for mode in BlendMode::ALL {
            if mode != BlendMode::Opaque {
                assert!(!mode.occludes(), "{mode:?} must not occlude");
            }
        }
    }

    #[test]
    fn category_and_mode_names_round_trip_and_typos_are_refused() {
        for category in MaterialCategory::ALL {
            assert_eq!(
                MaterialCategory::parse(category.as_str()).unwrap(),
                category
            );
        }
        for mode in BlendMode::ALL {
            assert_eq!(BlendMode::parse(mode.as_str()).unwrap(), mode);
        }
        // A typo must not quietly become `custom`.
        assert!(MaterialCategory::parse("stoen").is_err());
        assert!(BlendMode::parse("opague").is_err());
    }

    #[test]
    fn wanted_maps_are_sorted_and_deduplicated() {
        let material = SurfaceMaterial::builder(
            id("nexora:material/x"),
            MaterialCategory::Custom,
            Resolution::square(16).unwrap(),
            provenance(),
        )
        .wants(MapRole::Height)
        .wants(MapRole::Roughness)
        .wants(MapRole::Height)
        .build()
        .unwrap();
        assert_eq!(
            material.wanted_maps(),
            [MapRole::Roughness, MapRole::Height]
        );
    }

    #[test]
    fn a_scale_must_be_finite_and_positive() {
        assert!(PhysicalScale::new(0.25).is_ok());
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(PhysicalScale::new(bad).is_err(), "{bad} must be refused");
        }
        let scaled = SurfaceMaterial::builder(
            id("nexora:material/brick"),
            MaterialCategory::Brick,
            Resolution::square(64).unwrap(),
            provenance(),
        )
        .scaled(PhysicalScale::new(2.0).unwrap())
        .build()
        .unwrap();
        assert!((scaled.physical_scale().metres_per_tile - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_name_is_derived_from_the_identifier_when_none_is_given() {
        let material = SurfaceMaterial::builder(
            id("nexora:material/dark_oak_plank"),
            MaterialCategory::Wood,
            Resolution::square(32).unwrap(),
            provenance(),
        )
        .build()
        .unwrap();
        assert_eq!(material.name(), "dark oak plank");
    }
}
