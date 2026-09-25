//! What every backend shares: resource tables, the command rules, and device
//! memory accounting.
//!
//! A rule written twice drifts. The null backend and every native backend
//! keep their resources in a [`Resources`] table and validate each command list
//! with [`Resources::check`], so a write that one backend refuses is refused by
//! all of them, by the same code. What differs between backends is only what
//! each resource carries next to its descriptor: nothing for the null backend,
//! a GPU object for a native one. That is the `N` in [`Resource`].

use std::sync::Arc;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::memory::MemoryPool;

use crate::api::{BufferHandle, Command, CommandList, PipelineHandle, Slot, TextureHandle};
use crate::desc::{refused, BufferDesc, PipelineDesc, TextureDesc, Usage, COPY_ALIGNMENT};

/// A live resource: its descriptor, what it costs, when it was last used, and
/// whatever the backend keeps for it.
#[derive(Debug)]
pub struct Resource<D, N> {
    /// What it was created as.
    pub desc: D,
    /// Device memory it holds.
    pub bytes: u64,
    /// The fence value of the last submission that used it, 0 for none.
    pub last_use: u64,
    /// The backend's own object, if it has one.
    pub native: N,
}

#[derive(Debug)]
struct Entry<T> {
    generation: u32,
    live: Option<T>,
}

/// One kind of resource: slots, generations and a free list.
#[derive(Debug)]
struct Table<T> {
    entries: Vec<Entry<T>>,
    free: Vec<u32>,
}

impl<T> Table<T> {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }

    fn insert(&mut self, value: T, epoch: u32) -> Slot {
        if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];
            entry.live = Some(value);
            return Slot {
                index,
                generation: entry.generation,
                epoch,
            };
        }
        let index = u32::try_from(self.entries.len()).expect("fewer than 2^32 resources");
        self.entries.push(Entry {
            generation: 0,
            live: Some(value),
        });
        Slot {
            index,
            generation: 0,
            epoch,
        }
    }

    fn get(&self, slot: Slot, epoch: u32) -> Option<&T> {
        if slot.epoch != epoch {
            return None;
        }
        let entry = self.entries.get(slot.index as usize)?;
        (entry.generation == slot.generation)
            .then_some(entry.live.as_ref())
            .flatten()
    }

    fn get_mut(&mut self, slot: Slot, epoch: u32) -> Option<&mut T> {
        if slot.epoch != epoch {
            return None;
        }
        let entry = self.entries.get_mut(slot.index as usize)?;
        if entry.generation != slot.generation {
            return None;
        }
        entry.live.as_mut()
    }

    /// Take the resource out and retire its slot: the next insert may reuse
    /// the index, under a new generation, so the old handle stays stale.
    fn remove(&mut self, slot: Slot, epoch: u32) -> Option<T> {
        self.get(slot, epoch)?;
        let entry = &mut self.entries[slot.index as usize];
        let live = entry.live.take();
        entry.generation = entry.generation.wrapping_add(1);
        self.free.push(slot.index);
        live
    }

    fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.live.is_some())
            .count()
    }
}

/// What [`Resources::check`] found in a list that passed.
#[derive(Debug, Clone, Default)]
pub struct Checked {
    uses: Vec<Use>,
    /// Bytes the list's writes carry.
    pub uploaded: u64,
    /// Draws in the list.
    pub draws: u64,
}

#[derive(Debug, Clone, Copy)]
enum Use {
    Buffer(Slot),
    Texture(Slot),
    Pipeline(Slot),
}

/// Every resource of one device life, keyed by handle.
///
/// `B`, `T` and `P` are what the backend keeps for a buffer, a texture and a
/// pipeline: `()` for the null backend, GPU objects for a native one.
#[derive(Debug)]
pub struct Resources<B, T, P> {
    epoch: u32,
    buffers: Table<Resource<BufferDesc, B>>,
    textures: Table<Resource<TextureDesc, T>>,
    pipelines: Table<Resource<PipelineDesc, P>>,
}

