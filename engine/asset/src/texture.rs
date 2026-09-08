//! Texture maps: what they hold, how big they are, and whether they are there.
//!
//! Implements the map list from the Texture Forge brief §6 and the texture
//! concerns of `RENDERER and GRAPHICS.md` RENDER-14. Two rules from the brief
//! shape the whole module:
//!
//! * *"Not every material needs every map."* So a material carries a set, and
//!   which members are mandatory is a property of the category, not of the type.
//! * *"The system must report per-map status."* So absence is a value with a
//!   name ([`MapStatus::Missing`]) rather than a `None` the caller must
//!   interpret.
//!
//! # Colour space is part of the contract, not a renderer detail
//!
//! Albedo is authored in sRGB and everything else is linear. Getting that wrong
//! produces a texture that looks *almost* right, which is the worst failure
//! mode available: it survives review and shows up as flat lighting much later.
//! So the space is a property of the role, decided once, here.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Smallest edge a texture may have.
///
/// Voxel art lives at 16 and 32 far more often than at 4096; the floor exists
/// to catch a zero or a typo, not to impose a style.
pub const MIN_EDGE: u32 = 4;

/// Largest edge a texture may have.
///
/// The top of the ladder the brief asks for (512 · 1024 · 2048 · 4096 · 8192).
pub const MAX_EDGE: u32 = 8192;

/// Largest single map the encoder will hold in memory, in bytes.
///
/// 8192 × 8192 RGBA at 8 bits is 256 MiB, and a full PBR set is six of those.
/// The limit is stated so that asking for one fails with an explanation rather
/// than with an allocation failure.
pub const MAX_MAP_BYTES: u64 = 256 * 1024 * 1024;

/// How a channel's values should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSpace {
    /// Perceptual encoding. Albedo only.
    Srgb,
    /// Values that mean what they say. Everything a shader does arithmetic on.
    Linear,
}

impl ColorSpace {
    /// Stable lowercase name, for documents and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Srgb => "srgb",
            Self::Linear => "linear",
        }
    }
}

/// How many channels a map stores, and what they mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelLayout {
    /// One channel.
    Grey,
    /// One channel plus coverage.
    GreyAlpha,
    /// Three colour channels.
    Rgb,
    /// Three colour channels plus coverage.
    Rgba,
}

impl ChannelLayout {
    /// How many values per texel.
    #[must_use]
    pub const fn count(self) -> u32 {
        match self {
            Self::Grey => 1,
            Self::GreyAlpha => 2,
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }

    /// Whether the layout carries coverage.
    #[must_use]
    pub const fn has_alpha(self) -> bool {
        matches!(self, Self::GreyAlpha | Self::Rgba)
    }

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grey => "grey",
            Self::GreyAlpha => "grey_alpha",
            Self::Rgb => "rgb",
            Self::Rgba => "rgba",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not one of the four.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "grey" => Ok(Self::Grey),
            "grey_alpha" => Ok(Self::GreyAlpha),
            "rgb" => Ok(Self::Rgb),
            "rgba" => Ok(Self::Rgba),
            other => {
                Err(invalid("channel layout is not recognised")
                    .with_context("value", other.to_owned()))
            }
        }
    }
}

/// What one map describes.
///
/// The six PBR maps from the brief §6. A preview image is deliberately not one
/// of these: it is a picture *of* a material for a human, not an input to
/// lighting, and giving it a role here would let it be mistaken for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapRole {
    /// Base colour, in sRGB.
    Albedo,
    /// Tangent-space surface normals, packed into RGB.
    Normal,
    /// Microfacet roughness, `0` mirror to `1` fully diffuse.
    Roughness,
    /// Whether the surface is a conductor, `0` dielectric to `1` metal.
    Metallic,
    /// Displacement, for parallax and tessellation.
    Height,
    /// Baked self-occlusion.
    AmbientOcclusion,
}

impl MapRole {
    /// Every role, in a stable order.
    pub const ALL: [Self; 6] = [
        Self::Albedo,
        Self::Normal,
        Self::Roughness,
        Self::Metallic,
        Self::Height,
        Self::AmbientOcclusion,
    ];

