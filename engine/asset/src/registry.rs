//! The material registry.
//!
//! `Registry System.md` §70 is explicit that a specialised registry *"still
//! uses the generic infrastructure"*, and the Texture Forge brief §23 forbids
//! duplicating the Registry System. So this is not a second registry: it is
//! `nexora_runtime::registry::Registry<SurfaceMaterial>` with the rules that
//! are specific to materials, and nothing else.
//!
//! Everything the generic registry already guarantees is inherited unchanged —
//! duplicate identifiers are refused rather than overwritten, runtime ids are
//! session-local and never persisted, freezing closes registration, and the
//! fingerprint covers content rather than load order.
//!
//! # Why the handle is a `RuntimeId` and not a `SurfaceId`
//!
//! `nexora_mesh::mesh::SurfaceId` is what a face carries, and it is the handle
//! this whole system exists to give meaning to. But it lives in the mesher, and
//! an asset crate has no business depending on a mesher — the dependency would
//! run the wrong way and would say that materials are a rendering concept.
//!
//! Both are `u32` newtypes, and `nexora-simulation` is the one crate that can
//! see the registry and the mesher at once. So the conversion happens there,
//! in the open, exactly as `WorldSurfaces` already converts a block state into
//! a surface. The registry hands out a `RuntimeId`; the composition point
//! decides that it is also a `SurfaceId`.
//!
//! # What this registry deliberately does not remember
//!
//! Revisions. The brief §15 asks that materials be versioned and that changes
//! stay traceable, and they are — in the documents on disk and in each
//! material's [`crate::Provenance`]. A runtime registry that held every past
//! revision would be a database, which `Registry System.md` §4 says it is not.
//! One revision of a material is resolvable at a time, and which one is a
//! question the content pipeline answers before the engine starts.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_runtime::registry::{Registry, RegistryEntry, RuntimeId};

use crate::material::SurfaceMaterial;
use crate::provenance::AssetStatus;

/// The identifier of the material registry itself.
pub const MATERIAL_REGISTRY: &str = "nexora:registry/material";

/// Registered surface materials.
#[derive(Debug, Clone)]
pub struct MaterialRegistry {
    inner: Registry<SurfaceMaterial>,
}

impl MaterialRegistry {
    /// An empty material registry.
    ///
    /// # Errors
    ///
    /// Returns an error only if [`MATERIAL_REGISTRY`] stops being a valid
    /// identifier, which would be a build-time mistake rather than a runtime
    /// condition.
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Registry::new(Identifier::parse(MATERIAL_REGISTRY)?),
        })
    }

    /// Register a material under its own identifier.
    ///
    /// The material supplies the key, so a material cannot be filed under a
    /// name other than its own — a mismatch that would otherwise surface much
    /// later as a texture that resolves to the wrong surface.
    ///
    /// # Errors
    ///
    /// Returns an error when the registry is frozen, the identifier is already
    /// taken, or the material's asset status is [`AssetStatus::Blocked`].
    /// Blocked content is refused rather than registered-and-flagged: a blocked
    /// asset that is resolvable is a blocked asset that ships.
    pub fn register(&mut self, material: SurfaceMaterial) -> Result<RuntimeId> {
        if material.provenance().status == AssetStatus::Blocked {
            return Err(
                refused("a blocked asset must not become resolvable content")
                    .with_context("material", material.id().to_string()),
            );
        }
        let id = material.id().clone();
        self.inner.register(id, material)
    }

    /// Close registration.
    pub fn freeze(&mut self) {
        self.inner.freeze();
    }

    /// Whether registration is closed.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.inner.is_frozen()
    }

    /// How many materials are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Look up a material by identifier.
    #[must_use]
    pub fn get(&self, id: &Identifier) -> Option<&SurfaceMaterial> {
        self.inner.get(id).map(RegistryEntry::value)
    }

    /// Look up a material by its session handle.
    #[must_use]
    pub fn get_by_runtime_id(&self, runtime_id: RuntimeId) -> Option<&SurfaceMaterial> {
        self.inner
            .get_by_runtime_id(runtime_id)
            .map(RegistryEntry::value)
    }

    /// The session handle for an identifier, if registered.
    #[must_use]
    pub fn runtime_id_of(&self, id: &Identifier) -> Option<RuntimeId> {
        self.inner.runtime_id_of(id)
    }

    /// Resolve an identifier, failing loudly when it is absent.
    ///
    /// # Errors
    ///
    /// Returns the generic registry's "missing content" error, which names the
    /// registry and the identifier.
    pub fn require(&self, id: &Identifier) -> Result<&SurfaceMaterial> {
        self.inner.require(id).map(RegistryEntry::value)
    }

    /// Every registered material, in registration order.
    pub fn iter(&self) -> impl Iterator<Item = &SurfaceMaterial> {
        self.inner.iter().map(RegistryEntry::value)
    }

    /// Every registered identifier, sorted.
    #[must_use]
    pub fn identifiers(&self) -> Vec<Identifier> {
        self.inner.identifiers()
    }

    /// A content fingerprint, for handshakes and save validation.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        self.inner.fingerprint()
    }

    /// The registry this wraps, for callers that need the generic API.
    #[must_use]
    pub const fn inner(&self) -> &Registry<SurfaceMaterial> {
        &self.inner
    }

    /// Every registered material that must not enter a release artifact.
    ///
    /// The audit step from `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md`:
    /// *"all packaged assets → registry lookup → license check → attribution
    /// check → provenance check → package approval"*. Returns each blocked
    /// material with the reason, so a release check can print a list rather
    /// than a boolean.
    #[must_use]
    pub fn release_audit(&self) -> Vec<(Identifier, &'static str)> {
        self.iter()
            .filter_map(|material| {
                let provenance = material.provenance();
                let reason = if provenance.may_ship() {
                    return None;
                } else if provenance.status == AssetStatus::EditorOnly {
                    "editor-only content is never packaged"
                } else if provenance.release != crate::provenance::ReleaseStatus::Cleared {
                    "not cleared for release"
                } else {
                    "provenance class forbids release"
                };
                Some((material.id().clone(), reason))
            })
            .collect()
    }
}

