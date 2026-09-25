//! A window's surface, and how a frame reaches it (ADR-0027).
//!
//! The RHI presents a *texture*: whatever the renderer drew into a colour
//! render target. The surface's own images belong to the window system, in a
//! format the platform chooses (usually BGRA), so `present` does not hand the
//! texture over. It draws it onto the surface image with one full-screen
//! triangle that samples it, which also converts the channel order, the colour
//! space and the size. Then the image is shown.
//!
//! This module never sees a window type. It takes anything `wgpu` can make a
//! surface from, so the window host (`nexora-window`, over `winit`) stays out
//! of this crate's dependencies.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// A surface the backend presents to.
#[derive(Debug)]
pub(crate) struct Presenter {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) blit: Blit,
    /// Frames shown since the surface was opened.
    pub(crate) presented: u64,
}

/// What drawing a texture onto the surface needs, per device.
#[derive(Debug)]
pub(crate) struct Blit {
    pub(crate) layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) sampler: wgpu::Sampler,
}

/// The blit: a triangle that covers the target, sampling the texture with
/// (0, 0) at the top-left, as the RHI lays texels out, stretched to the
/// window.
const BLIT_WGSL: &str = r"
struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> Varyings {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Varyings;
    out.position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

@fragment
fn fs_main(in: Varyings) -> @location(0) vec4<f32> {
    return textureSample(frame, frame_sampler, in.uv);
}
";

/// Surface formats in the order they are preferred: 8 bits a channel, sRGB
/// first, so a linear render target is encoded for the display once.
const PREFERRED: [wgpu::TextureFormat; 4] = [
    wgpu::TextureFormat::Bgra8UnormSrgb,
    wgpu::TextureFormat::Rgba8UnormSrgb,
    wgpu::TextureFormat::Bgra8Unorm,
    wgpu::TextureFormat::Rgba8Unorm,
];

/// Whether a presented image in `format` can be read back as 4 bytes a texel.
pub(crate) fn readable(format: wgpu::TextureFormat) -> bool {
    PREFERRED.contains(&format)
}

/// The configuration a surface opens with on `adapter`, `width` × `height`.
pub(crate) fn configure(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<wgpu::SurfaceConfiguration> {
    let caps = surface.get_capabilities(adapter);
    let mut config = surface
        .get_default_config(adapter, width.max(1), height.max(1))
        .ok_or_else(|| {
            Error::new(
                Domain::Render,
                "rhi-wgpu",
                "the adapter cannot present to this window",
            )
            .with_recovery(Recovery::DisableSubsystem)
        })?;
    if let Some(format) = PREFERRED
        .iter()
        .find(|format| caps.formats.contains(format))
    {
        config.format = *format;
    }
    // FIFO is the one mode every platform guarantees; the default already is.
    config.present_mode = wgpu::PresentMode::Fifo;
    if caps.usages.contains(wgpu::TextureUsages::COPY_SRC) {
        // So a probe can read back what reached the window.
        config.usage |= wgpu::TextureUsages::COPY_SRC;
    }
    Ok(config)
}

/// Build the blit for `device`, writing `format`.
pub(crate) fn blit(device: &wgpu::Device, format: wgpu::TextureFormat) -> Blit {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("nexora present"),
        source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("nexora present"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nexora present"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("nexora present"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("nexora present"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        // Nearest: a texel lands on the window as it is, at any size, which
        // is what 16x16 art wants and what a readback can check exactly.
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    Blit {
        layout,
        pipeline,
        sampler,
    }
}

/// The next surface image, reconfiguring once if the surface changed under
/// it. The flag says the image no longer matches the window exactly.
pub(crate) fn acquire(
    presenter: &Presenter,
    device: &wgpu::Device,
) -> Result<(wgpu::SurfaceTexture, bool)> {
    for attempt in 0..2 {
        match presenter.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => return Ok((frame, false)),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => return Ok((frame, true)),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost
                if attempt == 0 =>
            {
                presenter.surface.configure(device, &presenter.config);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(
                    not_shown("the window is not taking frames now").with_recovery(Recovery::Retry)
                );
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err(
                    not_shown("the surface no longer matches the window: resize it")
                        .with_recovery(Recovery::Retry),
                );
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(not_shown("the surface was lost").with_recovery(Recovery::Manual));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(not_shown("the driver refused to hand out a surface image")
                    .with_recovery(Recovery::Manual));
            }
        }
    }
    Err(not_shown("the surface did not recover after reconfiguring").with_recovery(Recovery::Retry))
}

fn not_shown(message: &'static str) -> Error {
    Error::new(Domain::Render, "rhi-wgpu", message).with_context("during", "present")
}