    /// Stable lowercase name, used in file names and documents.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Albedo => "albedo",
            Self::Normal => "normal",
            Self::Roughness => "roughness",
            Self::Metallic => "metallic",
            Self::Height => "height",
            Self::AmbientOcclusion => "ambient_occlusion",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not a known role.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|role| role.as_str() == raw)
            .ok_or_else(|| {
                invalid("map role is not recognised").with_context("value", raw.to_owned())
            })
    }

    /// The space this role's values live in.
    #[must_use]
    pub const fn color_space(self) -> ColorSpace {
        match self {
            // The only perceptual map. Everything else is arithmetic.
            Self::Albedo => ColorSpace::Srgb,
            _ => ColorSpace::Linear,
        }
    }

    /// The layout this role is normally authored in.
    #[must_use]
    pub const fn natural_layout(self) -> ChannelLayout {
        match self {
            Self::Albedo => ChannelLayout::Rgba,
            Self::Normal => ChannelLayout::Rgb,
            _ => ChannelLayout::Grey,
        }
    }

    /// Whether a material is incomplete without this map.
    ///
    /// Only albedo. A surface with no colour is not a surface; a surface with
    /// no height map is simply flat, which is a legitimate thing to be.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::Albedo)
    }
}

/// Where one map stands within a material.
///
/// The five values the brief asks to be reportable. They answer two questions
/// at once — is this map wanted, and is it here — because that is the pair a
/// caller actually needs: `Required` means wanted and absent, `Generated` means
/// wanted and present, and `Missing` means neither wanted nor present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapStatus {
    /// The material needs this map and does not have it.
    Required,
    /// The material could use this map and does not have it.
    Optional,
    /// Present, well-formed and consistent with the material.
    Generated,
    /// Not present and not expected.
    Missing,
    /// Present but wrong — bad size, bad channel count, bad values.
    Invalid,
}

impl MapStatus {
    /// Whether this status should stop a material from being registered.
    #[must_use]
    pub const fn is_blocking(self) -> bool {
        matches!(self, Self::Required | Self::Invalid)
    }

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::Generated => "generated",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        }
    }
}

/// A texture's pixel dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Resolution {
    /// Texels across.
    pub width: u32,
    /// Texels down.
    pub height: u32,
}

impl Resolution {
    /// A square texture.
    ///
    /// # Errors
    ///
    /// Returns an error when the edge is out of range or not a power of two.
    pub fn square(edge: u32) -> Result<Self> {
        Self::new(edge, edge)
    }

    /// A texture of arbitrary aspect.
    ///
    /// # Errors
    ///
    /// Returns an error when either edge is outside `MIN_EDGE..=MAX_EDGE` or is
    /// not a power of two. Powers of two are demanded because mipmaps and the
    /// atlas that RENDER-14 and RENDER-15 describe both assume them, and a
    /// texture that cannot be mipmapped is a texture that shimmers at distance.
    pub fn new(width: u32, height: u32) -> Result<Self> {
        for (name, edge) in [("width", width), ("height", height)] {
            if !(MIN_EDGE..=MAX_EDGE).contains(&edge) {
                return Err(invalid("texture edge is out of range")
                    .with_context("axis", name)
                    .with_context("value", edge.to_string())
                    .with_context("range", format!("{MIN_EDGE}..={MAX_EDGE}")));
            }
            if !edge.is_power_of_two() {
                return Err(invalid("texture edge must be a power of two")
                    .with_context("axis", name)
                    .with_context("value", edge.to_string()));
            }
        }
        Ok(Self { width, height })
    }

    /// How many texels the texture holds.
    #[must_use]
    pub const fn texels(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Whether both edges are equal.
    #[must_use]
    pub const fn is_square(self) -> bool {
        self.width == self.height
    }
}

/// A pixel format: how many channels, at what depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureFormat {
    /// Which channels are stored.
    pub channels: ChannelLayout,
    /// Bits per channel. Eight or sixteen.
    pub bits_per_channel: u8,
}

