//! Evidence that the GPU did the work, not just that it accepted it.
//!
//! The conformance suite checks that a backend keeps the contract's rules. It
//! cannot see whether a single byte arrived, because the contract has no
//! readback. [`run`] closes that gap for this backend: it uploads a texture and
//! reads it back byte for byte, then draws into a render target and reads the
//! pixels the GPU shaded.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_rhi::{
    BufferDesc, Command, CommandList, PipelineDesc, Rhi, TextureDesc, TextureFormat, Usage,
    VertexAttribute, VertexFormat,
};

use crate::{conformance_shaders, WgpuRhi};

/// The colour [`crate::CONFORMANCE_WGSL`]'s fragment shader writes, as RGBA8.
pub const SHADED: [u8; 4] = [255, 0, 255, 255];

/// What [`run`] observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proof {
    /// Bytes uploaded to a texture and read back identical.
    pub uploaded_bytes: usize,
    /// Texels in the draw's target.
    pub target_texels: usize,
    /// Of those, how many the GPU shaded with the fragment shader's colour.
    pub shaded_texels: usize,
}

/// Upload, read back, draw, read back. Leaves the backend holding nothing.
///
/// # Errors
///
/// A readback differs from what was written, the draw shaded the wrong
/// texels, or the backend refused a step.
pub fn run(rhi: &mut WgpuRhi) -> Result<Proof> {
    // A 16x16 RGBA8 texture: the first visual generation's size. 1 KiB whose
    // rows are shorter than the 256-byte copy alignment, so padding is
    // exercised in both directions.
    let edge = 16;
    let pattern: Vec<u8> = (0..edge * edge * 4).map(|i| (i * 7 % 251) as u8).collect();
    let texture = rhi.create_texture(&TextureDesc {
        label: "proof upload".into(),
        width: edge,
        height: edge,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: Usage::SAMPLED | Usage::COPY_DST | Usage::COPY_SRC,
    })?;
    let mut upload = CommandList::new("proof upload");
    upload.push(Command::WriteTexture {
        texture,
        data: pattern.clone(),
    });
    let fence = rhi.submit(upload)?;
    rhi.wait(fence)?;
    let back = rhi.read_texture(texture)?;
    rhi.destroy_texture(texture)?;
    if back != pattern {
        let first = back
            .iter()
            .zip(&pattern)
            .position(|(a, b)| a != b)
            .unwrap_or(back.len().min(pattern.len()));
        return Err(wrong("a texture read back is not what was uploaded")
            .with_context("first_difference", first.to_string()));
    }

    // One triangle that covers the whole target: every texel must be shaded.
    let target_edge = 4;
    let target = rhi.create_texture(&TextureDesc {
        label: "proof target".into(),
        width: target_edge,
        height: target_edge,
        format: TextureFormat::Rgba8Unorm,
        usage: Usage::RENDER_TARGET | Usage::COPY_SRC,
    })?;
    let vertices: Vec<u8> = [
        [-1.0f32, -1.0, 0.0, 1.0],
        [3.0, -1.0, 0.0, 1.0],
        [-1.0, 3.0, 0.0, 1.0],
    ]
    .iter()
    .flatten()
    .flat_map(|value| value.to_le_bytes())
    .collect();
    let buffer = rhi.create_buffer(&BufferDesc {
        label: "proof triangle".into(),
        size: vertices.len() as u64,
        usage: Usage::VERTEX | Usage::COPY_DST,
    })?;
    let shaders = conformance_shaders();
    let pipeline = rhi.create_pipeline(&PipelineDesc {
        label: "proof".into(),
        vertex: shaders.vertex,
        fragment: shaders.fragment,
        vertex_stride: 16,
        attributes: vec![VertexAttribute::position(VertexFormat::Float32x4)],
        bindings: Vec::new(),
        depth: None,
        targets: vec![TextureFormat::Rgba8Unorm],
    })?;
    let mut draw = CommandList::new("proof draw");
    draw.push(Command::WriteBuffer {
        buffer,
        offset: 0,
        data: vertices,
    })
    .push(Command::Draw {
        pipeline,
        buffer,
        target,
        depth: None,
        bindings: Vec::new(),
        vertices: 3,
    });
    let fence = rhi.submit(draw)?;
    rhi.wait(fence)?;
    let pixels = rhi.read_texture(target)?;
    rhi.destroy_pipeline(pipeline)?;
    rhi.destroy_buffer(buffer)?;
    rhi.destroy_texture(target)?;
    rhi.poll()?;
    let target_texels = pixels.len() / 4;
    let shaded_texels = pixels
        .chunks_exact(4)
        .filter(|texel| *texel == SHADED)
        .count();
    if shaded_texels != target_texels {
        return Err(wrong("the draw did not shade every texel of its target")
            .with_context("shaded", shaded_texels.to_string())
            .with_context("texels", target_texels.to_string()));
    }
    Ok(Proof {
        uploaded_bytes: pattern.len(),
        target_texels,
        shaded_texels,
    })
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "rhi-wgpu-proof", message).with_recovery(Recovery::DisableSubsystem)
}

