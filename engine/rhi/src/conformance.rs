//! The rules every backend must keep, as one runnable suite.
//!
//! `RENDER HARDWARE INTERFACE.md`, *Tests*, asks for **backend parity**. With
//! one backend, parity is a promise; this suite is what turns it into a check
//! the second backend cannot skip. It talks to the backend only through
//! [`Rhi`], asks nothing a real GPU could not answer, and leaves the backend
//! holding exactly the memory it held before.
//!
//! Device loss is not here: no portable call makes a real GPU lose its
//! device. The null backend's own tests cover it, and a native backend covers
//! it with whatever fault injection its API offers.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

use crate::api::{Command, CommandList, Fence, Rhi};
use crate::desc::{
    BufferDesc, PipelineDesc, ShaderStage, TextureDesc, TextureFormat, Usage, COPY_ALIGNMENT,
};

/// The cases, in the order [`run`] runs them.
pub const CASES: [&str; 9] = [
    "capabilities",
    "descriptor-rules",
    "stale-handles",
    "write-rules",
    "fence-order",
    "deferred-destruction",
    "draw-rules",
    "present",
    "at-rest",
];

/// A vertex and a fragment shader the backend under test accepts.
///
/// The suite cannot write shaders: which language a backend reads is its own
/// business until ADR-0025's successor decides it. A backend that does not
/// validate shaders can use [`TestShaders::opaque`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestShaders {
    /// Accepted as a pipeline's vertex stage.
    pub vertex: ShaderStage,
    /// Accepted as a pipeline's fragment stage.
    pub fragment: ShaderStage,
}

impl TestShaders {
    /// Placeholder stages, for backends whose capabilities say they do not
    /// validate shader code.
    #[must_use]
    pub fn opaque() -> Self {
        let stage = ShaderStage {
            entry: "main".into(),
            code: b"conformance".to_vec(),
        };
        Self {
            vertex: stage.clone(),
            fragment: stage,
        }
    }
}

/// What a passing run reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conformance {
    /// The backend that passed.
    pub backend: &'static str,
    /// Every case, in order. All of [`CASES`].
    pub passed: Vec<&'static str>,
}

/// Run every case against `rhi`. Stops at the first broken rule.
///
/// # Errors
///
/// Names the case and the rule the backend broke. A backend that fails is not
/// fit to put under a renderer: the error says so with `DisableSubsystem`.
pub fn run(rhi: &mut dyn Rhi, shaders: &TestShaders) -> Result<Conformance> {
    let baseline = rhi.allocated_bytes();
    let mut passed = Vec::with_capacity(CASES.len());
    for case in CASES {
        let outcome = match case {
            "capabilities" => capabilities(rhi),
            "descriptor-rules" => descriptor_rules(rhi, shaders, baseline),
            "stale-handles" => stale_handles(rhi),
            "write-rules" => write_rules(rhi),
            "fence-order" => fence_order(rhi),
            "deferred-destruction" => deferred_destruction(rhi, baseline),
            "draw-rules" => draw_rules(rhi, shaders),
            "present" => present(rhi),
            _ => at_rest(rhi, baseline),
        };
        outcome.map_err(|error| error.with_context("case", case))?;
        passed.push(case);
    }
    Ok(Conformance {
        backend: rhi.capabilities().backend,
        passed,
    })
}

fn broke(rule: &'static str) -> Error {
    Error::new(
        Domain::Render,
        "rhi-conformance",
        "the backend broke an RHI rule",
    )
    .with_recovery(Recovery::DisableSubsystem)
    .with_context("rule", rule)
}

fn ensure(holds: bool, rule: &'static str) -> Result<()> {
    if holds {
        Ok(())
    } else {
        Err(broke(rule))
    }
}

fn buffer(size: u64, usage: Usage) -> BufferDesc {
    BufferDesc {
        label: "conformance".into(),
        size,
        usage,
    }
}

fn texture(edge: u32, format: TextureFormat, usage: Usage) -> TextureDesc {
    TextureDesc {
        label: "conformance".into(),
        width: edge,
        height: edge,
        format,
        usage,
    }
}

fn pipeline(shaders: &TestShaders, targets: Vec<TextureFormat>) -> PipelineDesc {
    PipelineDesc {
        label: "conformance".into(),
        vertex: shaders.vertex.clone(),
        fragment: shaders.fragment.clone(),
        vertex_stride: 16,
        targets,
    }
}

fn one(command: Command) -> CommandList {
    let mut list = CommandList::new("conformance");
    list.push(command);
    list
}

