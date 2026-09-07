//! Gravity fields (PHY-8).
//!
//! `PHYSICS.md` §9 asks for gravity to be a property of the dimension rather
//! than a constant in the solver, because Space and the special dimensions will
//! need low, high and radial gravity. This phase implements the **uniform**
//! field: a direction and a magnitude. Radial gravity and falloff are named in
//! the document and are not here; see the technical debt register rather than
//! assuming they were forgotten.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

use crate::math::Vec3;

/// Standard surface gravity, in metres per second squared.
pub const EARTHLIKE_GRAVITY: f64 = 9.806_65;

/// The largest magnitude a field may declare.
///
/// Not a physical limit — a bound on untrusted configuration. A mod-supplied
/// field of `1e300` would push every body past the sweep cap on its first step.
pub const MAX_GRAVITY_MAGNITUDE: f64 = 1_000.0;

/// A uniform acceleration applied to every dynamic body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityField {
    acceleration: Vec3,
}

impl GravityField {
    /// Earthlike gravity pulling towards negative Y.
    #[must_use]
    pub fn earthlike() -> Self {
        Self {
            acceleration: Vec3::new(0.0, -EARTHLIKE_GRAVITY, 0.0),
        }
    }

    /// No gravity at all.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            acceleration: Vec3::ZERO,
        }
    }

    /// A field pulling along `direction` at `magnitude` metres per second
    /// squared.
    ///
    /// # Errors
    ///
    /// Returns an error when the direction has no length, when either input is
    /// not finite, or when the magnitude is negative or exceeds
    /// [`MAX_GRAVITY_MAGNITUDE`].
    pub fn directional(direction: Vec3, magnitude: f64) -> Result<Self> {
        direction.require_finite("gravity-field")?;
        if !magnitude.is_finite() || !(0.0..=MAX_GRAVITY_MAGNITUDE).contains(&magnitude) {
            return Err(Error::new(
                Domain::Physics,
                "gravity-field",
                "gravity magnitude outside its permitted range",
            )
            .with_recovery(Recovery::Reject)
            .with_context("magnitude", magnitude.to_string())
            .with_context("max", MAX_GRAVITY_MAGNITUDE.to_string()));
        }
        let length = direction.length();
        if length <= 0.0 {
            return Err(Error::new(
                Domain::Physics,
                "gravity-field",
                "gravity direction has no length",
            )
            .with_recovery(Recovery::Reject));
        }
        Ok(Self {
            acceleration: direction.scaled(magnitude / length),
        })
    }

    /// The acceleration this field applies, in metres per second squared.
    #[must_use]
    pub const fn acceleration(self) -> Vec3 {
        self.acceleration
    }

    /// The magnitude of the field.
    #[must_use]
    pub fn magnitude(self) -> f64 {
        self.acceleration.length()
    }

    /// Whether the field does anything.
    #[must_use]
    pub fn is_weightless(self) -> bool {
        self.acceleration == Vec3::ZERO
    }
}

impl Default for GravityField {
    fn default() -> Self {
        Self::earthlike()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earthlike_gravity_pulls_down_at_the_published_value() {
        let field = GravityField::earthlike();
        assert!(field.acceleration().y < 0.0);
        assert!((field.magnitude() - EARTHLIKE_GRAVITY).abs() < 1e-12);
    }

    #[test]
    fn weightlessness_is_representable() {
        assert!(GravityField::none().is_weightless());
        assert!(!GravityField::earthlike().is_weightless());
    }

    #[test]
    fn a_direction_is_normalised_before_the_magnitude_is_applied() {
        // A long direction vector must not multiply the strength.
        let field = GravityField::directional(Vec3::new(0.0, -100.0, 0.0), 3.0).expect("valid");
        assert!((field.magnitude() - 3.0).abs() < 1e-12);
        assert!((field.acceleration().y + 3.0).abs() < 1e-12);
    }

    #[test]
    fn a_zero_direction_is_refused_rather_than_producing_nan() {
        let err = GravityField::directional(Vec3::ZERO, 9.8).expect_err("no direction");
        assert_eq!(err.domain(), Domain::Physics);
    }

    #[test]
    fn absurd_or_non_finite_magnitudes_are_refused() {
        let down = Vec3::new(0.0, -1.0, 0.0);
        assert!(GravityField::directional(down, -1.0).is_err());
        assert!(GravityField::directional(down, f64::NAN).is_err());
        assert!(GravityField::directional(down, MAX_GRAVITY_MAGNITUDE * 2.0).is_err());
        assert!(GravityField::directional(down, MAX_GRAVITY_MAGNITUDE).is_ok());
    }

    #[test]
    fn a_sideways_field_is_legal() {
        let field = GravityField::directional(Vec3::new(1.0, 0.0, 0.0), 5.0).expect("valid");
        assert!((field.acceleration().x - 5.0).abs() < 1e-12);
        assert!(field.acceleration().y.abs() < 1e-12);
    }
}
