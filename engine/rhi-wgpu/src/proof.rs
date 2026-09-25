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
