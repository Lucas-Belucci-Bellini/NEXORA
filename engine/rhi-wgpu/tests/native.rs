//! The native backend on whatever GPU this machine has: a real one, or Mesa's
//! lavapipe (software Vulkan) in CI.
//!
//! No adapter is a failure, not a skip. A machine that has no GPU says so with
//! `NEXORA_GPU=none`, and then these tests report that they did not run.

use std::sync::Arc;

use nexora_foundation::error::Recovery;
use nexora_foundation::memory::{MemoryBudget, MemoryClass, MemoryLedger, PoolSpec};
use nexora_rhi::conformance::{self, CASES};
use nexora_rhi::{
    BufferDesc, Command, CommandList, PipelineDesc, Rhi, ShaderStage, TextureDesc, TextureFormat,
    Usage, VertexAttribute, VertexFormat,
};
use nexora_rhi_wgpu::{conformance_shaders, proof, WgpuRhi};

fn backend() -> Option<WgpuRhi> {
    match WgpuRhi::new() {
        Ok(rhi) => Some(rhi),
        Err(error) if std::env::var("NEXORA_GPU").as_deref() == Ok("none") => {
            eprintln!("not run: NEXORA_GPU=none ({error})");
            None
        }
        Err(error) => panic!(
            "no GPU adapter: {error}\n\
             install a Vulkan driver (Linux: mesa-vulkan-drivers) or set NEXORA_GPU=none \
             to declare that this machine has none"
        ),
    }
}

#[test]
fn the_native_backend_passes_every_conformance_case() {
    let Some(mut rhi) = backend() else { return };
    let report = conformance::run(&mut rhi, &conformance_shaders()).unwrap();
    assert_eq!(report.backend, "wgpu");
    assert_eq!(report.passed, CASES.to_vec());
    assert_eq!(rhi.live(), (0, 0, 0));
    assert_eq!(rhi.allocated_bytes(), 0);
}

#[test]
fn bytes_reach_the_gpu_and_a_draw_shades_every_texel() {
    let Some(mut rhi) = backend() else { return };
    let proof = proof::run(&mut rhi).unwrap();
    assert_eq!(proof.uploaded_bytes, 16 * 16 * 4);
    assert_eq!(proof.shaded_texels, proof.target_texels);
    assert_eq!(proof.target_texels, 16);
    assert_eq!(rhi.allocated_bytes(), 0);
}

#[test]
fn shaders_are_validated_by_the_driver_stack_not_only_shaped() {
    let Some(mut rhi) = backend() else { return };
    assert!(rhi.capabilities().validates_shaders);
    let broken = ShaderStage {
        entry: "vs_main".into(),
        code: b"@vertex fn vs_main() -> this is not WGSL".to_vec(),
    };
    let good = conformance_shaders();
    let error = rhi
        .create_pipeline(&PipelineDesc {
            label: "broken".into(),
            vertex: broken,
            fragment: good.fragment.clone(),
            vertex_stride: 16,
            attributes: vec![VertexAttribute::position(VertexFormat::Float32x4)],
            bindings: Vec::new(),
            depth: None,
            targets: vec![TextureFormat::Rgba8Unorm],
        })
        .expect_err("not WGSL");
    assert_eq!(error.recovery(), Recovery::Reject);
    assert!(error.to_string().contains("did not compile"), "{error}");

    let missing_entry = ShaderStage {
        entry: "no_such_function".into(),
        ..good.vertex.clone()
    };
    assert!(rhi
        .create_pipeline(&PipelineDesc {
            label: "missing entry".into(),
            vertex: missing_entry,
            fragment: good.fragment,
            vertex_stride: 16,
            attributes: vec![VertexAttribute::position(VertexFormat::Float32x4)],
            bindings: Vec::new(),
            depth: None,
            targets: vec![TextureFormat::Rgba8Unorm],
        })
        .is_err());
    assert_eq!(rhi.live(), (0, 0, 0), "nothing half-built was kept");
}

