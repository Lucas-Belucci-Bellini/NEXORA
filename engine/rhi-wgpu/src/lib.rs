//! The first native RHI backend: [`WgpuRhi`], over `wgpu` (ADR-0026).
//!
//! `ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §5 puts a *native graphics
//! abstraction* between the RHI and Vulkan / DirectX / Metal. That layer is
//! `wgpu`, used through its safe API only, so this crate keeps the workspace's
//! `unsafe_code = "forbid"`.
//!
//! Everything a backend could disagree on is decided above this crate. The
//! descriptor rules come from `nexora_rhi::desc`, the command rules and
//! resource tables from `nexora_rhi::kit`, and `nexora_rhi::conformance` holds
//! this backend to the same nine cases as the null one. What this crate adds
//! is what only a driver can do: allocate, copy, compile, draw and signal.
//!
//! # Order, faithfully
//!
//! A command list runs in the order it was recorded. Writes are therefore
//! *encoded*, as copies from a staging buffer, not queued with
//! `Queue::write_*`: those land before the whole submission, so a list that
//! drew and then wrote the same buffer would see the write first.
//!
//! # Presentation
//!
//! [`WgpuRhi::new`] opens a device with no surface, as servers, tests and the
//! benchmark want, and [`Capabilities::presents`] is false. A window host
//! (ADR-0027: `nexora-window`) opens one with [`WgpuRhi::with_surface`]
//! instead; then `present` draws the texture onto the window and shows it.
//! This crate takes the window as anything `wgpu` can make a surface from, and
//! never names a windowing library.

pub mod proof;
mod surface;

use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::memory::MemoryPool;
use nexora_rhi::conformance::TestShaders;
use nexora_rhi::desc::{check_buffer, check_pipeline, check_texture, refused};
use nexora_rhi::kit::{device_lost, no_surface, presentable, Accounting, Resources};
use nexora_rhi::{
    Binding, BindingKind, BufferDesc, BufferHandle, Capabilities, ClearValue, Command, CommandList,
    Compare, Fence, Filter, PipelineDesc, PipelineHandle, Rhi, ShaderStage, TextureDesc,
    TextureFormat, TextureHandle, Usage, VertexFormat,
};

/// The engine's own ceiling on device memory, in bytes.
///
/// `wgpu` does not report how much memory a device has. This is a budget the
/// engine keeps, not a property of the GPU, and [`WgpuRhi::with_memory`]
/// changes it.
pub const DEFAULT_MEMORY: u64 = 1024 * 1024 * 1024;

/// How long [`Rhi::wait`] waits for one fence before it calls the GPU hung.
pub const WAIT_TIMEOUT: Duration = Duration::from_secs(10);

/// The WGSL the conformance suite runs on this backend.
///
/// `vs_main`/`fs_main`: position in, one flat colour out.
/// `vs_bound`/`fs_bound`: the suite's bound layout (ADR-0028), a position and
/// a texture coordinate in, the texture sampled through the sampler and
/// multiplied by the uniform tint.
pub const CONFORMANCE_WGSL: &str = r"
@vertex
fn vs_main(@location(0) position: vec4<f32>) -> @builtin(position) vec4<f32> {
    return position;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

struct Tint {
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> tint: Tint;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var image_sampler: sampler;

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_bound(@location(0) position: vec4<f32>, @location(1) uv: vec2<f32>) -> Varyings {
    var out: Varyings;
    out.position = position;
    out.uv = uv;
    return out;
}

@fragment
fn fs_bound(in: Varyings) -> @location(0) vec4<f32> {
    return textureSample(image, image_sampler, in.uv) * tint.color;
}
";

/// The conformance suite's shaders, in the language this backend reads.
#[must_use]
pub fn conformance_shaders() -> TestShaders {
    TestShaders {
        vertex: ShaderStage {
            entry: "vs_main".into(),
            code: CONFORMANCE_WGSL.as_bytes().to_vec(),
        },
        fragment: ShaderStage {
            entry: "fs_main".into(),
            code: CONFORMANCE_WGSL.as_bytes().to_vec(),
        },
        bound_vertex: ShaderStage {
            entry: "vs_bound".into(),
            code: CONFORMANCE_WGSL.as_bytes().to_vec(),
        },
        bound_fragment: ShaderStage {
            entry: "fs_bound".into(),
            code: CONFORMANCE_WGSL.as_bytes().to_vec(),
        },
    }
}

/// The adapter a backend opened, as a device class. Never a serial number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterClass {
    /// The adapter's name, as the driver reports it.
    pub name: String,
    /// `vulkan`, `dx12` or `metal`.
    pub backend: String,
    /// `discrete`, `integrated`, `cpu`, … as `wgpu` classifies it.
    pub kind: String,
    /// The driver, and its version when the driver says.
    pub driver: String,
}

/// The window surface a backend presents to, as a probe reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceInfo {
    /// Width in pixels, as last configured.
    pub width: u32,
    /// Height in pixels, as last configured.
    pub height: u32,
    /// The surface image format, as `wgpu` names it.
    pub format: String,
    /// How frames are queued for display.
    pub present_mode: String,
    /// Whether a presented image can be read back ([`WgpuRhi::present_and_capture`]).
    pub readable: bool,
    /// Frames shown so far.
    pub presented: u64,
}