impl TextureFormat {
    /// Eight bits per channel, the common case.
    #[must_use]
    pub const fn eight_bit(channels: ChannelLayout) -> Self {
        Self {
            channels,
            bits_per_channel: 8,
        }
    }

    /// Sixteen bits per channel, for maps where banding shows.
    #[must_use]
    pub const fn sixteen_bit(channels: ChannelLayout) -> Self {
        Self {
            channels,
            bits_per_channel: 16,
        }
    }

    /// Bytes per texel.
    ///
    /// # Errors
    ///
    /// Returns an error when the depth is neither eight nor sixteen. PNG
    /// defines others; the forge writes these two, and claiming support for a
    /// depth it cannot encode would be a lie the caller finds out about late.
    pub fn bytes_per_texel(self) -> Result<u64> {
        match self.bits_per_channel {
            8 => Ok(u64::from(self.channels.count())),
            16 => Ok(u64::from(self.channels.count()) * 2),
            other => Err(invalid("unsupported bit depth")
                .with_context("bits", other.to_string())
                .with_context("supported", "8, 16")),
        }
    }
}

/// One texture map: a role, a format, a size and the pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureMap {
    role: MapRole,
    format: TextureFormat,
    resolution: Resolution,
    pixels: Vec<u8>,
}

impl TextureMap {
    /// Wrap pixel data, checking that it is the size it claims to be.
    ///
    /// # Errors
    ///
    /// Returns an error when the format is unsupported, the map would exceed
    /// [`MAX_MAP_BYTES`], or the buffer length does not match the resolution
    /// and format exactly. A buffer that is nearly the right size is a bug that
    /// otherwise surfaces as a diagonally sheared texture.
    pub fn new(
        role: MapRole,
        format: TextureFormat,
        resolution: Resolution,
        pixels: Vec<u8>,
    ) -> Result<Self> {
        let expected = resolution
            .texels()
            .checked_mul(format.bytes_per_texel()?)
            .ok_or_else(|| invalid("texture size overflows"))?;
        if expected > MAX_MAP_BYTES {
            return Err(invalid("texture exceeds the per-map byte limit")
                .with_context("bytes", expected.to_string())
                .with_context("limit", MAX_MAP_BYTES.to_string()));
        }
        if pixels.len() as u64 != expected {
            return Err(invalid("pixel buffer does not match the declared size")
                .with_context("role", role.as_str())
                .with_context("expected", expected.to_string())
                .with_context("found", pixels.len().to_string()));
        }
        Ok(Self {
            role,
            format,
            resolution,
            pixels,
        })
    }

    /// What this map describes.
    #[must_use]
    pub const fn role(&self) -> MapRole {
        self.role
    }

    /// The pixel format.
    #[must_use]
    pub const fn format(&self) -> TextureFormat {
        self.format
    }

    /// The dimensions.
    #[must_use]
    pub const fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// The raw pixels, row-major from the top-left.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// How many bytes the map occupies.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }

    /// The colour space this map's values are in.
    #[must_use]
    pub const fn color_space(&self) -> ColorSpace {
        self.role.color_space()
    }
}

/// The maps a material carries, one per role at most.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapSet {
    maps: BTreeMap<MapRole, TextureMap>,
}

impl MapSet {
    /// An empty set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a map, refusing to replace one already present.
    ///
    /// # Errors
    ///
    /// Returns an error when the role is already filled. Silently replacing a
    /// map is how a pipeline ends up producing output nobody can trace to an
    /// input.
    pub fn insert(&mut self, map: TextureMap) -> Result<()> {
        let role = map.role();
        if self.maps.contains_key(&role) {
            return Err(invalid("this role already has a map").with_context("role", role.as_str()));
        }
        self.maps.insert(role, map);
        Ok(())
    }

    /// The map for a role, if present.
    #[must_use]
    pub fn get(&self, role: MapRole) -> Option<&TextureMap> {
        self.maps.get(&role)
    }

    /// Whether a role is filled.
    #[must_use]
    pub fn contains(&self, role: MapRole) -> bool {
        self.maps.contains_key(&role)
    }

    /// How many maps are present.
    #[must_use]
    pub fn len(&self) -> usize {
        self.maps.len()
    }

