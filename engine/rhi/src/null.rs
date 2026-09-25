//! The null backend: the whole contract, and no GPU.
//!
//! `RENDER HARDWARE INTERFACE.md`, *Headless*: *"Server builds can use a
//! null/headless backend without initializing a GPU."* This is that backend,
//! and it is not a stub. It keeps every resource table, enforces every rule on
//! [`crate::Rhi`], issues and retires fences in order, defers the destruction
//! of resources that in-flight work still uses, accounts device memory, and
//! can lose its device on request. What it does not do is draw: no pixel is
//! ever computed, and its capabilities say it cannot present.
//!
//! Its GPU finishes **one submission per [`crate::Rhi::poll`]**. A backend
//! that finished everything at submission would make "in flight" impossible to
//! observe, and deferred destruction, the rule most likely to be wrong in a
//! real backend, untestable.

use std::sync::Arc;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::memory::MemoryPool;

use crate::api::{
    BufferHandle, Command, CommandList, Fence, PipelineHandle, Rhi, Slot, TextureHandle,
};
use crate::desc::{
    check_buffer, check_pipeline, check_texture, refused, BufferDesc, Capabilities, PipelineDesc,
    TextureDesc, TextureFormat, Usage, COPY_ALIGNMENT,
};

/// Default device memory: enough for any test, small enough to hit on purpose.
const DEFAULT_MEMORY: u64 = 256 * 1024 * 1024;

/// What the null backend was asked to do, for reports and tests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullStats {
    /// Command lists accepted.
    pub submissions: u64,
    /// Command lists refused whole.
    pub refused_submissions: u64,
    /// Commands in accepted lists.
    pub commands: u64,
    /// Bytes carried by accepted buffer and texture writes.
    pub bytes_uploaded: u64,
    /// Accepted draws.
    pub draws: u64,
    /// Device losses, injected or otherwise.
    pub device_losses: u64,
}

#[derive(Debug)]
struct Entry<T> {
    generation: u32,
    live: Option<Live<T>>,
}

#[derive(Debug)]
struct Live<T> {
    desc: T,
    bytes: u64,
    /// The fence value of the last submission that used it, 0 for none.
    last_use: u64,
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