/// A surface image as it was shown, read back from the GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentedFrame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The surface format the bytes are in, as `wgpu` names it.
    pub format: String,
    /// Four bytes a texel, row-major from the top-left, in `format`'s order.
    pub texels: Vec<u8>,
}

/// What a texture keeps next to its descriptor.
#[derive(Debug)]
struct TextureObject {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

/// What a pipeline keeps: one native pipeline per target format the
/// descriptor allows, the layout of its binding slots, and a sampler for each
/// sampler slot (`None` for the others).
#[derive(Debug)]
struct PipelineObject {
    variants: Vec<(TextureFormat, wgpu::RenderPipeline)>,
    layout: wgpu::BindGroupLayout,
    samplers: Vec<Option<wgpu::Sampler>>,
}

/// The native backend. See the crate documentation.
#[derive(Debug)]
pub struct WgpuRhi {
    caps: Capabilities,
    adapter: wgpu::Adapter,
    adapter_class: AdapterClass,
    device: wgpu::Device,
    queue: wgpu::Queue,
    lost: Arc<AtomicBool>,
    resources: Resources<wgpu::Buffer, TextureObject, PipelineObject>,
    issued: u64,
    completed: Arc<AtomicU64>,
    submissions: Vec<(u64, wgpu::SubmissionIndex)>,
    memory: Accounting,
    presenter: Option<surface::Presenter>,
}

impl WgpuRhi {
    /// Open the highest-performance adapter this machine has, without a
    /// surface, and a device on it.
    ///
    /// # Errors
    ///
    /// No adapter was found (no driver for Vulkan, Direct3D 12 or Metal), or
    /// the adapter refused a device.
    pub fn new() -> Result<Self> {
        Self::with_memory(DEFAULT_MEMORY)
    }

    /// The same, with the engine's device memory ceiling set to `bytes`.
    ///
    /// # Errors
    ///
    /// As [`WgpuRhi::new`].
    pub fn with_memory(bytes: u64) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        Self::open(&instance, None, bytes)
    }

