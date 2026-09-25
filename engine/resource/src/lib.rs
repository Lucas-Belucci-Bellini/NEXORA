//! # NEXORA resource system
//!
//! `RESOURCE AND ASSET SYSTEM.md`:
//!
//! ```text
//! ResourceID → Manifest → Resolver → Loader → Cache → Runtime Handle
//! ```
//!
//! *"Resources are addressable data or runtime assets. The Resource System
//! resolves, validates, loads, caches and unloads them without making gameplay
//! systems own asset lifetime."*
//!
//! | stage | here |
//! | --- | --- |
//! | ResourceID | [`nexora_foundation::ident::Identifier`] — the one way content is named |
//! | Manifest | [`manifest::Manifest`] — what exists, where, how large, with which hash |
//! | Resolver | [`manager::ResourceManager::resolve`] — an id and a kind become a handle, or an error |
//! | Loader | [`manager::Loader`] — bytes become a typed value |
//! | Cache | [`cache::ResourceCache`] — bounded in bytes, with priority, last use, pinning and telemetry |
//! | Runtime Handle | [`manager::ResourceHandle`] — typed, cheap, and never a path |
//!
//! # The invariants, and where each is enforced
//!
//! * **Runtime uses resource IDs, not source paths.** A path appears in the
//!   manifest and nowhere in the API. It is validated as untrusted input
//!   (`RESOURCE AND ASSET SYSTEM.md` *Security*): relative, no `..`, no root.
//! * **Cache data is disposable.** Every cached value can be dropped and
//!   loaded again from its manifest entry; a caller holding one keeps its own
//!   `Arc`, so eviction never pulls a value out from under a user.
//! * **Missing optional resources use declared fallbacks.** A manifest entry
//!   that is `optional` and names a `fallback` is replaced by it when it cannot
//!   be read; a required one is an error. Nothing is substituted silently: every
//!   substitution is counted.
//!
//! # What this crate does not do
//!
//! It does not decode any format. A loader turns bytes into a value; which
//! loaders exist is the business of the crate that knows the format. It does
//! not do hot-reload file watching — [`manager::ResourceManager::invalidate`] is
//! the operation a watcher would call.

pub mod cache;
pub mod manager;
pub mod manifest;

pub use cache::{CacheStats, Priority, ResourceCache};
pub use manager::{BytesLoader, Loader, ResourceHandle, ResourceManager};
pub use manifest::{Gap, Manifest, ManifestEntry, ResourceKind, MANIFEST_FILE};
