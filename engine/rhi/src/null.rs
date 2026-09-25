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

use nexora_foundation::error::Result;
use nexora_foundation::memory::MemoryPool;

use crate::api::{BufferHandle, CommandList, Fence, PipelineHandle, Rhi, TextureHandle};
use crate::desc::{
    check_buffer, check_pipeline, check_texture, refused, BufferDesc, Capabilities, PipelineDesc,
    TextureDesc, TextureFormat, Usage,
};
use crate::kit::{device_lost, no_surface, Accounting, Resources};

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

/// The null backend. See the module documentation.
#[derive(Debug)]
pub struct NullRhi {
    caps: Capabilities,
    lost: bool,
    resources: Resources<(), (), ()>,
    issued: u64,
    completed: u64,
    memory: Accounting,
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
            lost: false,
            resources: Resources::new(0),
            issued: 0,
            completed: 0,
            memory: Accounting::new(bytes),
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
        self.memory.attach(pool);
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
        self.resources.live()
    }

    fn alive(&self) -> Result<()> {
        if self.lost {
            return Err(device_lost(self.caps.backend));
        }
        Ok(())
    }

    /// Free now, or when the last submission that used it completes.
    fn release(&mut self, bytes: u64, last_use: u64) {
        #[cfg(test)]
        let last_use = if self.eager_free { 0 } else { last_use };
        self.memory.release(bytes, last_use);
    }
}

impl Rhi for NullRhi {
    fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    fn create_buffer(&mut self, desc: &BufferDesc) -> Result<BufferHandle> {
        self.alive()?;
        check_buffer(desc, &self.caps)?;
        self.memory.reserve(&desc.label, desc.size)?;
        Ok(self.resources.insert_buffer(desc.clone(), ()))
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle> {
        self.alive()?;
        check_texture(desc, &self.caps)?;
        self.memory.reserve(&desc.label, desc.byte_len())?;
        Ok(self.resources.insert_texture(desc.clone(), ()))
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle> {
        self.alive()?;
        check_pipeline(desc, &self.caps)?;
        Ok(self.resources.insert_pipeline(desc.clone(), ()))
    }

    fn destroy_buffer(&mut self, buffer: BufferHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_buffer(buffer)?;
        self.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn destroy_texture(&mut self, texture: TextureHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_texture(texture)?;
        self.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn destroy_pipeline(&mut self, pipeline: PipelineHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_pipeline(pipeline)?;
        self.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn submit(&mut self, list: CommandList) -> Result<Fence> {
        self.alive()?;
        let checked = match self.resources.check(&list) {
            Ok(checked) => checked,
            Err(error) => {
                self.stats.refused_submissions += 1;
                return Err(error);
            }
        };
        self.issued += 1;
        self.resources.mark_used(&checked, self.issued);
        self.stats.submissions += 1;
        self.stats.commands += list.commands.len() as u64;
        self.stats.bytes_uploaded += checked.uploaded;
        self.stats.draws += checked.draws;
        Ok(Fence {
            epoch: self.resources.epoch(),
            value: self.issued,
        })
    }

    fn poll(&mut self) -> Result<Option<Fence>> {
        self.alive()?;
        if self.completed < self.issued {
            self.completed += 1;
            self.memory.retire(self.completed);
        }
        Ok((self.completed > 0).then_some(Fence {
            epoch: self.resources.epoch(),
            value: self.completed,
        }))
    }

    fn is_complete(&self, fence: Fence) -> bool {
        fence.epoch == self.resources.epoch() && fence.value <= self.completed
    }

    fn wait(&mut self, fence: Fence) -> Result<()> {
        self.alive()?;
        if fence.epoch != self.resources.epoch() || fence.value == 0 || fence.value > self.issued {
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
        let live = self.resources.texture(texture)?;
        if !live.desc.usage.contains(Usage::RENDER_TARGET) {
            return Err(refused("only a render target can be presented")
                .with_context("label", &live.desc.label));
        }
        Err(no_surface(self.caps.backend))
    }

    fn device_lost(&self) -> bool {
        self.lost
    }

    fn recreate(&mut self) -> Result<()> {
        if !self.lost {
            return Ok(());
        }
        self.resources = Resources::new(self.resources.epoch().wrapping_add(1));
        self.lost = false;
        self.issued = 0;
        self.completed = 0;
        self.memory.reset();
        Ok(())
    }

    fn allocated_bytes(&self) -> u64 {
        self.memory.allocated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Command;
    use crate::desc::ShaderStage;
    use nexora_foundation::error::Recovery;
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
