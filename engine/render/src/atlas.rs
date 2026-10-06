//! The albedo atlas (RENDER-15), and the one rule for which texel a point of
//! a face shows (ADR-0037).
//!
//! `RENDERER and GRAPHICS.md` RENDER-15: *Block IDs → Texture Atlas → GPU*,
//! with texture arrays left possible. Here the blocks are already surfaces —
//! a material's runtime id, resolved by the simulation's `SurfaceTable` — so
//! an atlas is a list of 16×16 tiles and a table from surface to tile. The
//! image is [`ATLAS_COLUMNS`] tiles wide and as many rows tall as it needs.
//!
//! # One rule, written twice
//!
//! The pass samples the atlas in WGSL and the reference ray cast samples it
//! here, and a frame is only believed when the two agree pixel by pixel. So
//! everything that decides a pixel's colour is a function in this module —
//! [`texel_of`], [`FACE_SHADE`], [`shade_texel`] — and the shader is a
//! transcription of it, kept beside it in [`crate::textured_wgsl`]:
//!
//! * **Which texel.** A point of a face, in blocks, keeps only its fraction
//!   within its block: a greedy rectangle of many blocks repeats its tile
//!   once per block. Faces across X read `(z, y)`, across Y `(x, z)`, across
//!   Z `(x, y)`; the vertical one counts down from the top, so a tile's first
//!   row is a wall's top.
//! * **No filtering.** The texel is fetched by its integer coordinates, so
//!   nothing about a sampler — filter, mip level, address mode — can make a
//!   frame differ from the reference.
//! * **Light is a factor per face**, applied in linear space: the albedo is
//!   sRGB, the atlas is uploaded as sRGB so the GPU decodes it, and the frame
//!   is an sRGB target, so the GPU encodes the result. [`shade_texel`] does
//!   the same arithmetic on the CPU.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::Axis;
use nexora_mesh::{Facing, SurfaceId};
use nexora_rhi::{Command, CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};

/// A tile's side, in texels: the first generation's rule (16×16).
pub const TILE: u32 = 16;

/// Tiles across the atlas.
pub const ATLAS_COLUMNS: u32 = 16;

/// The most tiles one atlas holds: 512 rows, an image 8,192 texels tall — the
/// largest 2D texture every `wgpu` backend is required to support.
pub const MAX_TILES: u32 = ATLAS_COLUMNS * 512;

/// The tile every surface without a texture shows: diagonal bands of magenta
/// and near-black, unlike any material, so a block whose texture is missing
/// is visible and plainly wrong rather than silently another block.
pub const UNMAPPED_TILE: u32 = 0;

/// The light each face receives, by face index (`face_index`): `+X`, `−X`,
/// `+Y`, `−Y`, `+Z`, `−Z`.
///
/// A fixed sun from above and a little to the side: tops full, bottoms half,
/// and the two horizontal axes apart so a corner reads as one. Not a light
/// model — there is none yet (RENDER-18) — only enough to tell faces apart.
pub const FACE_SHADE: [f32; 6] = [0.80, 0.80, 1.00, 0.50, 0.90, 0.90];

/// A face as the index the pass and [`FACE_SHADE`] use: `axis · 2`, plus one
/// when it faces the negative way.
#[must_use]
pub const fn face_index(axis: Axis, facing: Facing) -> u32 {
    let base = match axis {
        Axis::X => 0,
        Axis::Y => 2,
        Axis::Z => 4,
    };
    match facing {
        Facing::Positive => base,
        Facing::Negative => base + 1,
    }
}

/// The texel `(x, y)` of a tile that a point of a face shows, for a point
/// `p` in blocks (any whole-block offset: only the fraction within the block
/// counts).
#[must_use]
pub fn texel_of(axis: Axis, p: [f64; 3]) -> (u32, u32) {
    let (s, t) = match axis {
        Axis::X => (p[2], p[1]),
        Axis::Y => (p[0], p[2]),
        Axis::Z => (p[0], p[1]),
    };
    let texel = |value: f64| {
        ((value - value.floor()) * f64::from(TILE))
            .floor()
            .clamp(0.0, 15.0) as u32
    };
    let (x, y) = (texel(s), texel(t));
    // Walls count down from the top; a floor's rows run along +Z.
    let y = if axis == Axis::Y { y } else { TILE - 1 - y };
    (x, y)
}

