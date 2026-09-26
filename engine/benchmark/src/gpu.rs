//! The plan's "RHI" stage, on the native backend (ADR-0026, DEBT-0008).
//!
//! Measured headless, on whatever adapter this machine has: a real GPU, or a
//! software driver such as Mesa's lavapipe in CI. The adapter is part of the
//! result, and the report prints it. A number from lavapipe is the CPU
//! emulating a GPU, and reading it as a GPU's would be wrong.
//!
//! Correctness is checked before anything is timed, as `compare-stacks.sh`
//! refuses to time until the digests agree. The upload is read back byte for
//! byte and the draw must shade every texel. A backend that is fast because it
//! does nothing would otherwise produce the best row in the table.
//!
//! No window: the benchmark stays runnable where there is no display. Frame
//! time ([`frame_time`]) draws into a texture; presentation is the window
//! probe's to measure.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_rhi::{
    BufferDesc, Command, CommandList, PipelineDesc, Rhi, TextureDesc, TextureFormat, Usage,
    VertexAttribute, VertexFormat,
};
use nexora_rhi_wgpu::{conformance_shaders, proof, WgpuRhi};

use crate::{
    consume, measure, measure_throughput, record_bytes, record_quantity, Budget, Measurement,
};

/// Edge of the first visual generation's textures, and of the draw target.
const EDGE: u32 = 16;

/// Bytes of one 16x16 RGBA8 texture.
const TEXTURE_BYTES: u64 = (EDGE * EDGE * 4) as u64;

/// Textures in the first visual generation, uploaded in one submission.
const FIRST_GENERATION: usize = 16;

/// Bytes per vertex while the contract has no vertex format: a position of
/// four `f32`, the interim rule DEBT-0046 records.
const VERTEX_BYTES: u64 = 16;

/// What the RHI stage found on this machine.
#[derive(Debug)]
pub struct RhiStage {
    /// The measurements, empty when nothing could be measured.
    pub measurements: Vec<Measurement>,
    /// The adapter measured, as `name (backend, kind)`.
    pub adapter: Option<String>,
    /// Why nothing was measured, when nothing was.
    pub gap: Option<&'static str>,
}

/// Why the stage did not run when this machine says it has no GPU.
pub const DECLARED_NO_GPU: &str = "NEXORA_GPU=none: this machine declares no GPU";

/// Why the stage did not run when no driver answered.
pub const NO_ADAPTER: &str = "no GPU adapter answered: no Vulkan, Direct3D 12 or Metal driver";

