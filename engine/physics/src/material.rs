//! Physical materials (PHY-7) and how a contact combines two of them (PHY-15).
//!
//! `PHYSICS.md` §8 is explicit that physics must not assume everything behaves
//! the same, and §16 that friction depends on **both** surfaces in the contact,
//! not on one of them. So a material is data, contact response is a function of
//! two materials, and neither knows what block or entity it belongs to.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Index of a material in a [`MaterialTable`].
///
/// Deliberately an index rather than a name: the solver looks a material up per
/// voxel per axis per step, and a string comparison in that loop is a cost with
/// no benefit. The mapping from a namespaced identifier to a `MaterialId` is
/// the caller's, exactly as the block registry maps identifiers to runtime ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialId(pub u16);

impl MaterialId {
    /// The material every table starts with, used when nothing else is declared.
    pub const DEFAULT: Self = Self(0);
}

/// How a surface behaves in contact.
///
/// `density` is carried because buoyancy and mass-from-volume will need it
/// (PHY-16); nothing in this phase reads it, and that is recorded rather than
/// hidden — a field that lies about being used is worse than one documented as
/// reserved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsMaterial {
    /// Mass per unit volume, in kilograms per cubic metre. Reserved for PHY-16.
    pub density: f64,
    /// Resistance to sliding, `0.0` frictionless to `1.0` fully gripping.
    pub friction: f64,
    /// Bounce, `0.0` fully damped to `1.0` lossless.
    pub restitution: f64,
    /// Velocity lost per second while moving through this material.
    pub drag: f64,
}

impl PhysicsMaterial {
    /// A neutral surface: grippy, unbouncy, no drag.
    pub const DEFAULT: Self = Self {
        density: 1000.0,
        friction: 0.6,
        restitution: 0.0,
        drag: 0.0,
    };

    /// Validate the coefficients.
    ///
    /// # Errors
    ///
    /// Returns an error when a coefficient is not finite, negative, or outside
    /// the closed unit interval where the physics only makes sense inside it.
    pub fn validate(self) -> Result<Self> {
        let checks: [(&'static str, f64, f64); 4] = [
            ("density", self.density, f64::MAX),
            ("friction", self.friction, 1.0),
            ("restitution", self.restitution, 1.0),
            ("drag", self.drag, f64::MAX),
        ];
        for (name, value, limit) in checks {
            if !value.is_finite() || value < 0.0 || value > limit {
                return Err(Error::new(
                    Domain::Physics,
                    "physics-material",
                    "coefficient outside its permitted range",
                )
                .with_recovery(Recovery::Reject)
                .with_context("coefficient", name)
                .with_context("value", value.to_string())
                .with_context("max", limit.to_string()));
            }
        }
        Ok(self)
    }

    /// The friction of a contact between this material and another (PHY-15).
    ///
    /// The geometric mean, so the slipperier surface dominates: standing on ice
    /// in rubber boots is still slippery. An arithmetic mean would let a grippy
    /// body walk normally on ice, which is the wrong answer for the case the
    /// coefficient exists to describe.
    #[must_use]
    pub fn contact_friction(self, other: Self) -> f64 {
        (self.friction * other.friction).sqrt()
    }

    /// The bounce of a contact between this material and another.
    ///
    /// The maximum, so a bouncy body bounces off an unbouncy floor. Restitution
    /// describes what the pair gives back, and the more elastic surface is what
    /// gives it back.
    #[must_use]
    pub fn contact_restitution(self, other: Self) -> f64 {
        self.restitution.max(other.restitution)
    }
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The materials a physics world knows about.
///
/// Registration is append-only and the index is the identity, so a `MaterialId`
/// held across a step keeps meaning the same surface.
#[derive(Debug, Clone)]
pub struct MaterialTable {
    entries: Vec<PhysicsMaterial>,
}

impl MaterialTable {
    /// A table holding only [`PhysicsMaterial::DEFAULT`] at
    /// [`MaterialId::DEFAULT`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: vec![PhysicsMaterial::DEFAULT],
        }
    }

    /// Register a material and return its id.
    ///
    /// # Errors
    ///
    /// Returns an error when the material is invalid, or when the table is
    /// full. A full table is a real limit rather than a theoretical one: the id
    /// is a `u16` because it is read in the collision inner loop.
    pub fn register(&mut self, material: PhysicsMaterial) -> Result<MaterialId> {
        let material = material.validate()?;
        let index = u16::try_from(self.entries.len()).map_err(|_| {
            Error::new(
                Domain::Physics,
                "material-table",
                "the material table is full",
            )
            .with_recovery(Recovery::Reject)
            .with_context("capacity", u16::MAX.to_string())
        })?;
        self.entries.push(material);
        Ok(MaterialId(index))
    }

