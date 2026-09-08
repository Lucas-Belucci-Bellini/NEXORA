//! Colour, ramps and the quantisation that produces a pixel-art look.
//!
//! Everything here works in 8-bit sRGB, because that is what an albedo map
//! stores and what the art direction actually is. `NEXORA ART DIRECTION AND
//! PROCEDURAL VARIATION.md` asks for a *"recognisable original visual
//! identity"* built from a small library of primitives plus controlled
//! variation, not for physically measured reflectance.
//!
//! # Why quantisation is a first-class step rather than a filter
//!
//! A voxel texture reads as hand-drawn when its palette is small and its steps
//! are deliberate. Continuous noise mapped straight to 24-bit colour reads as
//! photographic mush at 32 pixels. So a [`Ramp`] holds a handful of stops and
//! [`Ramp::quantised`] snaps to them: the limited palette is where the style
//! comes from, and it is chosen, not lost.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// An 8-bit sRGB colour with coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Coverage.
    pub a: u8,
}

impl Rgba {
    /// Opaque black.
    pub const BLACK: Self = Self::opaque(0, 0, 0);

    /// An opaque colour.
    #[must_use]
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Mix towards another colour. `t` is clamped to `0.0..=1.0`.
    #[must_use]
    pub fn mix(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| -> u8 {
            let a = f64::from(a);
            let b = f64::from(b);
            (a + (b - a) * t).round().clamp(0.0, 255.0) as u8
        };
        Self {
            r: lerp(self.r, other.r),
            g: lerp(self.g, other.g),
            b: lerp(self.b, other.b),
            a: lerp(self.a, other.a),
        }
    }

    /// Scale brightness by a factor, keeping coverage.
    ///
    /// Used for the per-plank value jitter and the darkened edges that make
    /// adjacent strips read as separate boards rather than as one surface.
    #[must_use]
    pub fn scaled(self, factor: f64) -> Self {
        let apply =
            |channel: u8| -> u8 { (f64::from(channel) * factor).round().clamp(0.0, 255.0) as u8 };
        Self {
            r: apply(self.r),
            g: apply(self.g),
            b: apply(self.b),
            a: self.a,
        }
    }

    /// Perceived brightness, `0.0` to `1.0`.
    ///
    /// Rec. 601 luma coefficients. Used to derive a roughness or height field
    /// from an albedo map when a recipe has no better source.
    #[must_use]
    pub fn luma(self) -> f64 {
        (0.299 * f64::from(self.r) + 0.587 * f64::from(self.g) + 0.114 * f64::from(self.b)) / 255.0
    }
}

/// A colour ramp: the palette a recipe draws from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ramp {
    stops: Vec<Rgba>,
}

impl Ramp {
    /// Build a ramp from its stops, darkest first by convention.
    ///
    /// # Errors
    ///
    /// Returns an error when fewer than two stops are given: a ramp with one
    /// colour is a constant, and a recipe that wanted a constant should say so.
    pub fn new(stops: Vec<Rgba>) -> Result<Self> {
        if stops.len() < 2 {
            return Err(invalid("a ramp needs at least two stops")
                .with_context("stops", stops.len().to_string()));
        }
        Ok(Self { stops })
    }

    /// A ramp interpolated between two colours, in `steps` stops.
    ///
    /// # Errors
    ///
    /// Returns an error when fewer than two steps are asked for.
    pub fn between(dark: Rgba, light: Rgba, steps: usize) -> Result<Self> {
        if steps < 2 {
            return Err(
                invalid("a ramp needs at least two steps").with_context("steps", steps.to_string())
            );
        }
        let last = (steps - 1) as f64;
        Ok(Self {
            stops: (0..steps)
                .map(|index| dark.mix(light, index as f64 / last))
                .collect(),
        })
    }

    /// How many stops the ramp holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stops.len()
    }

    /// Whether the ramp is empty. Never true for a constructed ramp.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// The stops.
    #[must_use]
    pub fn stops(&self) -> &[Rgba] {
        &self.stops
    }

    /// Sample the ramp continuously. `t` is clamped to `0.0..=1.0`.
    #[must_use]
    pub fn sample(&self, t: f64) -> Rgba {
        let last = (self.stops.len() - 1) as f64;
        let position = t.clamp(0.0, 1.0) * last;
        let low = position.floor();
        let index = low as usize;
        if index + 1 >= self.stops.len() {
            return self.stops[self.stops.len() - 1];
        }
        self.stops[index].mix(self.stops[index + 1], position - low)
    }

    /// Snap to the nearest stop. `t` is clamped to `0.0..=1.0`.
    ///
    /// The pixel-art step: the result is always one of the palette's colours,
    /// never something between two of them.
    #[must_use]
    pub fn quantised(&self, t: f64) -> Rgba {
        let last = (self.stops.len() - 1) as f64;
        let index = (t.clamp(0.0, 1.0) * last).round() as usize;
        self.stops[index.min(self.stops.len() - 1)]
    }
}