#[test]
fn a_list_runs_in_the_order_it_was_recorded() {
    // Draw with a full-target triangle, then overwrite the same vertex buffer
    // with zeros in the same list. If the write jumped ahead of the draw, as a
    // queued write would, the triangle would be degenerate and shade nothing.
    let Some(mut rhi) = backend() else { return };
    let triangle: Vec<u8> = [
        [-1.0f32, -1.0, 0.0, 1.0],
        [3.0, -1.0, 0.0, 1.0],
        [-1.0, 3.0, 0.0, 1.0],
    ]
    .iter()
    .flatten()
    .flat_map(|value| value.to_le_bytes())
    .collect();
    let buffer = rhi
        .create_buffer(&BufferDesc {
            label: "vertices".into(),
            size: triangle.len() as u64,
            usage: Usage::VERTEX | Usage::COPY_DST,
        })
        .unwrap();
    let target = rhi
        .create_texture(&TextureDesc {
            label: "target".into(),
            width: 4,
            height: 4,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::RENDER_TARGET | Usage::COPY_SRC,
        })
        .unwrap();
    let shaders = conformance_shaders();
    let pipeline = rhi
        .create_pipeline(&PipelineDesc {
            label: "order".into(),
            vertex: shaders.vertex,
            fragment: shaders.fragment,
            vertex_stride: 16,
            attributes: vec![VertexAttribute::position(VertexFormat::Float32x4)],
            bindings: Vec::new(),
            depth: None,
            targets: vec![TextureFormat::Rgba8Unorm],
        })
        .unwrap();
    let zeros = vec![0; triangle.len()];
    let mut list = CommandList::new("draw then overwrite");
    list.push(Command::WriteBuffer {
        buffer,
        offset: 0,
        data: triangle,
    })
    .push(Command::Draw {
        pipeline,
        buffer,
        target,
        depth: None,
        bindings: Vec::new(),
        vertices: 3,
    })
    .push(Command::WriteBuffer {
        buffer,
        offset: 0,
        data: zeros,
    });
    let fence = rhi.submit(list).unwrap();
    rhi.wait(fence).unwrap();
    let pixels = rhi.read_texture(target).unwrap();
    assert!(
        pixels.chunks_exact(4).all(|texel| texel == proof::SHADED),
        "the overwrite ran before the draw: {pixels:?}"
    );
}

#[test]
fn a_lost_device_refuses_everything_until_recreated_and_old_handles_stay_dead() {
    let Some(mut rhi) = backend() else { return };
    let before = rhi
        .create_buffer(&BufferDesc {
            label: "before".into(),
            size: 16,
            usage: Usage::COPY_DST,
        })
        .unwrap();
    rhi.lose_device();
    assert!(rhi.device_lost());
    let error = rhi
        .create_buffer(&BufferDesc {
            label: "while lost".into(),
            size: 16,
            usage: Usage::COPY_DST,
        })
        .expect_err("lost");
    assert_eq!(error.recovery(), Recovery::DisableSubsystem);

    rhi.recreate().unwrap();
    assert!(!rhi.device_lost());
    assert_eq!(rhi.allocated_bytes(), 0);
    let mut stale = CommandList::new("stale");
    stale.push(Command::WriteBuffer {
        buffer: before,
        offset: 0,
        data: vec![0; 4],
    });
    assert!(rhi.submit(stale).is_err(), "a handle from the lost device");
    // The new device works end to end.
    let report = conformance::run(&mut rhi, &conformance_shaders()).unwrap();
    assert_eq!(report.passed.len(), CASES.len());
}

#[test]
fn device_memory_is_accounted_to_the_owner_pool() {
    let Some(mut rhi) = backend() else { return };
    let ledger = MemoryLedger::new();
    let pool = ledger
        .register(PoolSpec::new(
            "rhi.wgpu",
            MemoryClass::Gpu,
            MemoryBudget::capacity(4096).unwrap(),
        ))
        .unwrap();
    rhi.attach(Arc::clone(&pool));
    let texture = rhi
        .create_texture(&TextureDesc {
            label: "counted".into(),
            width: 16,
            height: 16,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::SAMPLED,
        })
        .unwrap();
    assert_eq!(pool.current(), 1024);
    assert!(
        rhi.create_buffer(&BufferDesc {
            label: "over budget".into(),
            size: 4096,
            usage: Usage::VERTEX,
        })
        .is_err(),
        "the owner's budget refuses before the device does"
    );
    rhi.destroy_texture(texture).unwrap();
    assert_eq!(pool.current(), 0);
}

#[test]
fn without_a_window_the_backend_says_it_cannot_present_and_does_not() {
    let Some(mut rhi) = backend() else { return };
    assert!(!rhi.capabilities().presents);
    assert!(rhi.surface().is_none());
    let target = rhi
        .create_texture(&TextureDesc {
            label: "target".into(),
            width: 4,
            height: 4,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::RENDER_TARGET,
        })
        .unwrap();
    let error = rhi.present(target).expect_err("no surface");
    assert_eq!(error.recovery(), Recovery::DisableSubsystem);
    assert!(rhi.present_and_capture(target).is_err());
    assert!(rhi.resize(64, 64).is_err(), "nothing to resize");
    rhi.destroy_texture(target).unwrap();
    assert_eq!(rhi.allocated_bytes(), 0);
}