/// An sRGB-encoded channel as linear light.
#[must_use]
pub fn srgb_to_linear(channel: u8) -> f32 {
    let c = f32::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light as an sRGB-encoded channel, rounded to the nearest.
#[must_use]
pub fn linear_to_srgb(light: f32) -> u8 {
    let l = light.clamp(0.0, 1.0);
    let c = if l <= 0.003_130_8 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (c * 255.0).round().clamp(0.0, 255.0) as u8
}

/// The colour an sRGB target holds for an albedo texel lit by `shade`:
/// decoded, multiplied, encoded — what the pass's fragment shader computes.
#[must_use]
pub fn shade_texel(texel: [u8; 4], shade: f32) -> [u8; 4] {
    [
        linear_to_srgb(srgb_to_linear(texel[0]) * shade),
        linear_to_srgb(srgb_to_linear(texel[1]) * shade),
        linear_to_srgb(srgb_to_linear(texel[2]) * shade),
        255,
    ]
}

/// The bytes of one 16×16 RGBA8 tile.
const TILE_BYTES: usize = (TILE * TILE * 4) as usize;

/// Tiles of albedo, and which surface shows which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atlas {
    tiles: Vec<Vec<u8>>,
    index: BTreeMap<SurfaceId, u32>,
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

impl Atlas {
    /// An atlas holding only [`UNMAPPED_TILE`].
    #[must_use]
    pub fn new() -> Self {
        let mut unmapped = Vec::with_capacity(TILE_BYTES);
        for y in 0..TILE {
            for x in 0..TILE {
                let band = ((x + y) / 4) % 2 == 0;
                unmapped.extend_from_slice(if band {
                    &[214, 38, 186, 255]
                } else {
                    &[24, 14, 30, 255]
                });
            }
        }
        Self {
            tiles: vec![unmapped],
            index: BTreeMap::new(),
        }
    }

    /// Give `surface` a tile of 16×16 RGBA8 texels, sRGB, row by row from the
    /// top, and return the tile.
    ///
    /// # Errors
    ///
    /// [`Recovery::Reject`] when the texels are not one tile, a texel is not
    /// opaque (the pass is the opaque layer), the surface already has a tile,
    /// or the atlas is full.
    pub fn add(&mut self, surface: SurfaceId, rgba: &[u8]) -> Result<u32> {
        if rgba.len() != TILE_BYTES {
            return Err(
                refused("a tile is not 16x16 RGBA8").with_context("bytes", rgba.len().to_string())
            );
        }
        if rgba.chunks_exact(4).any(|texel| texel[3] != 255) {
            return Err(refused("an opaque surface's tile has a transparent texel")
                .with_context("surface", surface.0.to_string()));
        }
        if self.index.contains_key(&surface) {
            return Err(refused("a surface already has a tile")
                .with_context("surface", surface.0.to_string()));
        }
        let tile = u32::try_from(self.tiles.len()).unwrap_or(u32::MAX);
        if tile >= MAX_TILES {
            return Err(refused("the atlas is full").with_context("tiles", tile.to_string()));
        }
        self.tiles.push(rgba.to_vec());
        self.index.insert(surface, tile);
        Ok(tile)
    }

    /// The tile a surface shows: its own, or [`UNMAPPED_TILE`].
    #[must_use]
    pub fn tile_of(&self, surface: SurfaceId) -> u32 {
        self.index.get(&surface).copied().unwrap_or(UNMAPPED_TILE)
    }

    /// Tiles held, [`UNMAPPED_TILE`] included.
    #[must_use]
    pub fn tiles(&self) -> u32 {
        u32::try_from(self.tiles.len()).unwrap_or(u32::MAX)
    }

    /// The image's width and height, in texels.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        let rows = self.tiles().div_ceil(ATLAS_COLUMNS).max(1);
        (ATLAS_COLUMNS * TILE, rows * TILE)
    }

    /// Texel `(x, y)` of `tile`, RGBA8 sRGB. A tile the atlas does not hold
    /// reads as [`UNMAPPED_TILE`].
    #[must_use]
    pub fn texel(&self, tile: u32, x: u32, y: u32) -> [u8; 4] {
        let tile = self
            .tiles
            .get(tile as usize)
            .unwrap_or(&self.tiles[UNMAPPED_TILE as usize]);
        let at = ((y.min(TILE - 1) * TILE + x.min(TILE - 1)) * 4) as usize;
        [tile[at], tile[at + 1], tile[at + 2], tile[at + 3]]
    }

    /// The whole image, RGBA8 sRGB, row by row from the top; tiles fill rows
    /// left to right, and the cells after the last tile are black.
    #[must_use]
    pub fn pixels(&self) -> Vec<u8> {
        let (width, height) = self.size();
        let mut out = vec![0u8; (width * height * 4) as usize];
        for (index, tile) in self.tiles.iter().enumerate() {
            let index = index as u32;
            let (column, row) = (index % ATLAS_COLUMNS, index / ATLAS_COLUMNS);
            for y in 0..TILE {
                let from = (y * TILE * 4) as usize;
                let to = (((row * TILE + y) * width + column * TILE) * 4) as usize;
                out[to..to + (TILE * 4) as usize]
                    .copy_from_slice(&tile[from..from + (TILE * 4) as usize]);
            }
        }
        out
    }

    /// Create the atlas as an sRGB texture and append its upload to `list`.
    ///
    /// # Errors
    ///
    /// The backend refused the texture.
    pub fn upload<R: Rhi + ?Sized>(
        &self,
        rhi: &mut R,
        list: &mut CommandList,
    ) -> Result<TextureHandle> {
        let (width, height) = self.size();
        let texture = rhi.create_texture(&TextureDesc {
            label: "albedo atlas".into(),
            width,
            height,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: Usage::SAMPLED | Usage::COPY_DST,
        })?;
        list.push(Command::WriteTexture {
            texture,
            data: self.pixels(),
        });
        Ok(texture)
    }
}