fn settle(rhi: &mut dyn Rhi, fence: Fence) -> Result<()> {
    rhi.wait(fence)?;
    rhi.poll()?;
    Ok(())
}

fn capabilities(rhi: &mut dyn Rhi) -> Result<()> {
    let caps = rhi.capabilities();
    ensure(!caps.backend.is_empty(), "a backend names itself")?;
    ensure(
        caps.supports(TextureFormat::Rgba8Unorm) && caps.supports(TextureFormat::Rgba8UnormSrgb),
        "every backend creates RGBA8 textures, linear and sRGB",
    )?;
    ensure(caps.max_texture_edge >= 16, "a 16x16 texture always fits")?;
    ensure(
        caps.max_buffer_bytes >= COPY_ALIGNMENT && caps.memory_bytes > 0,
        "a backend has memory to hand out",
    )
}

fn descriptor_rules(rhi: &mut dyn Rhi, shaders: &TestShaders, baseline: u64) -> Result<()> {
    let caps = rhi.capabilities().clone();
    ensure(
        rhi.create_buffer(&buffer(6, Usage::VERTEX)).is_err(),
        "a buffer size that is not a multiple of four is refused",
    )?;
    ensure(
        rhi.create_buffer(&buffer(16, Usage::NONE)).is_err(),
        "a buffer with no usage is refused",
    )?;
    ensure(
        rhi.create_texture(&texture(
            caps.max_texture_edge + 1,
            TextureFormat::Rgba8Unorm,
            Usage::SAMPLED,
        ))
        .is_err(),
        "a texture over the maximum edge is refused",
    )?;
    for format in TextureFormat::ALL {
        if !caps.supports(format) {
            ensure(
                rhi.create_texture(&texture(16, format, Usage::SAMPLED))
                    .is_err(),
                "a format the backend did not declare is refused",
            )?;
        }
    }
    ensure(
        rhi.create_pipeline(&pipeline(shaders, Vec::new())).is_err(),
        "a pipeline with no target is refused",
    )?;
    ensure(
        rhi.allocated_bytes() == baseline,
        "a refused create allocates nothing",
    )
}

fn stale_handles(rhi: &mut dyn Rhi) -> Result<()> {
    let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST))?;
    rhi.destroy_buffer(buf)?;
    ensure(
        rhi.submit(one(Command::WriteBuffer {
            buffer: buf,
            offset: 0,
            data: vec![0; 4],
        }))
        .is_err(),
        "a destroyed buffer cannot be written",
    )?;
    ensure(
        rhi.destroy_buffer(buf).is_err(),
        "a buffer cannot be destroyed twice",
    )?;
    let tex = rhi.create_texture(&texture(4, TextureFormat::Rgba8Unorm, Usage::COPY_DST))?;
    rhi.destroy_texture(tex)?;
    ensure(
        rhi.submit(one(Command::WriteTexture {
            texture: tex,
            data: vec![0; 64],
        }))
        .is_err(),
        "a destroyed texture cannot be written",
    )
}

fn write_rules(rhi: &mut dyn Rhi) -> Result<()> {
    let target = rhi.create_buffer(&buffer(32, Usage::COPY_DST))?;
    let readonly = rhi.create_buffer(&buffer(32, Usage::VERTEX))?;
    let write = |buffer, offset, len| {
        one(Command::WriteBuffer {
            buffer,
            offset,
            data: vec![0xA5; len],
        })
    };
    ensure(
        rhi.submit(write(readonly, 0, 4)).is_err(),
        "a buffer without COPY_DST cannot be written",
    )?;
    ensure(
        rhi.submit(write(target, 2, 4)).is_err(),
        "a buffer write at an unaligned offset is refused",
    )?;
    ensure(
        rhi.submit(write(target, 0, 6)).is_err(),
        "a buffer write of an unaligned length is refused",
    )?;
    ensure(
        rhi.submit(write(target, 28, 8)).is_err(),
        "a buffer write past the end is refused",
    )?;
    let mut mixed = write(target, 0, 4);
    mixed.push(Command::WriteBuffer {
        buffer: target,
        offset: 32,
        data: vec![0; 4],
    });
    ensure(
        rhi.submit(mixed).is_err(),
        "one bad command refuses the whole list",
    )?;
    let fits = rhi.submit(write(target, 0, 32))?;

    let tex = rhi.create_texture(&texture(
        4,
        TextureFormat::Rgba8UnormSrgb,
        Usage::SAMPLED | Usage::COPY_DST,
    ))?;
    ensure(
        rhi.submit(one(Command::WriteTexture {
            texture: tex,
            data: vec![0; 60],
        }))
        .is_err(),
        "a texture upload short of one texel is refused",
    )?;
    let uploaded = rhi.submit(one(Command::WriteTexture {
        texture: tex,
        data: vec![0; 64],
    }))?;
    settle(rhi, fits.max(uploaded))?;
    rhi.destroy_texture(tex)?;
    rhi.destroy_buffer(readonly)?;
    rhi.destroy_buffer(target)
}

