//! The surface a recipe draws on.
//!
//! A [`Canvas`] holds two fields at once: colour, and the height that produced
//! it. Recipes have a structural idea of their surface — where the gap between
//! two planks is, how deep the mortar sits — and throwing that away only to
//! recover it later by reading brightness out of the albedo map is how a normal
//! map ends up bumpy wherever the wood happened to be dark.
//!
//! So the height field is authored, not inferred, and the PBR pipeline gets a
//! real input rather than a guess.

use nexora_asset::texture::{ChannelLayout, MapRole, Resolution, TextureFormat, TextureMap};
use nexora_foundation::error::Result;

use crate::color::Rgba;

/// A colour field and a height field of the same size.
#[derive(Debug, Clone)]
pub struct Canvas {
    resolution: Resolution,
    albedo: Vec<Rgba>,
    height: Vec<f64>,
}

impl Canvas {
    /// A canvas of opaque black at zero height.
    #[must_use]
    pub fn new(resolution: Resolution) -> Self {
        let texels = resolution.texels() as usize;
        Self {
            resolution,
            albedo: vec![Rgba::BLACK; texels],
            height: vec![0.0; texels],
        }
    }

    /// The dimensions.
    #[must_use]
    pub const fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Texels across.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.resolution.width
    }

    /// Texels down.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.resolution.height
    }

    /// Write one texel's colour and height.
    ///
    /// Coordinates outside the canvas are ignored rather than wrapped: a recipe
    /// that walks off the edge has a bug, and silently painting the far side
    /// would hide it.
    pub fn put(&mut self, x: u32, y: u32, colour: Rgba, height: f64) {
        if x >= self.width() || y >= self.height() {
            return;
        }
        let index = (y as usize) * (self.width() as usize) + (x as usize);
        self.albedo[index] = colour;
        self.height[index] = height.clamp(0.0, 1.0);
    }

    /// Read one texel's colour.
    #[must_use]
    pub fn colour_at(&self, x: u32, y: u32) -> Rgba {
        if x >= self.width() || y >= self.height() {
            return Rgba::BLACK;
        }
        self.albedo[(y as usize) * (self.width() as usize) + (x as usize)]
    }

    /// Read one texel's height.
    #[must_use]
    pub fn height_at(&self, x: u32, y: u32) -> f64 {
        if x >= self.width() || y >= self.height() {
            return 0.0;
        }
        self.height[(y as usize) * (self.width() as usize) + (x as usize)]
    }

    /// The albedo field as an RGBA map.
    ///
    /// # Errors
    ///
    /// Returns an error only if the resolution and the buffer disagree, which
    /// would be a bug in this module rather than in a caller.
    pub fn albedo_map(&self) -> Result<TextureMap> {
        let mut pixels = Vec::with_capacity(self.albedo.len() * 4);
        for colour in &self.albedo {
            pixels.extend_from_slice(&[colour.r, colour.g, colour.b, colour.a]);
        }
        TextureMap::new(
            MapRole::Albedo,
            TextureFormat::eight_bit(ChannelLayout::Rgba),
            self.resolution,
            pixels,
        )
    }

    /// The height field as a single-channel map.
    ///
    /// # Errors
    ///
    /// As [`Canvas::albedo_map`].
    pub fn height_map(&self) -> Result<TextureMap> {
        let pixels = self
            .height
            .iter()
            .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect();
        TextureMap::new(
            MapRole::Height,
            TextureFormat::eight_bit(ChannelLayout::Grey),
            self.resolution,
            pixels,
        )
    }

    /// Every texel's height, row-major.
    #[must_use]
    pub fn heights(&self) -> &[f64] {
        &self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas() -> Canvas {
        Canvas::new(Resolution::square(8).unwrap())
    }

    #[test]
    fn a_new_canvas_is_black_and_flat() {
        let canvas = canvas();
        assert_eq!(canvas.width(), 8);
        assert_eq!(canvas.height(), 8);
        assert_eq!(canvas.colour_at(3, 3), Rgba::BLACK);
        assert!(canvas.height_at(3, 3).abs() < f64::EPSILON);
        assert_eq!(canvas.heights().len(), 64);
    }

    #[test]
    fn writing_and_reading_a_texel_round_trips() {
        let mut canvas = canvas();
        canvas.put(2, 5, Rgba::opaque(10, 20, 30), 0.5);
        assert_eq!(canvas.colour_at(2, 5), Rgba::opaque(10, 20, 30));
        assert!((canvas.height_at(2, 5) - 0.5).abs() < f64::EPSILON);
        // Its neighbour is untouched: the index arithmetic is not transposed.
        assert_eq!(canvas.colour_at(5, 2), Rgba::BLACK);
    }

    #[test]
    fn writing_outside_the_canvas_does_nothing_rather_than_wrapping() {
        let mut canvas = canvas();
        canvas.put(8, 0, Rgba::opaque(255, 0, 0), 1.0);
        canvas.put(0, 8, Rgba::opaque(255, 0, 0), 1.0);
        // If out-of-range wrapped, one of these would now be red.
        assert_eq!(canvas.colour_at(0, 0), Rgba::BLACK);
        assert_eq!(canvas.colour_at(7, 7), Rgba::BLACK);
        assert_eq!(canvas.colour_at(9, 9), Rgba::BLACK);
    }

    #[test]
    fn heights_are_clamped_on_the_way_in() {
        let mut canvas = canvas();
        canvas.put(1, 1, Rgba::BLACK, 5.0);
        canvas.put(2, 2, Rgba::BLACK, -5.0);
        assert!((canvas.height_at(1, 1) - 1.0).abs() < f64::EPSILON);
        assert!(canvas.height_at(2, 2).abs() < f64::EPSILON);
    }

    #[test]
    fn the_maps_have_the_layout_their_roles_expect() {
        let mut canvas = canvas();
        canvas.put(0, 0, Rgba::opaque(1, 2, 3), 1.0);

        let albedo = canvas.albedo_map().unwrap();
        assert_eq!(albedo.role(), MapRole::Albedo);
        assert_eq!(albedo.format().channels, ChannelLayout::Rgba);
        assert_eq!(albedo.byte_len(), 8 * 8 * 4);
        assert_eq!(&albedo.pixels()[0..4], &[1, 2, 3, 255]);

        let height = canvas.height_map().unwrap();
        assert_eq!(height.role(), MapRole::Height);
        assert_eq!(height.format().channels, ChannelLayout::Grey);
        assert_eq!(height.byte_len(), 8 * 8);
        assert_eq!(height.pixels()[0], 255);
        assert_eq!(height.pixels()[1], 0);
    }

    #[test]
    fn a_non_square_canvas_indexes_correctly() {
        let mut canvas = Canvas::new(Resolution::new(16, 4).unwrap());
        canvas.put(15, 3, Rgba::opaque(9, 9, 9), 1.0);
        assert_eq!(canvas.colour_at(15, 3), Rgba::opaque(9, 9, 9));
        assert_eq!(canvas.albedo_map().unwrap().byte_len(), 16 * 4 * 4);
        // The last texel is the last four bytes, not somewhere in the middle.
        let map = canvas.albedo_map().unwrap();
        assert_eq!(&map.pixels()[map.byte_len() - 4..], &[9, 9, 9, 255]);
    }
}