fn refused(message: &'static str) -> Error {
    Error::new(Domain::Render, "albedo-atlas", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(rgba: [u8; 4]) -> Vec<u8> {
        rgba.repeat((TILE * TILE) as usize)
    }

    #[test]
    fn tiles_fill_rows_and_unmapped_surfaces_show_the_reserved_tile() {
        let mut atlas = Atlas::new();
        assert_eq!(atlas.size(), (256, 16));
        for k in 0..20u32 {
            let tile = atlas
                .add(SurfaceId(100 + k), &solid([k as u8, 1, 2, 255]))
                .unwrap();
            assert_eq!(tile, k + 1);
        }
        assert_eq!(atlas.size(), (256, 32), "twenty-one tiles are two rows");
        assert_eq!(atlas.tile_of(SurfaceId(107)), 8);
        assert_eq!(atlas.tile_of(SurfaceId(7)), UNMAPPED_TILE);
        let pixels = atlas.pixels();
        // Tile 17 is the second row's second cell.
        let at = ((16 * 256 + 16) * 4) as usize;
        assert_eq!(&pixels[at..at + 4], &[16, 1, 2, 255]);
        assert_eq!(atlas.texel(17, 3, 3), [16, 1, 2, 255]);
    }

    #[test]
    fn a_tile_must_be_one_opaque_tile_for_one_surface() {
        let mut atlas = Atlas::new();
        assert!(atlas.add(SurfaceId(1), &[0; 16]).is_err());
        assert!(atlas.add(SurfaceId(1), &solid([1, 2, 3, 128])).is_err());
        atlas.add(SurfaceId(1), &solid([1, 2, 3, 255])).unwrap();
        assert!(atlas.add(SurfaceId(1), &solid([1, 2, 3, 255])).is_err());
    }

    /// The vertical axis counts down from the top of a wall and along +Z on
    /// a floor; the horizontal one along the axis in either case; and only
    /// the fraction within the block counts.
    #[test]
    fn texels_are_read_from_the_point_within_its_block() {
        // Just above the bottom of a wall's block: the tile's last row.
        assert_eq!(texel_of(Axis::X, [5.0, 3.01, 7.99]), (15, 15));
        assert_eq!(texel_of(Axis::Z, [-0.99, 3.99, 2.0]), (0, 0));
        assert_eq!(texel_of(Axis::Y, [10.5, 64.0, -3.25]), (8, 12));
        // A greedy rectangle repeats its tile each block.
        assert_eq!(
            texel_of(Axis::Y, [0.3, 0.0, 0.3]),
            texel_of(Axis::Y, [7.3, 0.0, 12.3])
        );
    }

    #[test]
    fn faces_have_six_indices_and_the_top_is_lit_fully() {
        let mut seen = Vec::new();
        for axis in Axis::ALL {
            for facing in [Facing::Positive, Facing::Negative] {
                seen.push(face_index(axis, facing));
            }
        }
        assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(
            FACE_SHADE[face_index(Axis::Y, Facing::Positive) as usize],
            1.0
        );
    }

    /// Full light gives back the texel; half light is darker on every
    /// channel; and the encode inverts the decode on every value.
    #[test]
    fn shading_is_done_in_linear_light() {
        for value in 0..=255u8 {
            assert_eq!(linear_to_srgb(srgb_to_linear(value)), value);
        }
        let texel = [180, 120, 60, 255];
        assert_eq!(shade_texel(texel, 1.0), texel);
        let half = shade_texel(texel, 0.5);
        assert!(half.iter().zip(&texel).take(3).all(|(a, b)| a < b));
        // Linear half of sRGB 180 is about sRGB 133, not 90.
        assert!((130..=136).contains(&half[0]), "{half:?}");
    }
}
