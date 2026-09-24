//! The resource manager: resolve, load, cache, unload, invalidate.
//!
//! `RESOURCE AND ASSET SYSTEM.md` gives the API as
//!
//! ```ts
//! resolve<T>(id, type): ResourceHandle<T>;
//! load<T>(handle): Promise<Resource<T>>;
//! unload(id); invalidate(id);
//! ```
//!
//! and this is that API, synchronous: the job system decides what runs off
//! the tick thread, not the resource manager (`DEBT-0018` is the same
//! question for chunk generation, answered the same way).
//!
//! # Integrity before decoding
//!
//! A loader is handed bytes only after they have been read from inside the
//! root, counted against the manifest's size and hashed against its hash. A
//! file that is truncated, padded or edited never reaches a decoder: a decoder
//! is a parser of arbitrary bytes, which is exactly where untrusted content
//! does its damage (`NEXORA SECURITY THREAT MODEL.md`).

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::fnv1a64;
use nexora_foundation::ident::Identifier;

use crate::cache::{Priority, ResourceCache, Shared};
use crate::manifest::{missing, Manifest, ManifestEntry, ResourceKind, MANIFEST_FILE};

/// Turns a resource's bytes into a value.
pub trait Loader {
    /// What it produces.
    type Output: Send + Sync + 'static;

    /// The kind of resource it reads.
    fn kind(&self) -> ResourceKind;

    /// Build the value. `bytes` have already passed the integrity check.
    ///
    /// # Errors
    ///
    /// Returns an error when the bytes are not a valid resource of this kind.
    fn load(&self, id: &Identifier, bytes: &[u8]) -> Result<Self::Output>;

    /// What the value costs the cache, in bytes. Defaults to the file size.
    fn cost(&self, _value: &Self::Output, file_size: u64) -> u64 {
        file_size
    }
}

/// The trivial loader: the verified bytes themselves.
#[derive(Debug, Clone, Copy)]
pub struct BytesLoader(pub ResourceKind);

impl Loader for BytesLoader {
    type Output = Vec<u8>;

    fn kind(&self) -> ResourceKind {
        self.0
    }

    fn load(&self, _id: &Identifier, bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(bytes.to_vec())
    }
}

/// A resolved resource: an identifier the manifest declares, of the right kind.
///
/// Typed by what its loader produces, so a texture handle cannot be loaded as
/// a sound. Holds no path and no data; it is cheap to copy around and says
/// nothing about whether the resource is resident.
#[derive(Debug)]
pub struct ResourceHandle<T> {
    id: Identifier,
    kind: ResourceKind,
    _output: PhantomData<fn() -> T>,
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            kind: self.kind,
            _output: PhantomData,
        }
    }
}

impl<T> ResourceHandle<T> {
    /// The identifier it resolves.
    #[must_use]
    pub const fn id(&self) -> &Identifier {
        &self.id
    }

    /// The kind it was resolved as.
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }
}

/// Resolves, loads and caches resources declared by one manifest.
#[derive(Debug)]
pub struct ResourceManager {
    root: PathBuf,
    manifest: Manifest,
    cache: ResourceCache,
    fallbacks_used: u64,
    integrity_failures: u64,
}