/// Measure the native backend.
///
/// `mesh_vertices` sizes the vertex upload: the vertex count of the meshed
/// region the `mesh` suite measures, so the two stages describe one workload.
///
/// # Errors
///
/// The adapter refused work it had accepted, or a correctness check failed:
/// an upload did not read back identical, a draw did not shade its target, or
/// the stage left device memory behind. None of these is a gap; each is a
/// wrong answer.
pub fn rhi(budget: Budget, mesh_vertices: u64) -> Result<RhiStage> {
    if std::env::var("NEXORA_GPU").as_deref() == Ok("none") {
        return Ok(RhiStage {
            measurements: Vec::new(),
            adapter: None,
            gap: Some(DECLARED_NO_GPU),
        });
    }
    let Ok(mut rhi) = WgpuRhi::new() else {
        return Ok(RhiStage {
            measurements: Vec::new(),
            adapter: None,
            gap: Some(NO_ADAPTER),
        });
    };
    let class = rhi.adapter();
    let adapter = format!("{} ({}, {})", class.name, class.backend, class.kind);

    // Correctness first: the same proof `nexora-rhi-probe` runs.
    let checked = proof::run(&mut rhi)?;
    if checked.shaded_texels != checked.target_texels {
        return Err(wrong("the draw did not shade its whole target"));
    }

    let mut out = Vec::new();

    // Opening a device is a startup cost, paid once per process, and slow
    // enough that a handful of samples is all the budget allows.
    let refused = std::cell::Cell::new(0u32);
    out.push(measure(
        "rhi.device_open",
        "Find the adapter and open a device on it, with no surface: the RHI's startup",
        Budget {
            warmup_iterations: 0,
            samples: budget.samples.min(5),
            iterations_per_sample: 1,
        },
        || match WgpuRhi::new() {
            Ok(opened) => {
                consume(opened.live());
            }
            Err(_) => refused.set(refused.get() + 1),
        },
    ));
    if refused.get() > 0 {
        return Err(wrong("the adapter opened a device once and then refused"));
    }

    let failure = std::cell::RefCell::new(None::<Error>);
    let note = |result: Result<()>| {
        if let Err(error) = result {
            failure.borrow_mut().get_or_insert(error);
        }
    };

    // The cheapest thing a frame waits on: one empty submission and its fence.
    out.push(measure(
        "rhi.fence_roundtrip",
        "Submit a list holding only a marker and wait for its fence: synchronization alone",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            note((|| {
                let mut list = CommandList::new("fence");
                list.push(Command::Marker("fence".into()));
                let fence = rhi.submit(list)?;
                rhi.wait(fence)
            })());
        },
    ));

    let texture_desc = TextureDesc {
        label: "bench texture".into(),
        width: EDGE,
        height: EDGE,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: Usage::SAMPLED | Usage::COPY_DST,
    };
    out.push(measure(
        "rhi.texture_create_destroy",
        "Create a 16x16 RGBA8 texture and destroy it with no work in flight",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            note((|| {
                let texture = rhi.create_texture(&texture_desc)?;
                rhi.destroy_texture(texture)
            })());
        },
    ));

    let pattern: Vec<u8> = (0..TEXTURE_BYTES).map(|i| (i * 7 % 251) as u8).collect();
    let texture = rhi.create_texture(&texture_desc)?;
    out.push(measure_throughput(
        "rhi.upload_texture_16",
        "Upload one 16x16 RGBA8 texture and wait for it: one write, one fence",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        TEXTURE_BYTES,
        || {
            note((|| {
                let mut list = CommandList::new("upload");
                list.push(Command::WriteTexture {
                    texture,
                    data: pattern.clone(),
                });
                let fence = rhi.submit(list)?;
                rhi.wait(fence)
            })());
        },
    ));
    rhi.destroy_texture(texture)?;

    // The slice's own shape (ADR-0025): the first generation's sixteen
    // albedos, sixteen writes in one submission, waited on with one fence.
    // The texels are a pattern, not the forge's output: the benchmark does not
    // build content, and the cost of a copy does not depend on its bytes.
    let set = (0..FIRST_GENERATION)
        .map(|_| rhi.create_texture(&texture_desc))
        .collect::<Result<Vec<_>>>()?;
    out.push(measure_throughput(
        "rhi.upload_first_generation",
        "Upload sixteen 16x16 textures in one submission and one fence, as the slice does",
        Budget {
            iterations_per_sample: 10,
            ..budget
        },
        TEXTURE_BYTES * FIRST_GENERATION as u64,
        || {
            note((|| {
                let mut list = CommandList::new("first generation");
                for texture in &set {
                    list.push(Command::WriteTexture {
                        texture: *texture,
                        data: pattern.clone(),
                    });
                }
                let fence = rhi.submit(list)?;
                rhi.wait(fence)
            })());
        },
    ));
    for texture in set {
        rhi.destroy_texture(texture)?;
    }

    // A vertex buffer the size of the meshed 16³ region's. The layout is the
    // interim position-only one; the size is what the mesher produced.
    let mesh_bytes = mesh_vertices.max(1) * VERTEX_BYTES;
    out.push(record_bytes(
        "rhi.mesh_16_vertex_bytes",
        "Vertex bytes for the meshed 16 cubed region, at 16 bytes a vertex (DEBT-0046)",
        mesh_bytes,
    ));
    let vertices = rhi.create_buffer(&BufferDesc {
        label: "bench mesh".into(),
        size: mesh_bytes,
        usage: Usage::VERTEX | Usage::COPY_DST,
    })?;
    let mesh_data = vec![0u8; usize::try_from(mesh_bytes).unwrap_or(usize::MAX)];
    out.push(measure_throughput(
        "rhi.upload_mesh_16",
        "Upload the meshed 16 cubed region's vertex buffer and wait for it",
        Budget {
            iterations_per_sample: 10,
            ..budget
        },
        mesh_bytes,
        || {
            note((|| {
                let mut list = CommandList::new("mesh");
                list.push(Command::WriteBuffer {
                    buffer: vertices,
                    offset: 0,
                    data: mesh_data.clone(),
                });
                let fence = rhi.submit(list)?;
                rhi.wait(fence)
            })());
        },
    ));
    rhi.destroy_buffer(vertices)?;

    // One draw into a 16x16 target, the size the first generation's art is
    // drawn at, with the triangle already on the device.
    let triangle: Vec<u8> = [
        [-1.0f32, -1.0, 0.0, 1.0],
        [3.0, -1.0, 0.0, 1.0],
        [-1.0, 3.0, 0.0, 1.0],
    ]
    .iter()
    .flatten()
    .flat_map(|value| value.to_le_bytes())
    .collect();
    let buffer = rhi.create_buffer(&BufferDesc {
        label: "bench triangle".into(),
        size: triangle.len() as u64,
        usage: Usage::VERTEX | Usage::COPY_DST,
    })?;
    let target = rhi.create_texture(&TextureDesc {
        label: "bench target".into(),
        width: EDGE,
        height: EDGE,
        format: TextureFormat::Rgba8Unorm,
        usage: Usage::RENDER_TARGET | Usage::COPY_SRC,
    })?;
    let shaders = conformance_shaders();
    let pipeline = rhi.create_pipeline(&PipelineDesc {
        label: "bench".into(),
        vertex: shaders.vertex,
        fragment: shaders.fragment,
        vertex_stride: VERTEX_BYTES as u32,
        attributes: vec![VertexAttribute::position(VertexFormat::Float32x4)],
        bindings: Vec::new(),
        depth: None,
        targets: vec![TextureFormat::Rgba8Unorm],
    })?;
    let mut upload = CommandList::new("triangle");
    upload.push(Command::WriteBuffer {
        buffer,
        offset: 0,
        data: triangle,
    });
    let fence = rhi.submit(upload)?;
    rhi.wait(fence)?;
    out.push(measure(
        "rhi.draw_16",
        "Draw one full-target triangle into a 16x16 target and wait for it",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            note((|| {
                let mut list = CommandList::new("draw");
                list.push(Command::Draw {
                    pipeline,
                    buffer,
                    target,
                    depth: None,
                    bindings: Vec::new(),
                    vertices: 3,
                });
                let fence = rhi.submit(list)?;
                rhi.wait(fence)
            })());
        },
    ));
    // The timed draws must have drawn: read the target back once more.
    let drawn = rhi.read_texture(target)?;
    if !drawn.chunks_exact(4).all(|texel| texel == proof::SHADED) {
        return Err(wrong("a timed draw did not shade its target"));
    }
    rhi.destroy_pipeline(pipeline)?;
    rhi.destroy_texture(target)?;
    rhi.destroy_buffer(buffer)?;

    if let Some(error) = failure.into_inner() {
        return Err(error);
    }
    rhi.poll()?;
    if rhi.allocated_bytes() != 0 || rhi.live() != (0, 0, 0) {
        return Err(
            wrong("the RHI stage left device memory or resources behind")
                .with_context("bytes", rhi.allocated_bytes().to_string()),
        );
    }

    Ok(RhiStage {
        measurements: out,
        adapter: Some(adapter),
        gap: None,
    })
}