/// ADR-0028 on a driver: a texture sampled through a nearest sampler lands
/// on the target texel for texel, a uniform tints it, and the depth test keeps
/// the nearest draw. Each claim is read back, not assumed from acceptance.
#[test]
fn bindings_samplers_and_depth_reach_the_gpu() {
    use nexora_rhi::conformance::{BOUND_ATTRIBUTES, BOUND_SLOTS};
    use nexora_rhi::{Binding, ClearValue, Compare, DepthState};

    let Some(mut rhi) = backend() else { return };
    const EDGE: u32 = 16;
    let quadrants = [
        [255u8, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ];
    let mut image_texels = Vec::new();
    for y in 0..EDGE {
        for x in 0..EDGE {
            let q = usize::from(x >= EDGE / 2) + 2 * usize::from(y >= EDGE / 2);
            image_texels.extend_from_slice(&quadrants[q]);
        }
    }
    let color = |usage| TextureDesc {
        label: "bound".into(),
        width: EDGE,
        height: EDGE,
        format: TextureFormat::Rgba8Unorm,
        usage,
    };
    let image = rhi
        .create_texture(&color(Usage::SAMPLED | Usage::COPY_DST))
        .unwrap();
    let target = rhi
        .create_texture(&color(Usage::RENDER_TARGET | Usage::COPY_SRC))
        .unwrap();
    let depth = rhi
        .create_texture(&TextureDesc {
            format: TextureFormat::Depth32Float,
            usage: Usage::RENDER_TARGET,
            ..color(Usage::RENDER_TARGET)
        })
        .unwrap();
    let tint = rhi
        .create_buffer(&BufferDesc {
            label: "tint".into(),
            size: 16,
            usage: Usage::UNIFORM | Usage::COPY_DST,
        })
        .unwrap();
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
    let shaders = conformance_shaders();
    let pipeline = rhi
        .create_pipeline(&PipelineDesc {
            label: "bound".into(),
            vertex: shaders.bound_vertex,
            fragment: shaders.bound_fragment,
            vertex_stride: 24,
            attributes: BOUND_ATTRIBUTES.to_vec(),
            bindings: BOUND_SLOTS.to_vec(),
            depth: Some(DepthState {
                compare: Compare::Less,
                write: true,
            }),
            targets: vec![TextureFormat::Rgba8Unorm],
        })
        .unwrap();
    let rgba = |c: [f32; 4]| -> Vec<u8> { c.iter().flat_map(|v| v.to_le_bytes()).collect() };

    // Three triangles, middle, farther and nearer, each in its own buffer.
    let buffers: Vec<_> = [0.5f32, 0.7, 0.3]
        .iter()
        .map(|z| {
            let buffer = rhi
                .create_buffer(&BufferDesc {
                    label: "triangle".into(),
                    size: 72,
                    usage: Usage::VERTEX | Usage::COPY_DST,
                })
                .unwrap();
            (buffer, triangle(*z))
        })
        .collect();

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
    let mut frame = CommandList::new("bound frame");
    frame
        .push(Command::Clear {
            texture: target,
            value: ClearValue::Color([0.0, 0.0, 0.0, 1.0]),
        })
        .push(Command::Clear {
            texture: depth,
            value: ClearValue::Depth(1.0),
        })
        .push(Command::WriteTexture {
            texture: image,
            data: image_texels.clone(),
        });
    for (buffer, bytes) in &buffers {
        frame.push(Command::WriteBuffer {
            buffer: *buffer,
            offset: 0,
            data: bytes.clone(),
        });
    }
    // Middle, untinted: the image, texel for texel.
    frame
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([1.0, 1.0, 1.0, 1.0]),
        })
        .push(draw(buffers[0].0));
    let fence = rhi.submit(frame).unwrap();
    rhi.wait(fence).unwrap();
    assert_eq!(
        rhi.read_texture(target).unwrap(),
        image_texels,
        "the sampled image did not land texel for texel"
    );

    // Farther, tinted black: the depth test must reject every fragment.
    let mut farther = CommandList::new("farther");
    farther
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([0.0, 0.0, 0.0, 1.0]),
        })
        .push(draw(buffers[1].0));
    let fence = rhi.submit(farther).unwrap();
    rhi.wait(fence).unwrap();
    assert_eq!(
        rhi.read_texture(target).unwrap(),
        image_texels,
        "a farther draw passed the depth test"
    );

    // Nearer, tinted green: passes, and the tint multiplies the image.
    let mut nearer = CommandList::new("nearer");
    nearer
        .push(Command::WriteBuffer {
            buffer: tint,
            offset: 0,
            data: rgba([0.0, 1.0, 0.0, 1.0]),
        })
        .push(draw(buffers[2].0));
    let fence = rhi.submit(nearer).unwrap();
    rhi.wait(fence).unwrap();
    let expected: Vec<u8> = image_texels
        .chunks_exact(4)
        .flat_map(|t| [0, t[1], 0, t[3]])
        .collect();
    assert_eq!(
        rhi.read_texture(target).unwrap(),
        expected,
        "a nearer, tinted draw did not replace the image"
    );

    for (buffer, _) in buffers {
        rhi.destroy_buffer(buffer).unwrap();
    }
    rhi.destroy_buffer(tint).unwrap();
    rhi.destroy_pipeline(pipeline).unwrap();
    for texture in [image, target, depth] {
        rhi.destroy_texture(texture).unwrap();
    }
    rhi.poll().unwrap();
    assert_eq!(rhi.live(), (0, 0, 0));
}
