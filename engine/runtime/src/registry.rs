//! Content registries.
//!
//! Implements `Registry System.md`. The central principle from §1 is that the
//! engine does not need to know what a "ruby ore" is - only that something is
//! registered under a namespaced id. Two decisions here follow directly from
//! that document and matter far beyond Phase 0:
//!
//! * **Runtime ids are session-local** (§11-§12). Persistence stores the
//!   namespaced [`Identifier`], never the integer, because the integer depends
//!   on which mods happened to load. Chunk palettes therefore serialize ids as
//!   text and remap on load.
//! * **The fingerprint covers content, not order** (§38-§39). Two peers with
//!   the same content in a different load order must agree, or every multiplayer
//!   handshake becomes a coin flip.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::Fnv1a64;
use nexora_foundation::ident::Identifier;

/// A session-local numeric handle for a registered entry.
///
/// Cheap to compare and copy inside one process. Never write one to disk or to
/// the network: see the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeId(pub u32);

/// One registered entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry<T> {
    id: Identifier,
    runtime_id: RuntimeId,
    value: T,
}

impl<T> RegistryEntry<T> {
    /// The stable namespaced identifier.
    #[must_use]
    pub const fn id(&self) -> &Identifier {
        &self.id
    }

    /// The session-local handle.
    #[must_use]
    pub const fn runtime_id(&self) -> RuntimeId {
        self.runtime_id
    }

    /// The registered value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// A typed registry of namespaced content.
#[derive(Debug, Clone)]
pub struct Registry<T> {
    name: Identifier,
    entries: Vec<RegistryEntry<T>>,
    by_id: BTreeMap<Identifier, usize>,
    frozen: bool,
}

impl<T> Registry<T> {
    /// Create an empty registry, e.g. `nexora:registry/block`.
    #[must_use]
    pub fn new(name: Identifier) -> Self {
        Self {
            name,
            entries: Vec::new(),
            by_id: BTreeMap::new(),
            frozen: false,
        }
    }

    /// The registry's own identifier.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// How many entries are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether registration has been closed.
    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Register a value.
    ///
    /// # Errors
    ///
    /// Returns an error when the registry is frozen or the id is already taken.
    /// Duplicate registration is refused rather than overwritten: silently
    /// replacing another owner's content is how mod conflicts become
    /// undiagnosable.
    pub fn register(&mut self, id: Identifier, value: T) -> Result<RuntimeId> {
        if self.frozen {
            return Err(self
                .error("registry is frozen and cannot accept new entries")
                .with_recovery(Recovery::Reject)
                .with_context("entry", id.to_string()));
        }
        if self.by_id.contains_key(&id) {
            return Err(self
                .error("an entry with this id is already registered")
                .with_recovery(Recovery::Reject)
                .with_context("entry", id.to_string()));
        }

        let runtime_id = RuntimeId(
            u32::try_from(self.entries.len())
                .map_err(|_| self.error("registry exceeded the runtime id space").fatal())?,
        );
        self.by_id.insert(id.clone(), self.entries.len());
        self.entries.push(RegistryEntry {
            id,
            runtime_id,
            value,
        });
        Ok(runtime_id)
    }

    /// Close the registry to further registration.
    ///
    /// Freezing is what makes runtime ids meaningful: after this point the id
    /// space cannot shift underneath a save, a network peer, or a chunk palette.
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// Look up an entry by its namespaced identifier.
    #[must_use]
    pub fn get(&self, id: &Identifier) -> Option<&RegistryEntry<T>> {
        self.by_id.get(id).map(|index| &self.entries[*index])
    }

    /// Look up an entry by its session-local handle.
    #[must_use]
    pub fn get_by_runtime_id(&self, runtime_id: RuntimeId) -> Option<&RegistryEntry<T>> {
        self.entries.get(runtime_id.0 as usize)
    }

    /// The handle for an identifier, if registered.
    #[must_use]
    pub fn runtime_id_of(&self, id: &Identifier) -> Option<RuntimeId> {
        self.get(id).map(RegistryEntry::runtime_id)
    }

    /// Resolve an identifier, failing loudly when it is absent.
    ///
    /// # Errors
    ///
    /// Returns an error naming the registry and the missing id. This is the
    /// "missing content" path from `Registry System.md` §41: unresolved
    /// references must surface, not silently become a default.
    pub fn require(&self, id: &Identifier) -> Result<&RegistryEntry<T>> {
        self.get(id).ok_or_else(|| {
            self.error("no entry is registered under this id")
                .with_recovery(Recovery::Quarantine)
                .with_context("entry", id.to_string())
        })
    }

    /// Iterate entries in registration order.
    pub fn iter(&self) -> impl Iterator<Item = &RegistryEntry<T>> {
        self.entries.iter()
    }

    /// Every registered identifier, sorted.
    #[must_use]
    pub fn identifiers(&self) -> Vec<Identifier> {
        self.by_id.keys().cloned().collect()
    }

    /// A content fingerprint for handshakes and save validation.
    ///
    /// Derived from the sorted identifier set only, so registration order and
    /// mod load order do not change it.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str(&self.name.to_string());
        hasher.write_u64(self.entries.len() as u64);
        for id in self.by_id.keys() {
            hasher.write_str(&id.to_string());
        }
        hasher.finish()
    }