fn fence_order(rhi: &mut dyn Rhi) -> Result<()> {
    let buf = rhi.create_buffer(&buffer(16, Usage::COPY_DST))?;
    let mut fences = Vec::new();
    for offset in [0, 4, 8] {
        fences.push(rhi.submit(one(Command::WriteBuffer {
            buffer: buf,
            offset,
            data: vec![1; 4],
        }))?);
    }
    ensure(
        fences.windows(2).all(|pair| pair[0] < pair[1]),
        "fences are issued in submission order",
    )?;
    let last = *fences.last().expect("three fences");
    rhi.wait(last)?;
    ensure(
        fences.iter().all(|fence| rhi.is_complete(*fence)),
        "a completed fence completes every earlier one",
    )?;
    let never = Fence {
        epoch: last.epoch,
        value: last.value + 1_000,
    };
    ensure(
        rhi.wait(never).is_err(),
        "waiting on a fence never issued is refused, not a hang",
    )?;
    rhi.poll()?;
    rhi.destroy_buffer(buf)
}

fn deferred_destruction(rhi: &mut dyn Rhi, baseline: u64) -> Result<()> {
    const SIZE: u64 = 4096;
    let buf = rhi.create_buffer(&buffer(SIZE, Usage::COPY_DST))?;
    let fence = rhi.submit(one(Command::WriteBuffer {
        buffer: buf,
        offset: 0,
        data: vec![3; SIZE as usize],
    }))?;
    rhi.destroy_buffer(buf)?;
    if !rhi.is_complete(fence) {
        ensure(
            rhi.allocated_bytes() >= baseline + SIZE,
            "memory the GPU still reads is not freed under it",
        )?;
    }
    settle(rhi, fence)?;
    ensure(
        rhi.allocated_bytes() == baseline,
        "memory returns once the last use completes",
    )
}

fn draw_rules(rhi: &mut dyn Rhi, shaders: &TestShaders) -> Result<()> {
    let caps = rhi.capabilities().clone();
    let pipe = rhi.create_pipeline(&pipeline(shaders, vec![TextureFormat::Rgba8Unorm]))?;
    let vertices = rhi.create_buffer(&buffer(48, Usage::VERTEX))?;
    let target =
        rhi.create_texture(&texture(8, TextureFormat::Rgba8Unorm, Usage::RENDER_TARGET))?;
    let sampled = rhi.create_texture(&texture(8, TextureFormat::Rgba8Unorm, Usage::SAMPLED))?;
    let draw = |target, count| {
        one(Command::Draw {
            pipeline: pipe,
            buffer: vertices,
            target,
            vertices: count,
        })
    };
    let drawn = rhi.submit(draw(target, 3))?;
    ensure(
        rhi.submit(draw(target, 4)).is_err(),
        "a draw that reads past its vertex buffer is refused",
    )?;
    ensure(
        rhi.submit(draw(sampled, 3)).is_err(),
        "a draw into a texture that is not a render target is refused",
    )?;
    let mut other = None;
    if caps.supports(TextureFormat::R8Unorm) {
        let r8 = rhi.create_texture(&texture(8, TextureFormat::R8Unorm, Usage::RENDER_TARGET))?;
        ensure(
            rhi.submit(draw(r8, 3)).is_err(),
            "a draw into a format its pipeline was not built for is refused",
        )?;
        other = Some(r8);
    }
    settle(rhi, drawn)?;
    if let Some(r8) = other {
        rhi.destroy_texture(r8)?;
    }
    rhi.destroy_texture(sampled)?;
    rhi.destroy_texture(target)?;
    rhi.destroy_buffer(vertices)?;
    rhi.destroy_pipeline(pipe)
}