/// What [`bound`] observed, each count out of [`Bound::texels`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bound {
    /// Texels in the 16x16 target.
    pub texels: usize,
    /// Texels that show the sampled image exactly, after the first draw.
    pub sampled: usize,
    /// Texels a farther draw left alone, as the depth test must.
    pub kept_by_depth: usize,
    /// Texels a nearer draw replaced with the image times the uniform tint.
    pub tinted: usize,
}

/// ADR-0028 on this device: a vertex layout with a texture coordinate, a
/// uniform, a texture sampled through a nearest sampler, and a depth test.
/// Each step is read back. Leaves the backend holding nothing.
///
/// The depth test is the engine's own convention, **reverse-Z** (ADR-0029):
/// depth is cleared to `0`, the far end, and a fragment passes when its depth
/// is greater, which is nearer.
///
/// # Errors
///
/// A count falls short of every texel, or the backend refused a step.
pub fn bound(rhi: &mut WgpuRhi) -> Result<Bound> {
    use nexora_rhi::conformance::{BOUND_ATTRIBUTES, BOUND_SLOTS};
    use nexora_rhi::{Binding, ClearValue, Compare, DepthState};

    const EDGE: u32 = 16;
    let quadrants = [
        [255u8, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ];
    let mut image_texels = Vec::with_capacity((EDGE * EDGE * 4) as usize);
    for y in 0..EDGE {
        for x in 0..EDGE {
            let q = usize::from(x >= EDGE / 2) + 2 * usize::from(y >= EDGE / 2);
            image_texels.extend_from_slice(&quadrants[q]);
        }
    }
    let color = |usage| TextureDesc {
        label: "proof bound".into(),
        width: EDGE,
        height: EDGE,
        format: TextureFormat::Rgba8Unorm,
        usage,
    };
    let image = rhi.create_texture(&color(Usage::SAMPLED | Usage::COPY_DST))?;
    let target = rhi.create_texture(&color(Usage::RENDER_TARGET | Usage::COPY_SRC))?;
    let depth = rhi.create_texture(&TextureDesc {
        format: TextureFormat::Depth32Float,
        ..color(Usage::RENDER_TARGET)
    })?;
    let tint = rhi.create_buffer(&BufferDesc {
        label: "proof tint".into(),
        size: 16,
        usage: Usage::UNIFORM | Usage::COPY_DST,
    })?;
    let shaders = conformance_shaders();
    let pipeline = rhi.create_pipeline(&PipelineDesc {
        label: "proof bound".into(),
        vertex: shaders.bound_vertex,
        fragment: shaders.bound_fragment,
        vertex_stride: 24,
        attributes: BOUND_ATTRIBUTES.to_vec(),
        bindings: BOUND_SLOTS.to_vec(),
        depth: Some(DepthState {
            compare: Compare::Greater,
            write: true,
        }),
        targets: vec![TextureFormat::Rgba8Unorm],
    })?;
    // A triangle covering the target at depth `z`, with texture coordinates
    // that put texel (0, 0) at the top-left, as the RHI lays texels out.
    let triangle = |z: f32| -> Vec<u8> {
        [
            [-1.0f32, -1.0, z, 1.0, 0.0, 1.0],
            [3.0, -1.0, z, 1.0, 2.0, 1.0],
            [-1.0, 3.0, z, 1.0, 0.0, -1.0],
        ]
        .iter()
        .flatten()
        .flat_map(|value| value.to_le_bytes())
        .collect()
    };
    // Middle, farther and nearer, each in a buffer of its own. Reverse-Z:
    // a smaller depth is farther.
    let mut triangles = Vec::with_capacity(3);
    for z in [0.5f32, 0.3, 0.7] {
        let buffer = rhi.create_buffer(&BufferDesc {
            label: "proof triangle".into(),
            size: 72,
            usage: Usage::VERTEX | Usage::COPY_DST,
        })?;
        triangles.push((buffer, triangle(z)));
    }
    let rgba = |c: [f32; 4]| -> Vec<u8> { c.iter().flat_map(|v| v.to_le_bytes()).collect() };
    let draw = |buffer| Command::Draw {
        pipeline,
        buffer,
        target,
        depth: Some(depth),
        bindings: vec![
            Binding::Uniform(tint),
            Binding::Texture(image),
            Binding::Sampler,
        ],
        vertices: 3,
    };
    let matching = |got: &[u8], want: &[u8]| {
        got.chunks_exact(4)
            .zip(want.chunks_exact(4))
            .filter(|(a, b)| a == b)
            .count()
    };

    // Middle, untinted: the image, texel for texel.
    let mut frame = CommandList::new("proof bound frame");
    frame
        .push(Command::Clear {
            texture: target,
            value: ClearValue::Color([0.0, 0.0, 0.0, 1.0]),
        })
        .push(Command::Clear {
            texture: depth,
            value: ClearValue::Depth(0.0),
        })
        .push(Command::WriteTexture {
            texture: image,
            data: image_texels.clone(),
        });
    for (buffer, bytes) in &triangles {
        frame.push(Command::WriteBuffer {
            buffer: *buffer,
            offset: 0,
            data: bytes.clone(),
        });
    }
    frame
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([1.0, 1.0, 1.0, 1.0]),
        })
        .push(draw(triangles[0].0));
    let fence = rhi.submit(frame)?;
    rhi.wait(fence)?;
    let sampled = matching(&rhi.read_texture(target)?, &image_texels);

    // Farther, tinted black: the depth test must reject every fragment.
    let mut farther = CommandList::new("proof farther");
    farther
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([0.0, 0.0, 0.0, 1.0]),
        })
        .push(draw(triangles[1].0));
    let fence = rhi.submit(farther)?;
    rhi.wait(fence)?;
    let kept_by_depth = matching(&rhi.read_texture(target)?, &image_texels);

    // Nearer, tinted green: passes, and the tint multiplies the image.
    let mut nearer = CommandList::new("proof nearer");
    nearer
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([0.0, 1.0, 0.0, 1.0]),
        })
        .push(draw(triangles[2].0));
    let fence = rhi.submit(nearer)?;
    rhi.wait(fence)?;
    let green: Vec<u8> = image_texels
        .chunks_exact(4)
        .flat_map(|t| [0, t[1], 0, t[3]])
        .collect();
    let tinted = matching(&rhi.read_texture(target)?, &green);

    for (buffer, _) in triangles {
        rhi.destroy_buffer(buffer)?;
    }
    rhi.destroy_buffer(tint)?;
    rhi.destroy_pipeline(pipeline)?;
    for texture in [image, target, depth] {
        rhi.destroy_texture(texture)?;
    }
    rhi.poll()?;

    let texels = (EDGE * EDGE) as usize;
    let result = Bound {
        texels,
        sampled,
        kept_by_depth,
        tinted,
    };
    for (count, message) in [
        (sampled, "a sampled image did not land texel for texel"),
        (kept_by_depth, "a farther draw passed the depth test"),
        (tinted, "a nearer draw was not tinted by its uniform"),
    ] {
        if count != texels {
            return Err(wrong(message)
                .with_context("texels", count.to_string())
                .with_context("of", texels.to_string()));
        }
    }
    Ok(result)
}