impl<B, T, P> Resources<B, T, P> {
    /// Empty tables for device life `epoch`.
    #[must_use]
    pub const fn new(epoch: u32) -> Self {
        Self {
            epoch,
            buffers: Table::new(),
            textures: Table::new(),
            pipelines: Table::new(),
        }
    }

    /// The device life these handles belong to.
    #[must_use]
    pub const fn epoch(&self) -> u32 {
        self.epoch
    }

    /// Live resources, as `(buffers, textures, pipelines)`.
    #[must_use]
    pub fn live(&self) -> (usize, usize, usize) {
        (
            self.buffers.len(),
            self.textures.len(),
            self.pipelines.len(),
        )
    }

    /// Add a buffer.
    pub fn insert_buffer(&mut self, desc: BufferDesc, native: B) -> BufferHandle {
        let bytes = desc.size;
        BufferHandle(
            self.buffers
                .insert(resource(desc, bytes, native), self.epoch),
        )
    }

    /// Add a texture.
    pub fn insert_texture(&mut self, desc: TextureDesc, native: T) -> TextureHandle {
        let bytes = desc.byte_len();
        TextureHandle(
            self.textures
                .insert(resource(desc, bytes, native), self.epoch),
        )
    }

    /// Add a pipeline. Pipelines are not counted as device memory.
    pub fn insert_pipeline(&mut self, desc: PipelineDesc, native: P) -> PipelineHandle {
        PipelineHandle(self.pipelines.insert(resource(desc, 0, native), self.epoch))
    }

    /// A live buffer.
    ///
    /// # Errors
    ///
    /// The handle is stale.
    pub fn buffer(&self, handle: BufferHandle) -> Result<&Resource<BufferDesc, B>> {
        self.buffers
            .get(handle.0, self.epoch)
            .ok_or_else(|| stale("buffer"))
    }

    /// A live texture.
    ///
    /// # Errors
    ///
    /// The handle is stale.
    pub fn texture(&self, handle: TextureHandle) -> Result<&Resource<TextureDesc, T>> {
        self.textures
            .get(handle.0, self.epoch)
            .ok_or_else(|| stale("texture"))
    }

    /// A live pipeline.
    ///
    /// # Errors
    ///
    /// The handle is stale.
    pub fn pipeline(&self, handle: PipelineHandle) -> Result<&Resource<PipelineDesc, P>> {
        self.pipelines
            .get(handle.0, self.epoch)
            .ok_or_else(|| stale("pipeline"))
    }

    /// Take a buffer out; its handle is stale from now on.
    ///
    /// # Errors
    ///
    /// The handle is already stale.
    pub fn remove_buffer(&mut self, handle: BufferHandle) -> Result<Resource<BufferDesc, B>> {
        self.buffers
            .remove(handle.0, self.epoch)
            .ok_or_else(|| stale("buffer"))
    }

    /// Take a texture out; its handle is stale from now on.
    ///
    /// # Errors
    ///
    /// The handle is already stale.
    pub fn remove_texture(&mut self, handle: TextureHandle) -> Result<Resource<TextureDesc, T>> {
        self.textures
            .remove(handle.0, self.epoch)
            .ok_or_else(|| stale("texture"))
    }

    /// Take a pipeline out; its handle is stale from now on.
    ///
    /// # Errors
    ///
    /// The handle is already stale.
    pub fn remove_pipeline(&mut self, handle: PipelineHandle) -> Result<Resource<PipelineDesc, P>> {
        self.pipelines
            .remove(handle.0, self.epoch)
            .ok_or_else(|| stale("pipeline"))
    }

    /// Validate every command in `list` against the rules on
    /// [`crate::Command`]. All or nothing: the first broken rule refuses the
    /// list, naming the command.
    ///
    /// # Errors
    ///
    /// A command names a stale handle or breaks a rule.
    pub fn check(&self, list: &CommandList) -> Result<Checked> {
        let mut checked = Checked::default();
        for (index, command) in list.commands.iter().enumerate() {
            self.check_one(command, &mut checked).map_err(|error| {
                error
                    .with_context("list", &list.label)
                    .with_context("command", index.to_string())
            })?;
        }
        Ok(checked)
    }