fn present(rhi: &mut dyn Rhi) -> Result<()> {
    let presents = rhi.capabilities().presents;
    let sampled = rhi.create_texture(&texture(8, TextureFormat::Rgba8Unorm, Usage::SAMPLED))?;
    ensure(
        rhi.present(sampled).is_err(),
        "only a render target can be presented",
    )?;
    let target =
        rhi.create_texture(&texture(8, TextureFormat::Rgba8Unorm, Usage::RENDER_TARGET))?;
    let shown = rhi.present(target).is_ok();
    ensure(
        shown == presents,
        "a backend presents exactly when its capabilities say it can",
    )?;
    rhi.destroy_texture(target)?;
    rhi.destroy_texture(sampled)
}

fn at_rest(rhi: &mut dyn Rhi, baseline: u64) -> Result<()> {
    // Every case settles its own fences; one more poll lets a backend that
    // retires lazily catch up. Then hold it to what it held before the suite:
    // a suite that leaks cannot prove anything else does not.
    rhi.poll()?;
    ensure(
        rhi.allocated_bytes() == baseline,
        "the suite leaves the backend holding what it held before",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::null::NullRhi;

    #[test]
    fn the_null_backend_passes_every_case() {
        let mut rhi = NullRhi::new();
        let report = run(&mut rhi, &TestShaders::opaque()).unwrap();
        assert_eq!(report.backend, "null");
        assert_eq!(report.passed, CASES.to_vec());
        assert_eq!(rhi.live(), (0, 0, 0));
        assert_eq!(rhi.allocated_bytes(), 0);
    }

    #[test]
    fn the_suite_runs_on_a_backend_already_holding_memory() {
        let mut rhi = NullRhi::new();
        let held = rhi.create_buffer(&buffer(64, Usage::VERTEX)).unwrap();
        run(&mut rhi, &TestShaders::opaque()).unwrap();
        assert_eq!(rhi.allocated_bytes(), 64);
        rhi.destroy_buffer(held).unwrap();
    }

    #[test]
    fn a_backend_that_frees_under_the_gpu_fails_the_suite() {
        // The bug deferred destruction exists to prevent: memory handed back
        // the moment its handle dies, while the GPU still reads it.
        let mut rhi = NullRhi::new();
        rhi.eager_free = true;
        let error = run(&mut rhi, &TestShaders::opaque()).expect_err("caught");
        assert!(error
            .context()
            .iter()
            .any(|(key, value)| *key == "case" && value == "deferred-destruction"));
    }

    /// A backend whose `present` claims success without a surface.
    struct Liar(NullRhi);

    #[test]
    fn a_backend_that_presents_without_saying_so_fails_the_suite() {
        let mut rhi = Liar(NullRhi::new());
        let error = run(&mut rhi, &TestShaders::opaque()).expect_err("caught");
        assert!(error
            .context()
            .iter()
            .any(|(key, value)| *key == "case" && value == "present"));
    }

    impl Rhi for Liar {
        fn capabilities(&self) -> &crate::desc::Capabilities {
            self.0.capabilities()
        }
        fn create_buffer(&mut self, desc: &BufferDesc) -> Result<crate::api::BufferHandle> {
            self.0.create_buffer(desc)
        }
        fn create_texture(&mut self, desc: &TextureDesc) -> Result<crate::api::TextureHandle> {
            self.0.create_texture(desc)
        }
        fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<crate::api::PipelineHandle> {
            self.0.create_pipeline(desc)
        }
        fn destroy_buffer(&mut self, buffer: crate::api::BufferHandle) -> Result<()> {
            self.0.destroy_buffer(buffer)
        }
        fn destroy_texture(&mut self, texture: crate::api::TextureHandle) -> Result<()> {
            self.0.destroy_texture(texture)
        }
        fn destroy_pipeline(&mut self, pipeline: crate::api::PipelineHandle) -> Result<()> {
            self.0.destroy_pipeline(pipeline)
        }
        fn submit(&mut self, list: CommandList) -> Result<Fence> {
            self.0.submit(list)
        }
        fn poll(&mut self) -> Result<Option<Fence>> {
            self.0.poll()
        }
        fn is_complete(&self, fence: Fence) -> bool {
            self.0.is_complete(fence)
        }
        fn wait(&mut self, fence: Fence) -> Result<()> {
            self.0.wait(fence)
        }
        fn present(&mut self, _texture: crate::api::TextureHandle) -> Result<()> {
            Ok(())
        }
        fn device_lost(&self) -> bool {
            self.0.device_lost()
        }
        fn recreate(&mut self) -> Result<()> {
            self.0.recreate()
        }
        fn allocated_bytes(&self) -> u64 {
            self.0.allocated_bytes()
        }
    }
}