    /// An ordered snapshot of `(identifier, runtime id)` for diagnostics.
    #[must_use]
    pub fn snapshot(&self) -> Vec<(Identifier, RuntimeId)> {
        let mut out: Vec<(Identifier, RuntimeId)> = self
            .entries
            .iter()
            .map(|entry| (entry.id.clone(), entry.runtime_id))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn error(&self, message: &'static str) -> Error {
        Error::new(Domain::Registry, "registry", message)
            .with_context("registry", self.name.to_string())
    }
}

/// Compare two registry fingerprints, as a client and server would on connect.
///
/// # Errors
///
/// Returns an error describing the mismatch when the fingerprints differ.
pub fn require_matching_fingerprint(local: u64, remote: u64, registry: &Identifier) -> Result<()> {
    if local == remote {
        return Ok(());
    }
    Err(Error::new(
        Domain::Registry,
        "handshake",
        "registry contents differ between peers",
    )
    .with_recovery(Recovery::Reject)
    .with_context("registry", registry.to_string())
    .with_context("local", format!("{local:#018x}"))
    .with_context("remote", format!("{remote:#018x}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn block_registry() -> Registry<&'static str> {
        Registry::new(id("nexora:registry/block"))
    }

    #[test]
    fn registration_assigns_sequential_runtime_ids() {
        let mut registry = block_registry();
        assert_eq!(
            registry.register(id("nexora:block/air"), "air").unwrap(),
            RuntimeId(0)
        );
        assert_eq!(
            registry
                .register(id("nexora:block/stone"), "stone")
                .unwrap(),
            RuntimeId(1)
        );
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());

        assert_eq!(
            registry.get(&id("nexora:block/stone")).unwrap().value(),
            &"stone"
        );
        assert_eq!(
            registry.get_by_runtime_id(RuntimeId(0)).unwrap().value(),
            &"air"
        );
        assert_eq!(
            registry.runtime_id_of(&id("nexora:block/air")),
            Some(RuntimeId(0))
        );
    }

    #[test]
    fn duplicate_ids_are_refused_not_overwritten() {
        let mut registry = block_registry();
        registry
            .register(id("nexora:block/stone"), "stone")
            .unwrap();
        let err = registry
            .register(id("nexora:block/stone"), "impostor")
            .expect_err("duplicate must be refused");
        assert_eq!(err.recovery(), Recovery::Reject);
        // The original survived.
        assert_eq!(
            registry.get(&id("nexora:block/stone")).unwrap().value(),
            &"stone"
        );
    }

    #[test]
    fn freezing_closes_registration() {
        let mut registry = block_registry();
        registry
            .register(id("nexora:block/stone"), "stone")
            .unwrap();
        registry.freeze();
        assert!(registry.is_frozen());

        let err = registry
            .register(id("nexora:block/dirt"), "dirt")
            .expect_err("frozen must refuse");
        assert!(err.to_string().contains("frozen"), "{err}");
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn missing_content_surfaces_as_a_quarantine_error() {
        let registry = block_registry();
        let err = registry
            .require(&id("example:block/ruby"))
            .expect_err("must not resolve");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(err.to_string().contains("example:block/ruby"), "{err}");
    }

    #[test]
    fn fingerprint_ignores_registration_order() {
        let ids = [
            "nexora:block/air",
            "nexora:block/stone",
            "example:block/ruby",
        ];

        let mut forward = block_registry();
        for raw in ids {
            forward.register(id(raw), raw).unwrap();
        }

        let mut reverse = block_registry();
        for raw in ids.iter().rev() {
            reverse.register(id(raw), raw).unwrap();
        }

        // Same content, different load order: peers must still agree.
        assert_eq!(forward.fingerprint(), reverse.fingerprint());
        require_matching_fingerprint(forward.fingerprint(), reverse.fingerprint(), forward.name())
            .unwrap();

        // Runtime ids, by contrast, legitimately differ - which is exactly why
        // they are never persisted.
        assert_ne!(
            forward.runtime_id_of(&id("nexora:block/air")),
            reverse.runtime_id_of(&id("nexora:block/air"))
        );
    }

    #[test]
    fn fingerprint_changes_when_content_changes() {
        let mut base = block_registry();
        base.register(id("nexora:block/stone"), "stone").unwrap();
        let before = base.fingerprint();

        base.register(id("example:block/ruby"), "ruby").unwrap();
        assert_ne!(base.fingerprint(), before);
    }

    #[test]
    fn a_fingerprint_mismatch_is_reported_with_both_sides() {
        let mut server = block_registry();
        server.register(id("nexora:block/stone"), "stone").unwrap();
        server.register(id("example:block/ruby"), "ruby").unwrap();

        let mut client = block_registry();
        client.register(id("nexora:block/stone"), "stone").unwrap();

        let err =
            require_matching_fingerprint(client.fingerprint(), server.fingerprint(), server.name())
                .expect_err("a client missing content must be told");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("nexora:registry/block"), "{err}");
    }

    #[test]
    fn snapshot_is_sorted_by_identifier() {
        let mut registry = block_registry();
        registry
            .register(id("nexora:block/stone"), "stone")
            .unwrap();
        registry.register(id("example:block/ruby"), "ruby").unwrap();
        registry.register(id("nexora:block/air"), "air").unwrap();

        let rendered: Vec<String> = registry
            .snapshot()
            .into_iter()
            .map(|(id, _)| id.to_string())
            .collect();
        assert_eq!(
            rendered,
            [
                "example:block/ruby",
                "nexora:block/air",
                "nexora:block/stone"
            ]
        );
        assert_eq!(registry.identifiers().len(), 3);
    }

    #[test]
    fn iteration_preserves_registration_order() {
        let mut registry = block_registry();
        for raw in ["nexora:block/zeta", "nexora:block/alpha"] {
            registry.register(id(raw), raw).unwrap();
        }
        let order: Vec<&str> = registry.iter().map(|entry| *entry.value()).collect();
        assert_eq!(order, ["nexora:block/zeta", "nexora:block/alpha"]);
    }
}