    /// Open the highest-performance adapter that can present to `window`, a
    /// device on it, and the window's surface at `width` × `height` pixels.
    /// [`Capabilities::presents`] is true.
    ///
    /// `window` is anything `wgpu` makes a surface from; a window host passes
    /// its window here (ADR-0027).
    ///
    /// # Errors
    ///
    /// The window refused a surface, no adapter can present to it, or the
    /// adapter refused a device.
    pub fn with_surface(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window).map_err(|error| {
            Error::new(Domain::Render, "rhi-wgpu", "the window refused a surface")
                .with_recovery(Recovery::DisableSubsystem)
                .with_context("cause", error.to_string())
        })?;
        Self::open(&instance, Some((surface, width, height)), DEFAULT_MEMORY)
    }

    fn open(
        instance: &wgpu::Instance,
        window: Option<(wgpu::Surface<'static>, u32, u32)>,
        bytes: u64,
    ) -> Result<Self> {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: window.as_ref().map(|(surface, _, _)| surface),
            apply_limit_buckets: false,
        }))
        .map_err(|error| {
            Error::new(
                Domain::Render,
                "rhi-wgpu",
                "no GPU adapter: no Vulkan, Direct3D 12 or Metal driver answered",
            )
            .with_recovery(Recovery::DisableSubsystem)
            .with_context("cause", error.to_string())
        })?;
        let info = adapter.get_info();
        let adapter_class = AdapterClass {
            name: info.name.clone(),
            backend: info.backend.to_str().to_owned(),
            kind: format!("{:?}", info.device_type).to_lowercase(),
            driver: [info.driver.as_str(), info.driver_info.as_str()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
        };
        let (device, queue, lost) = open_device(&adapter)?;
        let presenter = match window {
            Some((surface, width, height)) => {
                let config = surface::configure(&surface, &adapter, width, height)?;
                surface.configure(&device, &config);
                let blit = surface::blit(&device, config.format);
                Some(surface::Presenter {
                    surface,
                    config,
                    blit,
                    presented: 0,
                })
            }
            None => None,
        };
        let limits = device.limits();
        let caps = Capabilities {
            backend: "wgpu",
            device: format!("{} ({})", adapter_class.name, adapter_class.backend),
            max_texture_edge: limits.max_texture_dimension_2d,
            max_buffer_bytes: limits.max_buffer_size.min(bytes),
            memory_bytes: bytes,
            // All five are guaranteed by the WebGPU specification on every
            // adapter wgpu will open.
            formats: TextureFormat::ALL.to_vec(),
            presents: presenter.is_some(),
            validates_shaders: true,
        };
        Ok(Self {
            caps,
            adapter,
            adapter_class,
            device,
            queue,
            lost,
            resources: Resources::new(0),
            issued: 0,
            completed: Arc::new(AtomicU64::new(0)),
            submissions: Vec::new(),
            memory: Accounting::new(bytes),
            presenter,
        })
    }

    /// The surface this backend presents to, if it has one.
    #[must_use]
    pub fn surface(&self) -> Option<SurfaceInfo> {
        self.presenter.as_ref().map(|presenter| SurfaceInfo {
            width: presenter.config.width,
            height: presenter.config.height,
            format: format!("{:?}", presenter.config.format),
            present_mode: format!("{:?}", presenter.config.present_mode),
            readable: readable_surface(presenter),
            presented: presenter.presented,
        })
    }

    /// The window changed size: reconfigure the surface to `width` × `height`.
    /// A zero edge (a minimized window) keeps the old size until it is not.
    ///
    /// # Errors
    ///
    /// The device is lost, or the backend has no surface.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.alive()?;
        let presenter = self
            .presenter
            .as_mut()
            .ok_or_else(|| no_surface(self.caps.backend))?;
        if width == 0 || height == 0 {
            return Ok(());
        }
        presenter.config.width = width;
        presenter.config.height = height;
        presenter.surface.configure(&self.device, &presenter.config);
        Ok(())
    }

    /// [`Rhi::present`], and read back the surface image exactly as it went
    /// to the window. `None` when this surface cannot be read (its platform
    /// does not allow copies from it, or its format is not 8 bits a channel).
    /// Waits for the GPU.
    ///
    /// Not part of [`Rhi`]: it exists so tests and the local validation probe
    /// can prove a frame reached the window, not only that `present` returned.
    ///
    /// # Errors
    ///
    /// As [`Rhi::present`].
    pub fn present_and_capture(
        &mut self,
        texture: TextureHandle,
    ) -> Result<Option<PresentedFrame>> {
        self.show(texture, true)
    }

    /// The adapter this backend runs on.
    #[must_use]
    pub const fn adapter(&self) -> &AdapterClass {
        &self.adapter_class
    }

    /// Account device memory in `pool` (`MemoryClass::Gpu`, ADR-0024).
    pub fn attach(&mut self, pool: Arc<MemoryPool>) {
        self.memory.attach(pool);
    }

    /// Live resources, as `(buffers, textures, pipelines)`.
    #[must_use]
    pub fn live(&self) -> (usize, usize, usize) {
        self.resources.live()
    }

    /// Destroy the device, as a driver reset would. The device-lost callback
    /// fires and every call fails until [`Rhi::recreate`]. For tests and
    /// fault drills.
    pub fn lose_device(&mut self) {
        self.device.destroy();
        // Destruction is reported through the device-lost callback; polling
        // makes sure it has run before the caller looks.
        let _ = self.device.poll(wgpu::PollType::Poll);
        self.lost.store(true, Ordering::SeqCst);
    }

    /// Read a texture back to the CPU, row-major from the top-left, exactly
    /// [`TextureDesc::byte_len`] bytes. Waits for the GPU.
    ///
    /// Not part of [`Rhi`]: it exists so tests and the local validation probe
    /// can prove that uploads and draws happened on the GPU, rather than
    /// trusting that a submission was accepted.
    ///
    /// # Errors
    ///
    /// The handle is stale, the texture lacks `COPY_SRC`, or the device is
    /// lost.
    pub fn read_texture(&mut self, texture: TextureHandle) -> Result<Vec<u8>> {
        self.alive()?;
        let live = self.resources.texture(texture)?;
        if !live.desc.usage.contains(Usage::COPY_SRC) {
            return Err(refused("a texture read back needs COPY_SRC usage")
                .with_context("label", &live.desc.label));
        }
        let (width, height) = (live.desc.width, live.desc.height);
        let row = width * texel_bytes(live.desc.format);
        let padded = padded_row(row);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nexora readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nexora readback"),
            });
        encoder.copy_texture_to_buffer(
            live.native.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            extent(width, height),
        );
        self.queue.submit([encoder.finish()]);
        self.map_rows(&staging, padded, row, height)
    }

    /// Draw `texture` onto the next surface image and show it.
    fn show(&mut self, texture: TextureHandle, capture: bool) -> Result<Option<PresentedFrame>> {
        self.alive()?;
        let live = self.resources.texture(texture)?;
        presentable(&live.desc)?;
        let Some(presenter) = self.presenter.as_mut() else {
            return Err(no_surface(self.caps.backend));
        };
        let (frame, suboptimal) = surface::acquire(presenter, &self.device)?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bindings = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nexora present"),
            layout: &presenter.blit.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&live.native.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&presenter.blit.sampler),
                },
            ],
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nexora present"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nexora present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&presenter.blit.pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.draw(0..3, 0..1);
        }
        let (width, height) = (presenter.config.width, presenter.config.height);
        let capture = (capture && readable_surface(presenter)).then(|| {
            let padded = padded_row(width * 4);
            let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nexora present readback"),
                size: u64::from(padded) * u64::from(height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                frame.texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded),
                        rows_per_image: Some(height),
                    },
                },
                extent(width, height),
            );
            (staging, padded)
        });
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let index = self.queue.submit([encoder.finish()]);
        if let Some(error) = block_on(scope.pop()) {
            return Err(Error::new(
                Domain::Render,
                "rhi-wgpu",
                "the driver refused to present a checked texture",
            )
            .with_recovery(Recovery::Manual)
            .with_context("label", &live.desc.label)
            .with_context("cause", error.to_string()));
        }
        self.queue.present(frame);
        presenter.presented += 1;
        if suboptimal {
            presenter.surface.configure(&self.device, &presenter.config);
        }
        let format = format!("{:?}", presenter.config.format);
        // Presenting reads the texture on the GPU, so it is in flight like a
        // draw: its memory waits for this fence if the handle dies first.
        self.issued += 1;
        let value = self.issued;
        self.resources.mark_texture_used(texture, value);
        self.submissions.push((value, index));
        let completed = Arc::clone(&self.completed);
        self.queue.on_submitted_work_done(move || {
            completed.fetch_max(value, Ordering::SeqCst);
        });
        let Some((staging, padded)) = capture else {
            return Ok(None);
        };
        let texels = self.map_rows(&staging, padded, width * 4, height)?;
        Ok(Some(PresentedFrame {
            width,
            height,
            format,
            texels,
        }))
    }

    /// Wait for `staging` and copy its `height` rows of `row` bytes out,
    /// dropping the padding up to `padded`.
    fn map_rows(
        &self,
        staging: &wgpu::Buffer,
        padded: u32,
        row: u32,
        height: u32,
    ) -> Result<Vec<u8>> {
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(WAIT_TIMEOUT),
            })
            .map_err(|error| gpu_failed("reading a texture back", &error))?;
        let mapped = slice.get_mapped_range().map_err(|error| {
            Error::new(Domain::Render, "rhi-wgpu", "a readback buffer did not map")
                .with_recovery(Recovery::Retry)
                .with_context("cause", error.to_string())
        })?;
        let mut texels = Vec::with_capacity((row * height) as usize);
        for line in mapped.chunks(padded as usize).take(height as usize) {
            texels.extend_from_slice(&line[..row as usize]);
        }
        drop(mapped);
        staging.unmap();
        Ok(texels)
    }

    fn alive(&self) -> Result<()> {
        if self.lost.load(Ordering::SeqCst) {
            return Err(device_lost(self.caps.backend));
        }
        Ok(())
    }

    /// Retire every submission the GPU has finished.
    fn retire(&mut self) {
        let done = self.completed.load(Ordering::SeqCst);
        self.submissions.retain(|(value, _)| *value > done);
        self.memory.retire(done);
    }

    fn fence(&self, value: u64) -> Fence {
        Fence {
            epoch: self.resources.epoch(),
            value,
        }
    }

    /// Record a checked list into a command buffer, in order.
    fn encode(&self, list: &CommandList) -> Result<wgpu::CommandBuffer> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&list.label),
            });
        for command in &list.commands {
            match command {
                Command::WriteBuffer {
                    buffer,
                    offset,
                    data,
                } => {
                    let target = &self.resources.buffer(*buffer)?.native;
                    let staging = self.staging(data, "nexora buffer write");
                    encoder.copy_buffer_to_buffer(&staging, 0, target, *offset, data.len() as u64);
                }
                Command::WriteTexture { texture, data } => {
                    let live = self.resources.texture(*texture)?;
                    let (width, height) = (live.desc.width, live.desc.height);
                    let row = width * texel_bytes(live.desc.format);
                    let padded = padded_row(row);
                    let mut rows = Vec::with_capacity((padded * height) as usize);
                    for line in data.chunks(row as usize) {
                        rows.extend_from_slice(line);
                        rows.resize(rows.len() + (padded - row) as usize, 0);
                    }
                    let staging = self.staging(&rows, "nexora texture upload");
                    encoder.copy_buffer_to_texture(
                        wgpu::TexelCopyBufferInfo {
                            buffer: &staging,
                            layout: wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(padded),
                                rows_per_image: Some(height),
                            },
                        },
                        live.native.texture.as_image_copy(),
                        extent(width, height),
                    );
                }
                Command::Draw {
                    pipeline,
                    buffer,
                    target,
                    depth,
                    bindings,
                    vertices,
                } => {
                    let out = self.resources.texture(*target)?;
                    let object = &self.resources.pipeline(*pipeline)?.native;
                    let pipe = object
                        .variants
                        .iter()
                        .find(|(format, _)| *format == out.desc.format)
                        .map(|(_, pipe)| pipe)
                        .ok_or_else(|| refused("no pipeline variant for the target's format"))?;
                    let group = self.bind_group(object, bindings)?;
                    let depth_view = match depth {
                        Some(handle) => Some(&self.resources.texture(*handle)?.native.view),
                        None => None,
                    };
                    let vertex = &self.resources.buffer(*buffer)?.native;
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nexora draw"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &out.native.view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: depth_view.map(|view| {
                            wgpu::RenderPassDepthStencilAttachment {
                                view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    pass.set_pipeline(pipe);
                    if let Some(group) = &group {
                        pass.set_bind_group(0, group, &[]);
                    }
                    pass.set_vertex_buffer(0, vertex.slice(..));
                    pass.draw(0..*vertices, 0..1);
                }
                Command::Clear { texture, value } => {
                    let live = self.resources.texture(*texture)?;
                    let view = &live.native.view;
                    let (color, depth) = match value {
                        ClearValue::Color([r, g, b, a]) => (
                            Some(wgpu::RenderPassColorAttachment {
                                view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: f64::from(*r),
                                        g: f64::from(*g),
                                        b: f64::from(*b),
                                        a: f64::from(*a),
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            }),
                            None,
                        ),
                        ClearValue::Depth(depth) => (
                            None,
                            Some(wgpu::RenderPassDepthStencilAttachment {
                                view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(*depth),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                        ),
                    };
                    // A pass with no draws does exactly its loads and stores.
                    drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("nexora clear"),
                        color_attachments: &[color],
                        depth_stencil_attachment: depth,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    }));
                }
                Command::Marker(label) => encoder.insert_debug_marker(label),
            }
        }
        Ok(encoder.finish())
    }

    /// The bind group for one draw, in the pipeline's slot order. `None` for
    /// a pipeline without slots. The shared rules have already checked each
    /// binding's kind and usage.
    fn bind_group(
        &self,
        object: &PipelineObject,
        bindings: &[Binding],
    ) -> Result<Option<wgpu::BindGroup>> {
        if bindings.is_empty() {
            return Ok(None);
        }
        let mut entries = Vec::with_capacity(bindings.len());
        for ((slot, binding), sampler) in (0u32..).zip(bindings).zip(&object.samplers) {
            let resource = match (binding, sampler) {
                (Binding::Uniform(buffer), _) => {
                    self.resources.buffer(*buffer)?.native.as_entire_binding()
                }
                (Binding::Texture(texture), _) => wgpu::BindingResource::TextureView(
                    &self.resources.texture(*texture)?.native.view,
                ),
                (Binding::Sampler, Some(sampler)) => wgpu::BindingResource::Sampler(sampler),
                (Binding::Sampler, None) => {
                    return Err(refused("a sampler binding in a slot with no sampler"));
                }
            };
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource,
            });
        }
        Ok(Some(self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("nexora draw"),
                layout: &object.layout,
                entries: &entries,
            },
        )))
    }

    /// A buffer holding `bytes`, ready to copy from.
    fn staging(&self, bytes: &[u8], label: &str) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });
        buffer
            .slice(..)
            .get_mapped_range_mut()
            .expect("a buffer created mapped is mapped")
            .copy_from_slice(bytes);
        buffer.unmap();
        buffer
    }

    /// Create a shader module, turning a validation failure into a refusal.
    fn shader(&self, stage: &str, shader: &ShaderStage, label: &str) -> Result<wgpu::ShaderModule> {
        let source = std::str::from_utf8(&shader.code).map_err(|_| {
            refused("WGSL source must be UTF-8")
                .with_context("label", label)
                .with_context("stage", stage)
        })?;
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        if let Some(error) = block_on(scope.pop()) {
            return Err(refused("the shader did not compile")
                .with_context("label", label)
                .with_context("stage", stage)
                .with_context("cause", error.to_string()));
        }
        Ok(module)
    }
}