fn refused(message: &'static str) -> Error {
    Error::new(Domain::Content, "material-registry", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MaterialCategory, SurfaceMaterial};
    use crate::provenance::{GenerationTrace, Provenance, ReleaseStatus, Timestamp};
    use crate::texture::Resolution;
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
                1,
            ),
        )
    }

    fn material(path: &str) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id(path),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            provenance(),
        )
        .build()
        .expect("valid material")
    }

    fn cleared(path: &str) -> SurfaceMaterial {
        let mut record = provenance();
        record.release = ReleaseStatus::Cleared;
        record.reviewer = Some("operator".to_owned());
        record.reviewed_at = Some(Timestamp(1_700_000_000));
        SurfaceMaterial::builder(
            id(path),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            record,
        )
        .build()
        .expect("valid material")
    }

    #[test]
    fn a_material_is_filed_under_its_own_identifier() {
        let mut registry = MaterialRegistry::new().unwrap();
        let handle = registry
            .register(material("nexora:material/stone"))
            .unwrap();

        assert_eq!(handle, RuntimeId(0));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert_eq!(
            registry
                .get(&id("nexora:material/stone"))
                .unwrap()
                .id()
                .to_string(),
            "nexora:material/stone"
        );
        assert_eq!(
            registry.get_by_runtime_id(handle).unwrap().id().path(),
            "material/stone"
        );
        assert_eq!(
            registry.runtime_id_of(&id("nexora:material/stone")),
            Some(handle)
        );
    }

    #[test]
    fn the_generic_registry_rules_are_inherited_unchanged() {
        let mut registry = MaterialRegistry::new().unwrap();
        registry
            .register(material("nexora:material/stone"))
            .unwrap();

        // Duplicate refused, original intact.
        let err = registry
            .register(material("nexora:material/stone"))
            .expect_err("duplicates are refused");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert_eq!(registry.len(), 1);

        // Missing content surfaces rather than defaulting.
        let err = registry
            .require(&id("example:material/ruby"))
            .expect_err("missing content must surface");
        assert_eq!(err.recovery(), Recovery::Quarantine);

        // Freezing closes registration.
        registry.freeze();
        assert!(registry.is_frozen());
        assert!(registry.register(material("nexora:material/dirt")).is_err());
    }

    #[test]
    fn a_blocked_asset_cannot_be_registered_at_all() {
        let mut record = provenance();
        record.status = AssetStatus::Blocked;
        let blocked = SurfaceMaterial::builder(
            id("nexora:material/leaked"),
            MaterialCategory::Custom,
            Resolution::square(16).unwrap(),
            record,
        )
        .build()
        .unwrap();

        let mut registry = MaterialRegistry::new().unwrap();
        let err = registry
            .register(blocked)
            .expect_err("blocked content must not be resolvable");
        assert!(err.to_string().contains("blocked"), "{err}");
        assert!(registry.is_empty());
    }

    #[test]
    fn the_release_audit_lists_what_may_not_ship_and_why() {
        let mut registry = MaterialRegistry::new().unwrap();
        registry.register(cleared("nexora:material/stone")).unwrap();
        registry
            .register(material("nexora:material/draft"))
            .unwrap();

        let mut editor_only = provenance();
        editor_only.status = AssetStatus::EditorOnly;
        editor_only.release = ReleaseStatus::Cleared;
        editor_only.reviewer = Some("operator".to_owned());
        editor_only.reviewed_at = Some(Timestamp(1));
        registry
            .register(
                SurfaceMaterial::builder(
                    id("nexora:material/grid"),
                    MaterialCategory::Custom,
                    Resolution::square(16).unwrap(),
                    editor_only,
                )
                .build()
                .unwrap(),
            )
            .unwrap();

        let audit = registry.release_audit();
        assert_eq!(audit.len(), 2, "{audit:?}");
        let reasons: Vec<(String, &str)> = audit
            .into_iter()
            .map(|(id, reason)| (id.to_string(), reason))
            .collect();
        assert!(reasons.contains(&(
            "nexora:material/draft".to_owned(),
            "not cleared for release"
        )));
        assert!(reasons.contains(&(
            "nexora:material/grid".to_owned(),
            "editor-only content is never packaged"
        )));
    }

    #[test]
    fn the_fingerprint_ignores_registration_order() {
        let paths = [
            "nexora:material/stone",
            "nexora:material/wood",
            "example:material/ruby",
        ];

        let mut forward = MaterialRegistry::new().unwrap();
        for path in paths {
            forward.register(material(path)).unwrap();
        }
        let mut reverse = MaterialRegistry::new().unwrap();
        for path in paths.iter().rev() {
            reverse.register(material(path)).unwrap();
        }

        assert_eq!(forward.fingerprint(), reverse.fingerprint());
        assert_eq!(forward.identifiers(), reverse.identifiers());
        // Runtime ids legitimately differ, which is why they are never stored.
        assert_ne!(
            forward.runtime_id_of(&id("nexora:material/stone")),
            reverse.runtime_id_of(&id("nexora:material/stone"))
        );
        assert_eq!(forward.iter().count(), 3);
        assert_eq!(forward.inner().name().to_string(), MATERIAL_REGISTRY);
    }
}