/// Frame time (DEBT-0008): one frame of the first render pass, drawing the
/// meshed 16³ region through a camera, on this machine's adapter.
///
/// The same region the `mesh` suite meshes, from the same generated world,
/// through `nexora_render::ChunkPass` (reverse-Z, one submission, one fence).
/// Before timing, the frame is checked against a ray cast on the CPU
/// (`nexora_render::reference`): every pixel judged must show the face the
/// ray reaches first, and most pixels must be judged. After timing, the
/// target is read again and must still hold that frame.
///
/// Headless: the frame is drawn into a texture, not a window. What a window
/// adds is presentation, which the window probe measures, not the drawing.
///
/// # Errors
///
/// No adapter answered (call this only when [`rhi`] measured), the frame did
/// not match the reference, or the stage left device memory behind.
pub fn frame_time(budget: Budget) -> Result<Vec<Measurement>> {
    use nexora_camera::{Camera, Projection, RenderOrigin};
    use nexora_foundation::spatial::{BlockPos, WorldPosition};
    use nexora_mesh::{mesh_region, Extent};
    use nexora_render::reference::check_frame;
    use nexora_render::ChunkPass;
    use nexora_simulation::WorldSurfaces;

    const SIZE: u32 = 256;

    let world = crate::suites::populated_world()?;
    let surface = world.surface_height(8, 8);
    let view = WorldSurfaces::untextured(&world);
    let region = Extent::cubic(BlockPos::new(0, surface - 8, 0), 16)?;
    let mesh = mesh_region(&view, region);

    let position = WorldPosition::new(-10.5, surface as f64 + 14.0, -9.5);
    let mut camera = Camera::new(
        position,
        Projection::perspective(60f64.to_radians(), 0.1, 256.0)?,
    )?;
    camera.look_at(WorldPosition::new(8.0, surface as f64 - 2.0, 8.0))?;
    let state = camera.sample(RenderOrigin::containing(position)?, SIZE, SIZE)?;

    let mut rhi = WgpuRhi::new()?;
    let color = rhi.create_texture(&TextureDesc {
        label: "frame time".into(),
        width: SIZE,
        height: SIZE,
        format: TextureFormat::Rgba8Unorm,
        usage: Usage::RENDER_TARGET | Usage::COPY_SRC,
    })?;
    let depth = rhi.create_texture(&TextureDesc {
        label: "frame time depth".into(),
        width: SIZE,
        height: SIZE,
        format: TextureFormat::Depth32Float,
        usage: Usage::RENDER_TARGET,
    })?;
    let pass = ChunkPass::new(&mut rhi, TextureFormat::Rgba8Unorm)?;
    let mut upload = CommandList::new("frame time upload");
    let chunk = pass.upload(&mut rhi, &mut upload, &mesh.opaque, region)?;
    let fence = rhi.submit(upload)?;
    rhi.wait(fence)?;

    let draw = |rhi: &mut WgpuRhi| -> Result<nexora_render::FrameStats> {
        let mut frame = CommandList::new("frame");
        let stats = pass.record(&mut frame, &state, color, depth, &[chunk])?;
        let fence = rhi.submit(frame)?;
        rhi.wait(fence)?;
        Ok(stats)
    };

    // Correctness first.
    let stats = draw(&mut rhi)?;
    if stats.drawn != 1 {
        return Err(wrong("the frame did not draw the region it looks at"));
    }
    let checked = rhi.read_texture(color)?;
    let check = check_frame(&view, region, &camera, (SIZE, SIZE), &checked)?;
    if check.matching != check.judged || check.judged * 2 < check.pixels {
        return Err(
            wrong("the frame does not show what a ray cast says it must")
                .with_context("judged", check.judged.to_string())
                .with_context("matching", check.matching.to_string())
                .with_context("pixels", check.pixels.to_string()),
        );
    }

    let failure = std::cell::RefCell::new(None);
    let mut out = vec![measure(
        "frame.draw_chunk_16",
        "One frame at 256x256: clear, camera, the meshed 16 cubed region with reverse-Z depth, one submission, one fence",
        Budget {
            iterations_per_sample: 10,
            ..budget
        },
        || {
            if let Err(error) = draw(&mut rhi) {
                failure.borrow_mut().get_or_insert(error);
            }
        },
    )];
    if let Some(error) = failure.into_inner() {
        return Err(error);
    }
    if rhi.read_texture(color)? != checked {
        return Err(wrong("a timed frame did not draw the checked frame"));
    }
    out.push(record_quantity(
        "frame.chunk_16_vertices",
        "Vertices the frame draws: six per merged rectangle of the region",
        u64::from(chunk.vertex_count),
    ));
    out.push(record_quantity(
        "frame.pixels_judged",
        "Of the frame's 65,536 pixels, those checked against the CPU ray cast (all matched)",
        check.judged as u64,
    ));

    chunk.destroy(&mut rhi)?;
    pass.destroy(&mut rhi)?;
    rhi.destroy_texture(color)?;
    rhi.destroy_texture(depth)?;
    rhi.poll()?;
    if rhi.allocated_bytes() != 0 || rhi.live() != (0, 0, 0) {
        return Err(wrong(
            "the frame stage left device memory or resources behind",
        ));
    }
    Ok(out)
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "benchmark-rhi", message).with_recovery(Recovery::Manual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rhi_stage_measures_the_native_backend_or_says_why_not() {
        let budget = Budget {
            warmup_iterations: 0,
            samples: 1,
            iterations_per_sample: 1,
        };
        let stage = rhi(budget, 3228).expect("correct answers, or a declared gap");
        if std::env::var("NEXORA_GPU").as_deref() == Ok("none") {
            assert_eq!(stage.gap, Some(DECLARED_NO_GPU));
            assert!(stage.measurements.is_empty());
            return;
        }
        // No adapter is a failure here, as in `nexora-rhi-wgpu`'s tests: CI
        // installs one, and a machine without one says so.
        assert_eq!(
            stage.gap, None,
            "no adapter: set NEXORA_GPU=none to declare it"
        );
        assert!(stage.adapter.is_some());
        let names: Vec<_> = stage.measurements.iter().map(|m| m.name).collect();
        for expected in [
            "rhi.device_open",
            "rhi.fence_roundtrip",
            "rhi.texture_create_destroy",
            "rhi.upload_texture_16",
            "rhi.upload_first_generation",
            "rhi.mesh_16_vertex_bytes",
            "rhi.upload_mesh_16",
            "rhi.draw_16",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
        let bytes = stage
            .measurements
            .iter()
            .find(|m| m.name == "rhi.mesh_16_vertex_bytes")
            .expect("recorded");
        assert_eq!(bytes.median() as u64, 3228 * VERTEX_BYTES);
    }

    #[test]
    fn a_frame_is_checked_against_a_ray_cast_before_it_is_timed() {
        if std::env::var("NEXORA_GPU").as_deref() == Ok("none") {
            return;
        }
        let budget = Budget {
            warmup_iterations: 0,
            samples: 1,
            iterations_per_sample: 1,
        };
        let out = frame_time(budget).expect("a frame that matches the reference");
        let value = |name: &str| {
            out.iter()
                .find(|m| m.name == name)
                .unwrap_or_else(|| panic!("missing {name}"))
                .median() as u64
        };
        assert!(value("frame.draw_chunk_16") > 0);
        // Six vertices per merged rectangle of the region the mesh suite meshes.
        assert_eq!(value("frame.chunk_16_vertices") % 6, 0);
        assert!(value("frame.pixels_judged") * 2 >= 256 * 256);
    }
}