fn open_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue, Arc<AtomicBool>)> {
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("nexora"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
        ..Default::default()
    }))
    .map_err(|error| {
        Error::new(Domain::Render, "rhi-wgpu", "the adapter refused a device")
            .with_recovery(Recovery::DisableSubsystem)
            .with_context("cause", error.to_string())
    })?;
    let lost = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&lost);
    device.set_device_lost_callback(move |_, _| flag.store(true, Ordering::SeqCst));
    Ok((device, queue, lost))
}

impl Rhi for WgpuRhi {
    fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    fn create_buffer(&mut self, desc: &BufferDesc) -> Result<BufferHandle> {
        self.alive()?;
        check_buffer(desc, &self.caps)?;
        self.memory.reserve(&desc.label, desc.size)?;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&desc.label),
            size: desc.size,
            usage: buffer_usages(desc.usage),
            mapped_at_creation: false,
        });
        Ok(self.resources.insert_buffer(desc.clone(), buffer))
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle> {
        self.alive()?;
        check_texture(desc, &self.caps)?;
        self.memory.reserve(&desc.label, desc.byte_len())?;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&desc.label),
            size: extent(desc.width, desc.height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format(desc.format),
            usage: texture_usages(desc.usage) | presented_usages(desc),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(self
            .resources
            .insert_texture(desc.clone(), TextureObject { texture, view }))
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle> {
        self.alive()?;
        check_pipeline(desc, &self.caps)?;
        let vertex = self.shader("vertex", &desc.vertex, &desc.label)?;
        let fragment = self.shader("fragment", &desc.fragment, &desc.label)?;
        let attributes: Vec<_> = desc
            .attributes
            .iter()
            .map(|attribute| wgpu::VertexAttribute {
                format: vertex_format(attribute.format),
                offset: u64::from(attribute.offset),
                shader_location: attribute.location,
            })
            .collect();
        let entries: Vec<_> = (0u32..)
            .zip(&desc.bindings)
            .map(|(slot, kind)| layout_entry(slot, *kind))
            .collect();
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&desc.label),
                entries: &entries,
            });
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&desc.label),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
        let samplers = desc
            .bindings
            .iter()
            .map(|kind| match kind {
                BindingKind::Sampler(mode) => {
                    Some(self.device.create_sampler(&wgpu::SamplerDescriptor {
                        label: Some(&desc.label),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        mag_filter: filter(*mode),
                        min_filter: filter(*mode),
                        ..Default::default()
                    }))
                }
                _ => None,
            })
            .collect();
        let depth_stencil = desc.depth.map(|state| wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(state.write),
            depth_compare: Some(compare(state.compare)),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        let mut variants = Vec::with_capacity(desc.targets.len());
        for format in &desc.targets {
            let pipeline = self
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&desc.label),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vertex,
                        entry_point: Some(&desc.vertex.entry),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[Some(wgpu::VertexBufferLayout {
                            array_stride: u64::from(desc.vertex_stride),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &attributes,
                        })],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: depth_stencil.clone(),
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment,
                        entry_point: Some(&desc.fragment.entry),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: texture_format(*format),
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: None,
                });
            variants.push((*format, pipeline));
        }
        if let Some(error) = block_on(scope.pop()) {
            // The shaders compiled, so what failed is how they meet the
            // layout: a binding or an attribute the shader reads differently.
            return Err(refused("the pipeline did not link")
                .with_context("label", &desc.label)
                .with_context("cause", error.to_string()));
        }
        Ok(self.resources.insert_pipeline(
            desc.clone(),
            PipelineObject {
                variants,
                layout,
                samplers,
            },
        ))
    }

    fn destroy_buffer(&mut self, buffer: BufferHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_buffer(buffer)?;
        // wgpu keeps the allocation alive for in-flight work by itself; the
        // accounting mirrors that, so the ledger never shows memory freed
        // that the GPU is still reading.
        self.memory.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn destroy_texture(&mut self, texture: TextureHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_texture(texture)?;
        self.memory.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn destroy_pipeline(&mut self, pipeline: PipelineHandle) -> Result<()> {
        self.alive()?;
        let gone = self.resources.remove_pipeline(pipeline)?;
        self.memory.release(gone.bytes, gone.last_use);
        Ok(())
    }

    fn submit(&mut self, list: CommandList) -> Result<Fence> {
        self.alive()?;
        let checked = self.resources.check(&list)?;
        let commands = self.encode(&list)?;
        let scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let index = self.queue.submit([commands]);
        if let Some(error) = block_on(scope.pop()) {
            // A list the shared rules accepted and the driver refused is a
            // parity failure, not a caller error: say so loudly.
            return Err(Error::new(
                Domain::Render,
                "rhi-wgpu",
                "the driver refused a checked list",
            )
            .with_recovery(Recovery::Manual)
            .with_context("list", &list.label)
            .with_context("cause", error.to_string()));
        }
        self.issued += 1;
        let value = self.issued;
        self.resources.mark_used(&checked, value);
        self.submissions.push((value, index));
        let completed = Arc::clone(&self.completed);
        self.queue.on_submitted_work_done(move || {
            completed.fetch_max(value, Ordering::SeqCst);
        });
        Ok(self.fence(value))
    }

    fn poll(&mut self) -> Result<Option<Fence>> {
        self.alive()?;
        self.device
            .poll(wgpu::PollType::Poll)
            .map_err(|error| gpu_failed("polling", &error))?;
        self.retire();
        let done = self.completed.load(Ordering::SeqCst);
        Ok((done > 0).then(|| self.fence(done)))
    }

    fn is_complete(&self, fence: Fence) -> bool {
        fence.epoch == self.resources.epoch()
            && fence.value <= self.completed.load(Ordering::SeqCst)
    }

    fn wait(&mut self, fence: Fence) -> Result<()> {
        self.alive()?;
        if fence.epoch != self.resources.epoch() || fence.value == 0 || fence.value > self.issued {
            return Err(refused("the fence was never issued by this device life")
                .with_context("fence", fence.value.to_string())
                .with_context("issued", self.issued.to_string()));
        }
        if let Some((_, index)) = self
            .submissions
            .iter()
            .find(|(value, _)| *value == fence.value)
        {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index.clone()),
                    timeout: Some(WAIT_TIMEOUT),
                })
                .map_err(|error| gpu_failed("waiting on a fence", &error))?;
        }
        self.retire();
        if !self.is_complete(fence) {
            // The work is done but its callback has not run yet: callbacks
            // run inside a poll, so one more settles it.
            self.device
                .poll(wgpu::PollType::Poll)
                .map_err(|error| gpu_failed("waiting on a fence", &error))?;
            self.retire();
        }
        Ok(())
    }

    fn present(&mut self, texture: TextureHandle) -> Result<()> {
        self.show(texture, false).map(|_| ())
    }

    fn device_lost(&self) -> bool {
        self.lost.load(Ordering::SeqCst)
    }

    fn recreate(&mut self) -> Result<()> {
        if !self.device_lost() {
            return Ok(());
        }
        let (device, queue, lost) = open_device(&self.adapter)?;
        self.device = device;
        self.queue = queue;
        self.lost = lost;
        self.resources = Resources::new(self.resources.epoch().wrapping_add(1));
        self.issued = 0;
        self.completed = Arc::new(AtomicU64::new(0));
        self.submissions.clear();
        self.memory.reset();
        if let Some(presenter) = self.presenter.as_mut() {
            // The surface outlives the device; its configuration and the blit
            // belong to the old one.
            presenter.surface.configure(&self.device, &presenter.config);
            presenter.blit = surface::blit(&self.device, presenter.config.format);
        }
        Ok(())
    }

    fn allocated_bytes(&self) -> u64 {
        self.memory.allocated()
    }
}