impl ResourceManager {
    /// A manager over a root, with its manifest and a cache budget.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, manifest: Manifest, cache_bytes: u64) -> Self {
        Self {
            root: root.into(),
            manifest,
            cache: ResourceCache::new(cache_bytes),
            fallbacks_used: 0,
            integrity_failures: 0,
        }
    }

    /// Open the manifest at `<root>/resources.json`.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest is absent or does not pass
    /// [`Manifest::from_text`].
    pub fn open(root: impl Into<PathBuf>, cache_bytes: u64) -> Result<Self> {
        let root = root.into();
        let path = root.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&path).map_err(|cause| {
            unreadable("the resource manifest could not be read")
                .with_context("path", path.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        Ok(Self::new(root, Manifest::from_text(&text)?, cache_bytes))
    }

    /// The manifest.
    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The cache, for its statistics and pins.
    #[must_use]
    pub const fn cache(&self) -> &ResourceCache {
        &self.cache
    }

    /// Mutable access to the cache, to pin or unpin.
    pub fn cache_mut(&mut self) -> &mut ResourceCache {
        &mut self.cache
    }

    /// How many loads were served by a declared fallback.
    #[must_use]
    pub const fn fallbacks_used(&self) -> u64 {
        self.fallbacks_used
    }

    /// How many files failed their size or hash check.
    #[must_use]
    pub const fn integrity_failures(&self) -> u64 {
        self.integrity_failures
    }

    /// Resolve an identifier to a handle for a loader's output.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest does not declare the identifier, or
    /// declares it as another kind.
    pub fn resolve<L: Loader>(
        &self,
        id: &Identifier,
        loader: &L,
    ) -> Result<ResourceHandle<L::Output>> {
        let entry = self.manifest.get(id).ok_or_else(|| missing(id))?;
        if entry.kind != loader.kind() {
            return Err(
                unreadable("the resource is a different kind than was asked for")
                    .with_context("resource", id.to_string())
                    .with_context("declared", entry.kind.as_str())
                    .with_context("asked", loader.kind().as_str()),
            );
        }
        Ok(ResourceHandle {
            id: id.clone(),
            kind: entry.kind,
            _output: PhantomData,
        })
    }

    /// Load a resolved resource, from the cache when it is there.
    ///
    /// An optional resource that cannot be read is replaced by its declared
    /// fallback, and the substitution is counted. A required one is an error.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, fails its integrity check or
    /// does not load, and there is no fallback; or when the cache cannot make
    /// room for the result.
    pub fn load<L: Loader>(
        &mut self,
        handle: &ResourceHandle<L::Output>,
        loader: &L,
    ) -> Result<Arc<L::Output>> {
        self.load_with_priority(handle, loader, Priority::Normal)
    }

    /// [`ResourceManager::load`], with a cache priority.
    ///
    /// # Errors
    ///
    /// See [`ResourceManager::load`].
    pub fn load_with_priority<L: Loader>(
        &mut self,
        handle: &ResourceHandle<L::Output>,
        loader: &L,
        priority: Priority,
    ) -> Result<Arc<L::Output>> {
        if let Some(value) = self.cache.get(&handle.id) {
            return downcast::<L::Output>(value, &handle.id);
        }
        let entry = self
            .manifest
            .get(&handle.id)
            .ok_or_else(|| missing(&handle.id))?
            .clone();

        let loaded = self
            .read(&entry)
            .and_then(|bytes| loader.load(&entry.id, &bytes));
        let value = match (loaded, &entry.fallback) {
            (Ok(value), _) => value,
            (Err(cause), Some(fallback)) if entry.optional => {
                self.fallbacks_used += 1;
                let fallback_entry = self
                    .manifest
                    .get(fallback)
                    .ok_or_else(|| missing(fallback))?
                    .clone();
                // A fallback that also fails is a real error: it was declared
                // as the last resort, and the manifest requires it to exist.
                let bytes = self
                    .read(&fallback_entry)
                    .map_err(|error| error.with_source(cause))?;
                loader.load(&fallback_entry.id, &bytes)?
            }
            (Err(cause), _) => return Err(cause),
        };

        let cost = loader.cost(&value, entry.size);
        let shared: Arc<L::Output> = Arc::new(value);
        self.cache.insert(
            entry.id.clone(),
            Arc::clone(&shared) as Shared,
            cost,
            priority,
        )?;
        Ok(shared)
    }

    /// Drop a resource from the cache. Holders keep what they hold.
    pub fn unload(&mut self, id: &Identifier) -> bool {
        self.cache.remove(id)
    }

    /// Drop a resource so the next load reads its file again.
    ///
    /// The operation a development file watcher calls for hot reload. It is
    /// [`ResourceManager::unload`] today because nothing is derived from a
    /// resource that would also need discarding; it stays a separate name so
    /// that when something is, callers do not have to change.
    pub fn invalidate(&mut self, id: &Identifier) -> bool {
        self.unload(id)
    }

    fn read(&mut self, entry: &ManifestEntry) -> Result<Vec<u8>> {
        let path = self.path_of(entry);
        let bytes = std::fs::read(&path).map_err(|cause| {
            unreadable("the resource file could not be read")
                .with_context("resource", entry.id.to_string())
                .with_context("cause", cause.to_string())
        })?;
        if bytes.len() as u64 != entry.size {
            self.integrity_failures += 1;
            return Err(
                corrupt("the resource file is not the size the manifest declares")
                    .with_context("resource", entry.id.to_string())
                    .with_context("declared", entry.size.to_string())
                    .with_context("found", bytes.len().to_string()),
            );
        }
        let hash = fnv1a64(&bytes);
        if hash != entry.hash {
            self.integrity_failures += 1;
            return Err(
                corrupt("the resource file does not hash to what the manifest declares")
                    .with_context("resource", entry.id.to_string())
                    .with_context("declared", format!("{:#018x}", entry.hash))
                    .with_context("found", format!("{hash:#018x}")),
            );
        }
        Ok(bytes)
    }

    fn path_of(&self, entry: &ManifestEntry) -> PathBuf {
        // The manifest has already refused anything but plain segments.
        let mut path = self.root.clone();
        for segment in Path::new(&entry.path).components() {
            path.push(segment);
        }
        path
    }
}

fn downcast<T: Send + Sync + 'static>(value: Shared, id: &Identifier) -> Result<Arc<T>> {
    value.downcast::<T>().map_err(|_| {
        // Two loaders of the same kind producing different types for one id.
        unreadable("the cached resource was loaded as a different type")
            .with_context("resource", id.to_string())
    })
}