/// Snap a `0.0..=1.0` value to one of `levels` evenly spaced steps.
///
/// # Panics
///
/// Never: `levels` of zero or one is treated as "no quantisation", which is the
/// only sensible reading of a request to snap to fewer than two values.
#[must_use]
pub fn quantise(value: f64, levels: u32) -> f64 {
    if levels < 2 {
        return value.clamp(0.0, 1.0);
    }
    let steps = f64::from(levels - 1);
    (value.clamp(0.0, 1.0) * steps).round() / steps
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "color", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixing_reaches_both_ends_exactly() {
        let dark = Rgba::opaque(10, 20, 30);
        let light = Rgba::opaque(200, 210, 220);
        assert_eq!(dark.mix(light, 0.0), dark);
        assert_eq!(dark.mix(light, 1.0), light);
        // Out-of-range factors clamp rather than extrapolating into nonsense.
        assert_eq!(dark.mix(light, -5.0), dark);
        assert_eq!(dark.mix(light, 5.0), light);

        let middle = dark.mix(light, 0.5);
        assert_eq!(middle, Rgba::opaque(105, 115, 125));
    }

    #[test]
    fn scaling_saturates_instead_of_wrapping() {
        let colour = Rgba::opaque(200, 100, 50);
        assert_eq!(colour.scaled(0.5), Rgba::opaque(100, 50, 25));
        // 200 * 4 would wrap to 32 in u8 arithmetic. It must clamp to 255.
        assert_eq!(colour.scaled(4.0), Rgba::opaque(255, 255, 200));
        assert_eq!(colour.scaled(0.0), Rgba::BLACK);
        // Coverage is not brightness and is left alone.
        assert_eq!(colour.scaled(0.5).a, 255);
    }

    #[test]
    fn luma_orders_colours_the_way_an_eye_does() {
        assert!((Rgba::opaque(255, 255, 255).luma() - 1.0).abs() < 1e-9);
        assert!(Rgba::BLACK.luma().abs() < 1e-9);
        // Green looks brighter than blue at the same channel value.
        assert!(Rgba::opaque(0, 255, 0).luma() > Rgba::opaque(0, 0, 255).luma());
    }

    #[test]
    fn a_ramp_needs_at_least_two_stops() {
        assert!(Ramp::new(vec![Rgba::BLACK]).is_err());
        assert!(Ramp::new(Vec::new()).is_err());
        assert!(Ramp::between(Rgba::BLACK, Rgba::opaque(255, 255, 255), 1).is_err());
        let ramp = Ramp::new(vec![Rgba::BLACK, Rgba::opaque(255, 255, 255)]).unwrap();
        assert_eq!(ramp.len(), 2);
        assert!(!ramp.is_empty());
    }

    #[test]
    fn sampling_a_ramp_is_continuous_and_hits_both_ends() {
        let ramp = Ramp::between(Rgba::opaque(20, 10, 0), Rgba::opaque(220, 180, 120), 5).unwrap();
        assert_eq!(ramp.sample(0.0), ramp.stops()[0]);
        assert_eq!(ramp.sample(1.0), ramp.stops()[4]);
        assert_eq!(ramp.sample(2.0), ramp.stops()[4]);

        // Monotonic in brightness, which is what makes a ramp a ramp.
        let mut previous = -1.0;
        for step in 0..=20 {
            let luma = ramp.sample(f64::from(step) / 20.0).luma();
            assert!(luma >= previous - 1e-9, "ramp must not dip at {step}");
            previous = luma;
        }
    }

    #[test]
    fn quantising_a_ramp_returns_only_palette_colours() {
        let ramp = Ramp::between(Rgba::opaque(30, 20, 10), Rgba::opaque(210, 170, 120), 6).unwrap();
        for step in 0..=100 {
            let colour = ramp.quantised(f64::from(step) / 100.0);
            assert!(
                ramp.stops().contains(&colour),
                "quantised colour must be a stop, got {colour:?}"
            );
        }
        // The whole range is used, not just the middle.
        assert_eq!(ramp.quantised(0.0), ramp.stops()[0]);
        assert_eq!(ramp.quantised(1.0), ramp.stops()[5]);
    }

    #[test]
    fn quantising_a_value_snaps_to_evenly_spaced_steps() {
        assert!((quantise(0.0, 5) - 0.0).abs() < 1e-9);
        assert!((quantise(1.0, 5) - 1.0).abs() < 1e-9);
        assert!((quantise(0.3, 5) - 0.25).abs() < 1e-9);
        assert!((quantise(0.6, 5) - 0.5).abs() < 1e-9);

        // Fewer than two levels means "leave it alone", not "collapse to zero".
        assert!((quantise(0.42, 1) - 0.42).abs() < 1e-9);
        assert!((quantise(0.42, 0) - 0.42).abs() < 1e-9);
        // Out of range still clamps.
        assert!((quantise(9.0, 4) - 1.0).abs() < 1e-9);
    }
}