    /// Look a material up, falling back to the default for an unknown id.
    ///
    /// The fallback is deliberate. An unknown id in the collision loop means
    /// some caller registered a block against a table it does not own; the
    /// honest recovery is to keep the body solid and grippy rather than to make
    /// the world unwalkable. [`Self::require`] is the checked form for callers
    /// that can act on the failure.
    #[must_use]
    pub fn get(&self, id: MaterialId) -> PhysicsMaterial {
        self.entries
            .get(usize::from(id.0))
            .copied()
            .unwrap_or(PhysicsMaterial::DEFAULT)
    }

    /// Look a material up, refusing an unknown id.
    ///
    /// # Errors
    ///
    /// Returns an error when the id was never registered in this table.
    pub fn require(&self, id: MaterialId) -> Result<PhysicsMaterial> {
        self.entries.get(usize::from(id.0)).copied().ok_or_else(|| {
            Error::new(
                Domain::Physics,
                "material-table",
                "material id was never registered",
            )
            .with_recovery(Recovery::Reject)
            .with_context("id", id.0.to_string())
            .with_context("registered", self.entries.len().to_string())
        })
    }

    /// How many materials are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether only the default material is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }
}

impl Default for MaterialTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_table_answers_the_default_id() {
        let table = MaterialTable::new();
        assert_eq!(table.get(MaterialId::DEFAULT), PhysicsMaterial::DEFAULT);
        assert!(table.is_empty());
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn registration_hands_back_stable_ids() {
        let mut table = MaterialTable::new();
        let ice = table
            .register(PhysicsMaterial {
                friction: 0.02,
                ..PhysicsMaterial::DEFAULT
            })
            .expect("valid");
        let rubber = table
            .register(PhysicsMaterial {
                restitution: 0.9,
                ..PhysicsMaterial::DEFAULT
            })
            .expect("valid");
        assert_ne!(ice, rubber);
        assert!((table.get(ice).friction - 0.02).abs() < 1e-12);
        assert!((table.get(rubber).restitution - 0.9).abs() < 1e-12);
        // Registering more does not move the earlier ones.
        let _ = table.register(PhysicsMaterial::DEFAULT).expect("valid");
        assert!((table.get(ice).friction - 0.02).abs() < 1e-12);
    }

    #[test]
    fn invalid_coefficients_are_refused() {
        assert!(PhysicsMaterial {
            friction: 1.5,
            ..PhysicsMaterial::DEFAULT
        }
        .validate()
        .is_err());
        assert!(PhysicsMaterial {
            restitution: -0.1,
            ..PhysicsMaterial::DEFAULT
        }
        .validate()
        .is_err());
        assert!(PhysicsMaterial {
            drag: f64::NAN,
            ..PhysicsMaterial::DEFAULT
        }
        .validate()
        .is_err());
    }

    #[test]
    fn an_unknown_id_falls_back_but_require_reports_it() {
        let table = MaterialTable::new();
        assert_eq!(table.get(MaterialId(999)), PhysicsMaterial::DEFAULT);
        let err = table.require(MaterialId(999)).expect_err("unregistered");
        assert_eq!(err.domain(), Domain::Physics);
    }

    #[test]
    fn the_slipperier_surface_wins_the_contact() {
        let ice = PhysicsMaterial {
            friction: 0.02,
            ..PhysicsMaterial::DEFAULT
        };
        let rubber = PhysicsMaterial {
            friction: 1.0,
            ..PhysicsMaterial::DEFAULT
        };
        let contact = ice.contact_friction(rubber);
        // Nearer the ice than the average of the two would be.
        assert!(contact < (ice.friction + rubber.friction) / 2.0);
        assert!(contact > ice.friction);
    }

    #[test]
    fn the_bouncier_surface_wins_the_contact() {
        let dead = PhysicsMaterial::DEFAULT;
        let bouncy = PhysicsMaterial {
            restitution: 0.8,
            ..PhysicsMaterial::DEFAULT
        };
        assert!((dead.contact_restitution(bouncy) - 0.8).abs() < 1e-12);
        assert!((bouncy.contact_restitution(dead) - 0.8).abs() < 1e-12);
    }
}