fn unreadable(message: &'static str) -> Error {
    Error::new(Domain::Content, "resource", message).with_recovery(Recovery::Reject)
}

fn corrupt(message: &'static str) -> Error {
    Error::new(Domain::Content, "resource", message).with_recovery(Recovery::Quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nexora-resource-{}-{name}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    /// Write a file and the manifest entry that describes it.
    fn put(root: &Path, raw: &str, kind: ResourceKind, bytes: &[u8]) -> ManifestEntry {
        let path = format!("{}.bin", raw.replace([':', '/'], "_"));
        std::fs::write(root.join(&path), bytes).unwrap();
        ManifestEntry {
            id: id(raw),
            kind,
            path,
            size: bytes.len() as u64,
            hash: fnv1a64(bytes),
            dependencies: Vec::new(),
            optional: false,
            fallback: None,
        }
    }

    /// A loader that parses a little-endian u32, and nothing else.
    struct Number;
    impl Loader for Number {
        type Output = u32;
        fn kind(&self) -> ResourceKind {
            ResourceKind::Data
        }
        fn load(&self, id: &Identifier, bytes: &[u8]) -> Result<u32> {
            let raw: [u8; 4] = bytes.try_into().map_err(|_| {
                unreadable("not four bytes").with_context("resource", id.to_string())
            })?;
            Ok(u32::from_le_bytes(raw))
        }
    }

    #[test]
    fn a_resource_is_resolved_loaded_and_then_served_from_the_cache() {
        let root = scratch("load");
        let entry = put(
            &root,
            "nexora:data/seven",
            ResourceKind::Data,
            &7u32.to_le_bytes(),
        );
        let manifest = Manifest::new([entry]).unwrap();
        std::fs::write(root.join(MANIFEST_FILE), manifest.to_text()).unwrap();

        let mut resources = ResourceManager::open(&root, 1024).expect("opens");
        let handle = resources
            .resolve(&id("nexora:data/seven"), &Number)
            .unwrap();
        assert_eq!(*resources.load(&handle, &Number).unwrap(), 7);
        assert_eq!(*resources.load(&handle, &Number).unwrap(), 7);
        assert_eq!(
            resources.cache().stats().hits,
            1,
            "the second load is a hit"
        );
        assert_eq!(resources.cache().stats().bytes, 4);

        // Unloaded, the next load reads the file again.
        assert!(resources.unload(&id("nexora:data/seven")));
        assert_eq!(*resources.load(&handle, &Number).unwrap(), 7);
        assert_eq!(resources.cache().stats().inserts, 2);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_wrong_kind_and_an_undeclared_id_do_not_resolve() {
        let root = scratch("resolve");
        let entry = put(&root, "nexora:texture/x", ResourceKind::Texture, b"pixels");
        let resources = ResourceManager::new(&root, Manifest::new([entry]).unwrap(), 1024);

        let err = resources
            .resolve(&id("nexora:texture/x"), &Number)
            .expect_err("kind");
        assert!(err.to_string().contains("different kind"), "{err}");
        let err = resources
            .resolve(&id("nexora:data/absent"), &Number)
            .expect_err("undeclared");
        assert!(err.to_string().contains("no resource is declared"), "{err}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_tampered_file_never_reaches_the_loader() {
        let root = scratch("tamper");
        let entry = put(
            &root,
            "nexora:data/seven",
            ResourceKind::Data,
            &7u32.to_le_bytes(),
        );
        let path = root.join(&entry.path);
        let mut resources = ResourceManager::new(&root, Manifest::new([entry]).unwrap(), 1024);
        let handle = resources
            .resolve(&id("nexora:data/seven"), &Number)
            .unwrap();

        // Same size, different bytes: only the hash can tell.
        std::fs::write(&path, 8u32.to_le_bytes()).unwrap();
        let err = resources.load(&handle, &Number).expect_err("edited");
        assert!(err.to_string().contains("hash"), "{err}");
        assert_eq!(err.recovery(), Recovery::Quarantine);

        // Truncated.
        std::fs::write(&path, [7u8, 0]).unwrap();
        let err = resources.load(&handle, &Number).expect_err("truncated");
        assert!(err.to_string().contains("size"), "{err}");
        assert_eq!(resources.integrity_failures(), 2);
        assert_eq!(
            resources.cache().stats().inserts,
            0,
            "nothing bad was cached"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_optional_resource_falls_back_and_a_required_one_does_not() {
        let root = scratch("fallback");
        let fallback = put(
            &root,
            "nexora:data/default",
            ResourceKind::Data,
            &1u32.to_le_bytes(),
        );
        let mut optional = put(
            &root,
            "nexora:data/detail",
            ResourceKind::Data,
            &9u32.to_le_bytes(),
        );
        optional.optional = true;
        optional.fallback = Some(id("nexora:data/default"));
        let required = put(
            &root,
            "nexora:data/core",
            ResourceKind::Data,
            &5u32.to_le_bytes(),
        );
        std::fs::remove_file(root.join(&optional.path)).unwrap();
        std::fs::remove_file(root.join(&required.path)).unwrap();

        let mut resources = ResourceManager::new(
            &root,
            Manifest::new([fallback, optional, required]).unwrap(),
            1024,
        );
        let detail = resources
            .resolve(&id("nexora:data/detail"), &Number)
            .unwrap();
        assert_eq!(
            *resources.load(&detail, &Number).unwrap(),
            1,
            "the declared fallback"
        );
        assert_eq!(resources.fallbacks_used(), 1, "and it is counted");

        let core = resources.resolve(&id("nexora:data/core"), &Number).unwrap();
        let err = resources.load(&core, &Number).expect_err("required");
        assert!(err.to_string().contains("could not be read"), "{err}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_loader_that_rejects_the_bytes_is_an_error_not_a_value() {
        let root = scratch("reject");
        let entry = put(&root, "nexora:data/odd", ResourceKind::Data, b"abc");
        let mut resources = ResourceManager::new(&root, Manifest::new([entry]).unwrap(), 1024);
        let handle = resources.resolve(&id("nexora:data/odd"), &Number).unwrap();
        let err = resources.load(&handle, &Number).expect_err("three bytes");
        assert!(err.to_string().contains("four bytes"), "{err}");

        // The bytes loader takes anything that passes integrity.
        let bytes = BytesLoader(ResourceKind::Data);
        let handle = resources.resolve(&id("nexora:data/odd"), &bytes).unwrap();
        assert_eq!(resources.load(&handle, &bytes).unwrap().as_slice(), b"abc");
        let _ = std::fs::remove_dir_all(&root);
    }
}