    /// Record that the resources a checked list uses are in flight until
    /// fence `value`.
    pub fn mark_used(&mut self, checked: &Checked, value: u64) {
        let epoch = self.epoch;
        for used in &checked.uses {
            let last_use = match *used {
                Use::Buffer(slot) => self.buffers.get_mut(slot, epoch).map(|r| &mut r.last_use),
                Use::Texture(slot) => self.textures.get_mut(slot, epoch).map(|r| &mut r.last_use),
                Use::Pipeline(slot) => self.pipelines.get_mut(slot, epoch).map(|r| &mut r.last_use),
            };
            if let Some(last_use) = last_use {
                *last_use = value;
            }
        }
    }

    fn check_one(&self, command: &Command, checked: &mut Checked) -> Result<()> {
        match command {
            Command::WriteBuffer {
                buffer,
                offset,
                data,
            } => {
                let live = self.buffer(*buffer)?;
                if !live.desc.usage.contains(Usage::COPY_DST) {
                    return Err(refused("a buffer write needs COPY_DST usage")
                        .with_context("label", &live.desc.label));
                }
                let len = data.len() as u64;
                let aligned = offset % COPY_ALIGNMENT == 0 && len % COPY_ALIGNMENT == 0;
                let inside = offset
                    .checked_add(len)
                    .is_some_and(|end| end <= live.desc.size);
                if len == 0 || !aligned || !inside {
                    return Err(refused(
                        "a buffer write must be non-empty, four-byte aligned and inside the buffer",
                    )
                    .with_context("label", &live.desc.label)
                    .with_context("offset", offset.to_string())
                    .with_context("len", len.to_string())
                    .with_context("size", live.desc.size.to_string()));
                }
                checked.uses.push(Use::Buffer(buffer.0));
                checked.uploaded += len;
            }
            Command::WriteTexture { texture, data } => {
                let live = self.texture(*texture)?;
                if !live.desc.usage.contains(Usage::COPY_DST) {
                    return Err(refused("a texture upload needs COPY_DST usage")
                        .with_context("label", &live.desc.label));
                }
                let expected = live.desc.byte_len();
                if data.len() as u64 != expected {
                    return Err(refused("a texture upload must carry every texel, exactly")
                        .with_context("label", &live.desc.label)
                        .with_context("expected", expected.to_string())
                        .with_context("got", data.len().to_string()));
                }
                checked.uses.push(Use::Texture(texture.0));
                checked.uploaded += expected;
            }
            Command::Draw {
                pipeline,
                buffer,
                target,
                vertices,
            } => {
                let pipe = self.pipeline(*pipeline)?;
                let vertex = self.buffer(*buffer)?;
                let out = self.texture(*target)?;
                if !vertex.desc.usage.contains(Usage::VERTEX) {
                    return Err(refused("a draw's vertex buffer needs VERTEX usage")
                        .with_context("label", &vertex.desc.label));
                }
                if !out.desc.usage.contains(Usage::RENDER_TARGET) {
                    return Err(refused("a draw's target needs RENDER_TARGET usage")
                        .with_context("label", &out.desc.label));
                }
                if !pipe.desc.targets.contains(&out.desc.format) {
                    return Err(
                        refused("the pipeline was not built for the target's format")
                            .with_context("pipeline", &pipe.desc.label)
                            .with_context("format", out.desc.format.as_str()),
                    );
                }
                let needed = u64::from(*vertices) * u64::from(pipe.desc.vertex_stride);
                if *vertices == 0 || needed > vertex.desc.size {
                    return Err(refused("a draw reads past the end of its vertex buffer")
                        .with_context("vertices", vertices.to_string())
                        .with_context("needed", needed.to_string())
                        .with_context("size", vertex.desc.size.to_string()));
                }
                checked.uses.extend([
                    Use::Pipeline(pipeline.0),
                    Use::Buffer(buffer.0),
                    Use::Texture(target.0),
                ]);
                checked.draws += 1;
            }
            Command::Marker(_) => {}
        }
        Ok(())
    }
}

const fn resource<D, N>(desc: D, bytes: u64, native: N) -> Resource<D, N> {
    Resource {
        desc,
        bytes,
        last_use: 0,
        native,
    }
}