    fn insert(&mut self, desc: T, bytes: u64, epoch: u32) -> Slot {
        let live = Live {
            desc,
            bytes,
            last_use: 0,
        };
        if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];
            entry.live = Some(live);
            return Slot {
                index,
                generation: entry.generation,
                epoch,
            };
        }
        let index = u32::try_from(self.entries.len()).expect("fewer than 2^32 resources");
        self.entries.push(Entry {
            generation: 0,
            live: Some(live),
        });
        Slot {
            index,
            generation: 0,
            epoch,
        }
    }

    fn get(&self, slot: Slot, epoch: u32) -> Option<&Live<T>> {
        if slot.epoch != epoch {
            return None;
        }
        let entry = self.entries.get(slot.index as usize)?;
        (entry.generation == slot.generation)
            .then_some(entry.live.as_ref())
            .flatten()
    }

    fn get_mut(&mut self, slot: Slot, epoch: u32) -> Option<&mut Live<T>> {
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
    fn remove(&mut self, slot: Slot, epoch: u32) -> Option<Live<T>> {
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

/// A destroyed resource whose memory in-flight work still uses.
#[derive(Debug, Clone, Copy)]
struct Grave {
    bytes: u64,
    until: u64,
}

/// The null backend. See the module documentation.
#[derive(Debug)]
pub struct NullRhi {
    caps: Capabilities,
    epoch: u32,
    lost: bool,
    buffers: Table<BufferDesc>,
    textures: Table<TextureDesc>,
    pipelines: Table<PipelineDesc>,
    issued: u64,
    completed: u64,
    graves: Vec<Grave>,
    allocated: u64,
    pool: Option<Arc<MemoryPool>>,
    stats: NullStats,
    /// Free destroyed resources at once, in-flight or not: the bug the
    /// conformance suite must catch, for the test that proves it does.
    #[cfg(test)]
    pub(crate) eager_free: bool,
}

impl Default for NullRhi {
    fn default() -> Self {
        Self::new()
    }
}

impl NullRhi {
    /// A null device with every format and generous limits.
    #[must_use]
    pub fn new() -> Self {
        Self::with_memory(DEFAULT_MEMORY)
    }

    /// A null device that hands out at most `bytes` of device memory.
    #[must_use]
    pub fn with_memory(bytes: u64) -> Self {
        Self {
            caps: Capabilities {
                backend: "null",
                device: "no device".into(),
                max_texture_edge: 8192,
                max_buffer_bytes: bytes,
                memory_bytes: bytes,
                formats: TextureFormat::ALL.to_vec(),
                presents: false,
                validates_shaders: false,
            },
            epoch: 0,
            lost: false,
            buffers: Table::new(),
            textures: Table::new(),
            pipelines: Table::new(),
            issued: 0,
            completed: 0,
            graves: Vec::new(),
            allocated: 0,
            pool: None,
            stats: NullStats::default(),
            #[cfg(test)]
            eager_free: false,
        }
    }

    /// Account device memory in `pool` (`MemoryClass::Gpu`, ADR-0024).
    ///
    /// The pool is the owner's budget: a create it does not admit is refused
    /// and counted there, before the device's own limit is reached.
    pub fn attach(&mut self, pool: Arc<MemoryPool>) {
        pool.record(self.allocated);
        self.pool = Some(pool);
    }

    /// Lose the device, as a driver reset or a removed GPU would. Every call
    /// then fails until [`Rhi::recreate`]. For tests and fault drills.
    pub fn lose_device(&mut self) {
        if !self.lost {
            self.lost = true;
            self.stats.device_losses += 1;
        }
    }

    /// What the backend was asked to do so far, across device lives.
    #[must_use]
    pub const fn stats(&self) -> NullStats {
        self.stats
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

    fn alive(&self) -> Result<()> {
        if self.lost {
            return Err(Error::new(Domain::Render, "rhi", "the device was lost")
                .with_recovery(Recovery::DisableSubsystem)
                .with_context("backend", self.caps.backend));
        }
        Ok(())
    }

    fn account(&self) {
        if let Some(pool) = &self.pool {
            pool.record(self.allocated);
        }
    }

    /// Reserve device memory for a new resource, or refuse it.
    fn reserve(&mut self, label: &str, bytes: u64) -> Result<()> {
        let over_device = self
            .allocated
            .checked_add(bytes)
            .is_none_or(|total| total > self.caps.memory_bytes);
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

    /// Free now, or when the last submission that used it completes.
    fn release(&mut self, bytes: u64, last_use: u64) {
        #[cfg(test)]
        let last_use = if self.eager_free { 0 } else { last_use };
        if last_use > self.completed {
            self.graves.push(Grave {
                bytes,
                until: last_use,
            });
        } else {
            self.allocated -= bytes;
            self.account();
        }
    }

    /// Free the memory of destroyed resources whose last use has completed.
    fn bury_completed(&mut self) {
        let completed = self.completed;
        let freed: u64 = self
            .graves
            .iter()
            .filter(|grave| grave.until <= completed)
            .map(|grave| grave.bytes)
            .sum();
        self.graves.retain(|grave| grave.until > completed);
        if freed > 0 {
            self.allocated -= freed;
            self.account();
        }
    }

    /// Check one command and say which resources it uses and what it uploads.
    fn check(&self, command: &Command) -> Result<(Vec<Use>, u64)> {
        let epoch = self.epoch;
        match command {
            Command::WriteBuffer {
                buffer,
                offset,
                data,
            } => {
                let live = self
                    .buffers
                    .get(buffer.0, epoch)
                    .ok_or_else(|| stale("buffer"))?;
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
                Ok((vec![Use::Buffer(buffer.0)], len))
            }
            Command::WriteTexture { texture, data } => {
                let live = self
                    .textures
                    .get(texture.0, epoch)
                    .ok_or_else(|| stale("texture"))?;
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
                Ok((vec![Use::Texture(texture.0)], expected))
            }
            Command::Draw {
                pipeline,
                buffer,
                target,
                vertices,
            } => {
                let pipe = self
                    .pipelines
                    .get(pipeline.0, epoch)
                    .ok_or_else(|| stale("pipeline"))?;
                let vertex = self
                    .buffers
                    .get(buffer.0, epoch)
                    .ok_or_else(|| stale("buffer"))?;
                let out = self
                    .textures
                    .get(target.0, epoch)
                    .ok_or_else(|| stale("texture"))?;
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
                Ok((
                    vec![
                        Use::Pipeline(pipeline.0),
                        Use::Buffer(buffer.0),
                        Use::Texture(target.0),
                    ],
                    0,
                ))
            }
            Command::Marker(_) => Ok((Vec::new(), 0)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Use {
    Buffer(Slot),
    Texture(Slot),
    Pipeline(Slot),
}

fn stale(kind: &'static str) -> Error {
    refused("the handle is stale: destroyed, never created, or from a lost device")
        .with_context("kind", kind)
}

impl Rhi for NullRhi {
    fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    fn create_buffer(&mut self, desc: &BufferDesc) -> Result<BufferHandle> {
        self.alive()?;
        check_buffer(desc, &self.caps)?;
        self.reserve(&desc.label, desc.size)?;
        Ok(BufferHandle(self.buffers.insert(
            desc.clone(),
            desc.size,
            self.epoch,
        )))
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle> {
        self.alive()?;
        check_texture(desc, &self.caps)?;
        let bytes = desc.byte_len();
        self.reserve(&desc.label, bytes)?;
        Ok(TextureHandle(self.textures.insert(
            desc.clone(),
            bytes,
            self.epoch,
        )))
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle> {
        self.alive()?;
        check_pipeline(desc, &self.caps)?;
        Ok(PipelineHandle(self.pipelines.insert(
            desc.clone(),
            0,
            self.epoch,
        )))
    }

    fn destroy_buffer(&mut self, buffer: BufferHandle) -> Result<()> {
        self.alive()?;
        let live = self
            .buffers
            .remove(buffer.0, self.epoch)
            .ok_or_else(|| stale("buffer"))?;
        self.release(live.bytes, live.last_use);
        Ok(())
    }

    fn destroy_texture(&mut self, texture: TextureHandle) -> Result<()> {
        self.alive()?;
        let live = self
            .textures
            .remove(texture.0, self.epoch)
            .ok_or_else(|| stale("texture"))?;
        self.release(live.bytes, live.last_use);
        Ok(())
    }

    fn destroy_pipeline(&mut self, pipeline: PipelineHandle) -> Result<()> {
        self.alive()?;
        let live = self
            .pipelines
            .remove(pipeline.0, self.epoch)
            .ok_or_else(|| stale("pipeline"))?;
        self.release(live.bytes, live.last_use);
        Ok(())
    }

    fn submit(&mut self, list: CommandList) -> Result<Fence> {
        self.alive()?;
        let mut uses = Vec::new();
        let mut uploaded = 0;
        let mut draws = 0;
        for (index, command) in list.commands.iter().enumerate() {
            match self.check(command) {
                Ok((used, bytes)) => {
                    uses.extend(used);
                    uploaded += bytes;
                    draws += u64::from(matches!(command, Command::Draw { .. }));
                }
                Err(error) => {
                    self.stats.refused_submissions += 1;
                    return Err(error
                        .with_context("list", &list.label)
                        .with_context("command", index.to_string()));
                }
            }
        }
        self.issued += 1;
        let value = self.issued;
        let epoch = self.epoch;
        for used in uses {
            let last_use = match used {
                Use::Buffer(slot) => self.buffers.get_mut(slot, epoch).map(|l| &mut l.last_use),
                Use::Texture(slot) => self.textures.get_mut(slot, epoch).map(|l| &mut l.last_use),
                Use::Pipeline(slot) => self.pipelines.get_mut(slot, epoch).map(|l| &mut l.last_use),
            };
            if let Some(last_use) = last_use {
                *last_use = value;
            }
        }
        self.stats.submissions += 1;
        self.stats.commands += list.commands.len() as u64;
        self.stats.bytes_uploaded += uploaded;
        self.stats.draws += draws;
        Ok(Fence { epoch, value })
    }

    fn poll(&mut self) -> Result<Option<Fence>> {
        self.alive()?;
        if self.completed < self.issued {
            self.completed += 1;
            self.bury_completed();
        }
        Ok((self.completed > 0).then_some(Fence {
            epoch: self.epoch,
            value: self.completed,
        }))
    }

    fn is_complete(&self, fence: Fence) -> bool {
        fence.epoch == self.epoch && fence.value <= self.completed
    }

    fn wait(&mut self, fence: Fence) -> Result<()> {
        self.alive()?;
        if fence.epoch != self.epoch || fence.value == 0 || fence.value > self.issued {
            return Err(refused("the fence was never issued by this device life")
                .with_context("fence", fence.value.to_string())
                .with_context("issued", self.issued.to_string()));
        }
        while self.completed < fence.value {
            self.poll()?;
        }
        Ok(())
    }

    fn present(&mut self, texture: TextureHandle) -> Result<()> {
        self.alive()?;
        let live = self
            .textures
            .get(texture.0, self.epoch)
            .ok_or_else(|| stale("texture"))?;
        if !live.desc.usage.contains(Usage::RENDER_TARGET) {
            return Err(refused("only a render target can be presented")
                .with_context("label", &live.desc.label));
        }
        Err(
            Error::new(Domain::Render, "rhi", "the null backend has no surface")
                .with_recovery(Recovery::DisableSubsystem)
                .with_context("backend", self.caps.backend),
        )
    }

    fn device_lost(&self) -> bool {
        self.lost
    }

    fn recreate(&mut self) -> Result<()> {
        if !self.lost {
            return Ok(());
        }
        self.epoch = self.epoch.wrapping_add(1);
        self.lost = false;
        self.buffers = Table::new();
        self.textures = Table::new();
        self.pipelines = Table::new();
        self.issued = 0;
        self.completed = 0;
        self.graves.clear();
        self.allocated = 0;
        self.account();
        Ok(())
    }

    fn allocated_bytes(&self) -> u64 {
        self.allocated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desc::ShaderStage;
    use nexora_foundation::memory::{MemoryBudget, MemoryClass, MemoryLedger, PoolSpec};

    fn buffer(size: u64, usage: Usage) -> BufferDesc {
        BufferDesc {
            label: "b".into(),
            size,
            usage,
        }
    }

    fn texture(edge: u32, usage: Usage) -> TextureDesc {
        TextureDesc {
            label: "t".into(),
            width: edge,
            height: edge,
            format: TextureFormat::Rgba8Unorm,
            usage,
        }
    }

    fn write(buffer: BufferHandle, offset: u64, len: usize) -> CommandList {
        let mut list = CommandList::new("write");
        list.push(Command::WriteBuffer {
            buffer,
            offset,
            data: vec![7; len],
        });
        list
    }

    #[test]
    fn a_destroyed_handle_is_stale_even_when_its_slot_is_reused() {
        let mut rhi = NullRhi::new();
        let first = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        rhi.destroy_buffer(first).unwrap();
        let second = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        assert_eq!(first.0.index, second.0.index, "the slot is reused");
        assert_ne!(first, second, "under a new generation");
        assert!(rhi.submit(write(first, 0, 4)).is_err());
        assert!(rhi.submit(write(second, 0, 4)).is_ok());
        assert!(rhi.destroy_buffer(first).is_err(), "no double destroy");
    }

    #[test]
    fn one_bad_command_refuses_the_whole_list() {
        let mut rhi = NullRhi::new();
        let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        let mut list = write(buf, 0, 4);
        list.push(Command::WriteBuffer {
            buffer: buf,
            offset: 16,
            data: vec![0; 4],
        });
        let error = rhi.submit(list).expect_err("the second write is outside");
        assert!(error
            .context()
            .iter()
            .any(|(k, v)| *k == "command" && v == "1"));
        let stats = rhi.stats();
        assert_eq!((stats.submissions, stats.refused_submissions), (0, 1));
        assert_eq!(stats.bytes_uploaded, 0, "nothing of it ran");
    }

    #[test]
    fn memory_in_use_by_the_gpu_outlives_its_handle_until_the_fence() {
        let mut rhi = NullRhi::new();
        let buf = rhi.create_buffer(&buffer(64, Usage::COPY_DST)).unwrap();
        let fence = rhi.submit(write(buf, 0, 64)).unwrap();
        rhi.destroy_buffer(buf).unwrap();
        assert_eq!(rhi.live().0, 0, "the handle is gone at once");
        assert_eq!(rhi.allocated_bytes(), 64, "the memory is not");
        assert!(!rhi.is_complete(fence));
        rhi.wait(fence).unwrap();
        assert_eq!(rhi.allocated_bytes(), 0);
    }

    #[test]
    fn a_resource_never_submitted_is_freed_at_once() {
        let mut rhi = NullRhi::new();
        let tex = rhi.create_texture(&texture(16, Usage::SAMPLED)).unwrap();
        assert_eq!(rhi.allocated_bytes(), 16 * 16 * 4);
        rhi.destroy_texture(tex).unwrap();
        assert_eq!(rhi.allocated_bytes(), 0);
    }

    #[test]
    fn fences_complete_one_per_poll_and_in_order() {
        let mut rhi = NullRhi::new();
        let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        let a = rhi.submit(write(buf, 0, 4)).unwrap();
        let b = rhi.submit(write(buf, 4, 4)).unwrap();
        assert!(a < b);
        assert_eq!(rhi.poll().unwrap(), Some(a));
        assert!(rhi.is_complete(a) && !rhi.is_complete(b));
        assert_eq!(rhi.poll().unwrap(), Some(b));
        assert_eq!(rhi.poll().unwrap(), Some(b), "nothing left to retire");
    }

    #[test]
    fn a_lost_device_refuses_everything_until_recreated_and_old_handles_stay_dead() {
        let mut rhi = NullRhi::new();
        let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        let fence = rhi.submit(write(buf, 0, 4)).unwrap();
        rhi.lose_device();
        assert!(rhi.device_lost());
        let error = rhi.submit(write(buf, 0, 4)).expect_err("lost");
        assert_eq!(error.recovery(), Recovery::DisableSubsystem);
        assert!(rhi.create_buffer(&buffer(16, Usage::COPY_DST)).is_err());
        assert!(rhi.poll().is_err());

        rhi.recreate().unwrap();
        assert!(!rhi.device_lost());
        assert_eq!(rhi.allocated_bytes(), 0, "the old life's memory is gone");
        assert!(
            rhi.submit(write(buf, 0, 4)).is_err(),
            "old handle, new life"
        );
        assert!(rhi.wait(fence).is_err(), "old fence, new life");
        assert!(!rhi.is_complete(fence));
        let fresh = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        assert_eq!(fresh.0.index, buf.0.index, "same slot, different life");
        assert_eq!(
            fresh.0.generation, buf.0.generation,
            "even the same generation"
        );
        assert!(
            rhi.submit(write(buf, 0, 4)).is_err(),
            "only the epoch tells them apart, and it must"
        );
        assert!(rhi.submit(write(fresh, 0, 4)).is_ok());
        assert_eq!(rhi.stats().device_losses, 1);
    }

    #[test]
    fn recreating_a_healthy_device_changes_nothing() {
        let mut rhi = NullRhi::new();
        let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST)).unwrap();
        rhi.recreate().unwrap();
        assert!(rhi.submit(write(buf, 0, 4)).is_ok());
    }

    #[test]
    fn device_memory_runs_out_and_comes_back() {
        let mut rhi = NullRhi::with_memory(64);
        let a = rhi.create_buffer(&buffer(48, Usage::VERTEX)).unwrap();
        let error = rhi
            .create_buffer(&buffer(32, Usage::VERTEX))
            .expect_err("full");
        assert_eq!(error.recovery(), Recovery::Retry);
        rhi.destroy_buffer(a).unwrap();
        assert!(rhi.create_buffer(&buffer(32, Usage::VERTEX)).is_ok());
    }

    #[test]
    fn the_owner_pool_sees_every_byte_and_refuses_before_the_device_does() {
        let ledger = MemoryLedger::new();
        let pool = ledger
            .register(PoolSpec::new(
                "rhi.test",
                MemoryClass::Gpu,
                MemoryBudget::capacity(1024).unwrap(),
            ))
            .unwrap();
        let mut rhi = NullRhi::new();
        rhi.attach(Arc::clone(&pool));
        let tex = rhi.create_texture(&texture(16, Usage::SAMPLED)).unwrap();
        assert_eq!(pool.current(), 1024);
        assert!(rhi.create_buffer(&buffer(4, Usage::VERTEX)).is_err());
        assert_eq!(pool.report().refusals, 1);
        rhi.destroy_texture(tex).unwrap();
        assert_eq!(pool.current(), 0);
        assert_eq!(pool.high_water(), 1024);
    }

    #[test]
    fn a_draw_is_checked_against_its_pipeline_buffer_and_target() {
        let mut rhi = NullRhi::new();
        let stage = ShaderStage {
            entry: "main".into(),
            code: vec![0; 4],
        };
        let pipeline = rhi
            .create_pipeline(&PipelineDesc {
                label: "p".into(),
                vertex: stage.clone(),
                fragment: stage,
                vertex_stride: 16,
                targets: vec![TextureFormat::Rgba8Unorm],
            })
            .unwrap();
        let vertices = rhi.create_buffer(&buffer(48, Usage::VERTEX)).unwrap();
        let target = rhi
            .create_texture(&texture(8, Usage::RENDER_TARGET))
            .unwrap();
        let sampled = rhi.create_texture(&texture(8, Usage::SAMPLED)).unwrap();
        let draw = |target, vertices_n| {
            let mut list = CommandList::new("draw");
            list.push(Command::Draw {
                pipeline,
                buffer: vertices,
                target,
                vertices: vertices_n,
            });
            list
        };
        assert!(rhi.submit(draw(target, 3)).is_ok());
        assert!(rhi.submit(draw(target, 4)).is_err(), "reads past the end");
        assert!(rhi.submit(draw(target, 0)).is_err(), "draws nothing");
        assert!(rhi.submit(draw(sampled, 3)).is_err(), "not a render target");
        assert_eq!(rhi.stats().draws, 1);
    }

    #[test]
    fn the_null_backend_says_it_cannot_present_and_then_does_not() {
        let mut rhi = NullRhi::new();
        assert!(!rhi.capabilities().presents);
        let target = rhi
            .create_texture(&texture(8, Usage::RENDER_TARGET))
            .unwrap();
        let error = rhi.present(target).expect_err("no surface");
        assert_eq!(error.recovery(), Recovery::DisableSubsystem);
    }
}