    /// Whether no map is present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }

    /// Every map, in role order.
    pub fn iter(&self) -> impl Iterator<Item = &TextureMap> {
        self.maps.values()
    }

    /// Total bytes held.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.maps.values().map(TextureMap::byte_len).sum()
    }

    /// Where every role stands, given which roles this material wants.
    ///
    /// The report the brief §6 asks for. `wanted` is the material's own opinion
    /// — a stone with no metal in it does not want a metallic map, and saying
    /// so turns an absence into an answer instead of a gap.
    #[must_use]
    pub fn status(&self, wanted: &[MapRole]) -> BTreeMap<MapRole, MapStatus> {
        MapRole::ALL
            .into_iter()
            .map(|role| {
                let asked_for = role.is_required() || wanted.contains(&role);
                let status = match (self.maps.contains_key(&role), asked_for) {
                    (true, _) => MapStatus::Generated,
                    (false, true) if role.is_required() => MapStatus::Required,
                    (false, true) => MapStatus::Optional,
                    (false, false) => MapStatus::Missing,
                };
                (role, status)
            })
            .collect()
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "texture", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(role: MapRole, edge: u32) -> TextureMap {
        let format = TextureFormat::eight_bit(role.natural_layout());
        let resolution = Resolution::square(edge).expect("valid edge");
        let bytes = (resolution.texels() * format.bytes_per_texel().unwrap()) as usize;
        TextureMap::new(role, format, resolution, vec![0; bytes]).expect("well-formed map")
    }

    #[test]
    fn resolutions_must_be_powers_of_two_inside_the_ladder() {
        for edge in [4, 16, 64, 512, 1024, 2048, 4096, 8192] {
            Resolution::square(edge).unwrap_or_else(|_| panic!("{edge} must be accepted"));
        }
        for edge in [0, 1, 3, 100, 1000, 12288, u32::MAX] {
            assert!(Resolution::square(edge).is_err(), "{edge} must be refused");
        }
        let wide = Resolution::new(64, 16).expect("non-square is legal");
        assert!(!wide.is_square());
        assert_eq!(wide.texels(), 1024);
    }

    #[test]
    fn a_buffer_that_is_nearly_right_is_refused() {
        let format = TextureFormat::eight_bit(ChannelLayout::Rgba);
        let resolution = Resolution::square(16).unwrap();
        let correct = (16 * 16 * 4) as usize;

        TextureMap::new(MapRole::Albedo, format, resolution, vec![0; correct])
            .expect("the exact size is accepted");

        for wrong in [correct - 1, correct + 1, 0] {
            let err = TextureMap::new(MapRole::Albedo, format, resolution, vec![0; wrong])
                .expect_err("a mismatched buffer must be refused");
            assert_eq!(err.recovery(), Recovery::Reject);
            assert!(err.to_string().contains("declared size"), "{err}");
        }
    }

    #[test]
    fn an_unsupported_bit_depth_fails_where_it_is_asked_for() {
        let format = TextureFormat {
            channels: ChannelLayout::Rgb,
            bits_per_channel: 32,
        };
        let err = format
            .bytes_per_texel()
            .expect_err("32 bits is not encodable");
        assert!(err.to_string().contains("bit depth"), "{err}");

        assert_eq!(
            TextureFormat::eight_bit(ChannelLayout::Rgba)
                .bytes_per_texel()
                .unwrap(),
            4
        );
        assert_eq!(
            TextureFormat::sixteen_bit(ChannelLayout::Grey)
                .bytes_per_texel()
                .unwrap(),
            2
        );
    }

    #[test]
    fn an_oversized_map_is_refused_before_it_is_allocated() {
        // 8192 square RGBA at 16 bits is 512 MiB: past the cap, and the error
        // arrives without anyone allocating it.
        let format = TextureFormat::sixteen_bit(ChannelLayout::Rgba);
        let resolution = Resolution::square(8192).unwrap();
        let err = TextureMap::new(MapRole::Albedo, format, resolution, Vec::new())
            .expect_err("must refuse before allocating");
        assert!(err.to_string().contains("byte limit"), "{err}");
    }

    #[test]
    fn albedo_is_the_only_perceptual_map() {
        assert_eq!(MapRole::Albedo.color_space(), ColorSpace::Srgb);
        for role in MapRole::ALL {
            if role != MapRole::Albedo {
                assert_eq!(role.color_space(), ColorSpace::Linear, "{role:?}");
            }
        }
        assert_eq!(ColorSpace::Srgb.as_str(), "srgb");
    }

    #[test]
    fn role_names_round_trip_and_unknown_names_are_refused() {
        for role in MapRole::ALL {
            assert_eq!(MapRole::parse(role.as_str()).unwrap(), role);
        }
        assert!(MapRole::parse("specular").is_err());
        assert!(MapRole::parse("Albedo").is_err());

        for layout in [
            ChannelLayout::Grey,
            ChannelLayout::GreyAlpha,
            ChannelLayout::Rgb,
            ChannelLayout::Rgba,
        ] {
            assert_eq!(ChannelLayout::parse(layout.as_str()).unwrap(), layout);
        }
        assert!(ChannelLayout::parse("bgr").is_err());
    }

    #[test]
    fn only_albedo_is_required_and_the_status_report_says_so() {
        let mut maps = MapSet::new();
        assert!(maps.is_empty());

        let empty = maps.status(&[MapRole::Roughness]);
        assert_eq!(empty[&MapRole::Albedo], MapStatus::Required);
        assert_eq!(empty[&MapRole::Roughness], MapStatus::Optional);
        assert_eq!(empty[&MapRole::Metallic], MapStatus::Missing);
        assert!(empty[&MapRole::Albedo].is_blocking());
        assert!(!empty[&MapRole::Roughness].is_blocking());
        assert!(!empty[&MapRole::Metallic].is_blocking());

        maps.insert(map(MapRole::Albedo, 16)).unwrap();
        maps.insert(map(MapRole::Roughness, 16)).unwrap();
        let filled = maps.status(&[MapRole::Roughness]);
        assert_eq!(filled[&MapRole::Albedo], MapStatus::Generated);
        assert_eq!(filled[&MapRole::Roughness], MapStatus::Generated);
        assert_eq!(filled[&MapRole::Metallic], MapStatus::Missing);
        assert!(filled.values().all(|status| !status.is_blocking()));

        // Every role appears, present or not: the report has no gaps.
        assert_eq!(filled.len(), MapRole::ALL.len());
    }

    #[test]
    fn a_map_set_refuses_to_overwrite_a_role() {
        let mut maps = MapSet::new();
        maps.insert(map(MapRole::Albedo, 16)).unwrap();
        let err = maps
            .insert(map(MapRole::Albedo, 32))
            .expect_err("replacement must be refused");
        assert!(err.to_string().contains("already"), "{err}");

        // The original survived, untouched.
        assert_eq!(maps.get(MapRole::Albedo).unwrap().resolution().width, 16);
        assert_eq!(maps.len(), 1);
        assert!(maps.contains(MapRole::Albedo));
    }

    #[test]
    fn a_map_set_reports_what_it_costs() {
        let mut maps = MapSet::new();
        maps.insert(map(MapRole::Albedo, 16)).unwrap();
        maps.insert(map(MapRole::Height, 16)).unwrap();
        // 16*16*4 for RGBA albedo, 16*16*1 for the single-channel height map.
        assert_eq!(maps.byte_len(), 1024 + 256);
        assert_eq!(maps.iter().count(), 2);
    }

    #[test]
    fn natural_layouts_match_what_each_role_needs() {
        assert_eq!(MapRole::Albedo.natural_layout(), ChannelLayout::Rgba);
        assert_eq!(MapRole::Normal.natural_layout(), ChannelLayout::Rgb);
        assert_eq!(MapRole::Height.natural_layout(), ChannelLayout::Grey);
        assert!(ChannelLayout::Rgba.has_alpha());
        assert!(!ChannelLayout::Rgb.has_alpha());
        assert_eq!(ChannelLayout::Rgb.count(), 3);
    }
}