fn gpu_failed(during: &'static str, error: &wgpu::PollError) -> Error {
    Error::new(Domain::Render, "rhi-wgpu", "the GPU did not answer")
        .with_recovery(Recovery::Retry)
        .with_context("during", during)
        .with_context("cause", error.to_string())
}

const fn texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
    }
}

fn texel_bytes(format: TextureFormat) -> u32 {
    u32::try_from(format.bytes_per_texel()).expect("a texel is a few bytes")
}

/// Rows of a texture copy are padded to the alignment every API shares.
fn padded_row(row: u32) -> u32 {
    row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

const fn extent(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}

const fn vertex_format(format: VertexFormat) -> wgpu::VertexFormat {
    match format {
        VertexFormat::Float32 => wgpu::VertexFormat::Float32,
        VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
        VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        VertexFormat::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
        VertexFormat::Uint32 => wgpu::VertexFormat::Uint32,
    }
}

const fn compare(compare: Compare) -> wgpu::CompareFunction {
    match compare {
        Compare::Less => wgpu::CompareFunction::Less,
        Compare::LessEqual => wgpu::CompareFunction::LessEqual,
        Compare::Always => wgpu::CompareFunction::Always,
    }
}

const fn filter(filter: Filter) -> wgpu::FilterMode {
    match filter {
        Filter::Nearest => wgpu::FilterMode::Nearest,
        Filter::Linear => wgpu::FilterMode::Linear,
    }
}

/// Slot `n` is `@group(0) @binding(n)`, visible to both stages.
fn layout_entry(slot: u32, kind: BindingKind) -> wgpu::BindGroupLayoutEntry {
    let ty = match kind {
        BindingKind::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingKind::Texture => wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        BindingKind::Sampler(_) => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    };
    wgpu::BindGroupLayoutEntry {
        binding: slot,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty,
        count: None,
    }
}

/// What a colour render target needs beyond its declared usage so `present`
/// can sample it onto the surface.
fn presented_usages(desc: &TextureDesc) -> wgpu::TextureUsages {
    if desc.usage.contains(Usage::RENDER_TARGET) && desc.format.is_color() {
        wgpu::TextureUsages::TEXTURE_BINDING
    } else {
        wgpu::TextureUsages::empty()
    }
}

fn readable_surface(presenter: &surface::Presenter) -> bool {
    presenter
        .config
        .usage
        .contains(wgpu::TextureUsages::COPY_SRC)
        && surface::readable(presenter.config.format)
}

fn buffer_usages(usage: Usage) -> wgpu::BufferUsages {
    let mut out = wgpu::BufferUsages::empty();
    for (ours, theirs) in [
        (Usage::VERTEX, wgpu::BufferUsages::VERTEX),
        (Usage::INDEX, wgpu::BufferUsages::INDEX),
        (Usage::UNIFORM, wgpu::BufferUsages::UNIFORM),
        (Usage::COPY_SRC, wgpu::BufferUsages::COPY_SRC),
        (Usage::COPY_DST, wgpu::BufferUsages::COPY_DST),
    ] {
        if usage.contains(ours) {
            out |= theirs;
        }
    }
    out
}

fn texture_usages(usage: Usage) -> wgpu::TextureUsages {
    let mut out = wgpu::TextureUsages::empty();
    for (ours, theirs) in [
        (Usage::SAMPLED, wgpu::TextureUsages::TEXTURE_BINDING),
        (Usage::COPY_SRC, wgpu::TextureUsages::COPY_SRC),
        (Usage::COPY_DST, wgpu::TextureUsages::COPY_DST),
        (Usage::RENDER_TARGET, wgpu::TextureUsages::RENDER_ATTACHMENT),
    ] {
        if usage.contains(ours) {
            out |= theirs;
        }
    }
    out
}

/// Drive a future to completion on this thread.
///
/// wgpu's native futures resolve inside its own polling, so a no-op waker and
/// a yield are enough; this is not an executor and nothing else runs on it.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        std::thread::yield_now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_rows_pad_to_the_shared_alignment() {
        assert_eq!(padded_row(64), 256);
        assert_eq!(padded_row(256), 256);
        assert_eq!(padded_row(257), 512);
    }

    #[test]
    fn usages_translate_bit_for_bit() {
        assert_eq!(
            buffer_usages(Usage::VERTEX | Usage::COPY_DST),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        );
        assert_eq!(
            texture_usages(Usage::SAMPLED | Usage::RENDER_TARGET),
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );
    }

    #[test]
    fn every_contract_format_has_a_wgpu_format_of_the_same_size() {
        for format in TextureFormat::ALL {
            let theirs = texture_format(format);
            assert_eq!(
                theirs.block_copy_size(None).map(u64::from),
                Some(format.bytes_per_texel()),
                "{format:?}"
            );
        }
    }
}