/// The error for a handle that is destroyed, never created, or from a
/// device life that was lost.
#[must_use]
pub fn stale(kind: &'static str) -> Error {
    refused("the handle is stale: destroyed, never created, or from a lost device")
        .with_context("kind", kind)
}

/// The error every call returns while the device is lost.
#[must_use]
pub fn device_lost(backend: &'static str) -> Error {
    Error::new(Domain::Render, "rhi", "the device was lost")
        .with_recovery(Recovery::DisableSubsystem)
        .with_context("backend", backend)
}

/// The error `present` returns from a backend that has no surface.
#[must_use]
pub fn no_surface(backend: &'static str) -> Error {
    Error::new(Domain::Render, "rhi", "this backend has no surface")
        .with_recovery(Recovery::DisableSubsystem)
        .with_context("backend", backend)
}

/// A destroyed resource whose memory in-flight work still uses.
#[derive(Debug, Clone, Copy)]
struct Grave {
    bytes: u64,
    until: u64,
}

/// Device memory held, including destroyed resources still in flight, and the
/// owner's pool it is reported to (`MemoryClass::Gpu`, ADR-0024).
#[derive(Debug)]
pub struct Accounting {
    limit: u64,
    allocated: u64,
    completed: u64,
    graves: Vec<Grave>,
    pool: Option<Arc<MemoryPool>>,
}

impl Accounting {
    /// Nothing held, and at most `limit` bytes ever.
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            allocated: 0,
            completed: 0,
            graves: Vec::new(),
            pool: None,
        }
    }

    /// Bytes held now.
    #[must_use]
    pub const fn allocated(&self) -> u64 {
        self.allocated
    }

    /// Report to `pool` from now on. The pool is the owner's budget: a
    /// reservation it does not admit is refused, and counted there.
    pub fn attach(&mut self, pool: Arc<MemoryPool>) {
        pool.record(self.allocated);
        self.pool = Some(pool);
    }

    /// Reserve `bytes` for a new resource, or refuse it.
    ///
    /// # Errors
    ///
    /// The device limit or the owner's budget would be exceeded. `Retry`:
    /// memory may come back once something is destroyed.
    pub fn reserve(&mut self, label: &str, bytes: u64) -> Result<()> {
        let over_device = self
            .allocated
            .checked_add(bytes)
            .is_none_or(|total| total > self.limit);
        let over_budget = self.pool.as_ref().is_some_and(|pool| !pool.admits(bytes));
        if over_device || over_budget {
            if let Some(pool) = &self.pool {
                pool.refuse();
            }
            return Err(Error::new(Domain::Render, "rhi", "out of device memory")
                .with_recovery(Recovery::Retry)
                .with_context("label", label)
                .with_context("requested", bytes.to_string())
                .with_context("allocated", self.allocated.to_string())
                .with_context("limit", if over_device { "device" } else { "budget" }));
        }
        self.allocated += bytes;
        self.account();
        Ok(())
    }

    /// Free `bytes` now, or once fence `last_use` completes if work still
    /// uses them.
    pub fn release(&mut self, bytes: u64, last_use: u64) {
        if last_use > self.completed {
            self.graves.push(Grave {
                bytes,
                until: last_use,
            });
        } else {
            self.free(bytes);
        }
    }

    /// Fence `completed` and everything before it have finished: free what
    /// was waiting on them.
    pub fn retire(&mut self, completed: u64) {
        self.completed = self.completed.max(completed);
        let done = self.completed;
        let freed: u64 = self
            .graves
            .iter()
            .filter(|grave| grave.until <= done)
            .map(|grave| grave.bytes)
            .sum();
        self.graves.retain(|grave| grave.until > done);
        if freed > 0 {
            self.free(freed);
        }
    }

    /// Forget everything: the device life that held it is over.
    pub fn reset(&mut self) {
        self.allocated = 0;
        self.completed = 0;
        self.graves.clear();
        self.account();
    }

    fn free(&mut self, bytes: u64) {
        self.allocated -= bytes;
        self.account();
    }

    fn account(&self) {
        if let Some(pool) = &self.pool {
            pool.record(self.allocated);
        }
    }
}
