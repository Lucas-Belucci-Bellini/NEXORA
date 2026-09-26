//! The chunk pass, on the null backend and on a real driver.
//!
//! The GPU test does not compare the frame with a picture typed by hand. It
//! checks it against `nexora_render::reference`: a ray per pixel walked
//! through the voxel grid on the CPU. The rasteriser, the depth test, the
//! camera's matrices and the mesher all have to agree with a grid walk that
//! shares no code with any of them.

use nexora_camera::{Camera, CameraState, Projection, RenderOrigin};
use nexora_foundation::spatial::{BlockPos, WorldPosition, MAX_BLOCK_COORD};
use nexora_mesh::{mesh_region, Extent, SurfaceId, VoxelView};
use nexora_render::reference::check_frame;
use nexora_render::{ChunkPass, FrameStats};
use nexora_rhi::{CommandList, NullRhi, Rhi, TextureDesc, TextureFormat, Usage};
use nexora_rhi_wgpu::WgpuRhi;

const EDGE: u32 = 16;
const FOV_DEGREES: f64 = 60.0;
const SIZE: u32 = 128;

/// Terrain from 3 to 8 blocks high, and a floating slab above it that hides
/// part of the terrain from a camera looking down: the depth test has
/// something to decide.
struct Scene {
    base: BlockPos,
}

impl Scene {
    fn solid_local(x: i64, y: i64, z: i64) -> bool {
        if !(0..i64::from(EDGE)).contains(&x)
            || !(0..i64::from(EDGE)).contains(&y)
            || !(0..i64::from(EDGE)).contains(&z)
        {
            return false;
        }
        let height = 3 + (x * 7 + z * 3) % 6;
        let slab = y == 11 && (4..10).contains(&x) && (5..11).contains(&z);
        y < height || slab
    }

    fn solid(&self, p: BlockPos) -> bool {
        Self::solid_local(p.x - self.base.x, p.y - self.base.y, p.z - self.base.z)
    }

    fn region(&self) -> Extent {
        Extent::cubic(self.base, EDGE).unwrap()
    }
}

impl VoxelView for Scene {
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
        self.solid(position).then_some(SurfaceId(0))
    }
}

fn camera_for(scene: &Scene) -> (Camera, RenderOrigin) {
    let b = scene.base;
    let at =
        |x: f64, y: f64, z: f64| WorldPosition::new(b.x as f64 + x, b.y as f64 + y, b.z as f64 + z);
    let position = at(-7.25, 20.5, -6.75);
    let mut camera = Camera::new(
        position,
        Projection::perspective(FOV_DEGREES.to_radians(), 0.1, 200.0).unwrap(),
    )
    .unwrap();
    camera.look_at(at(8.0, 5.0, 8.0)).unwrap();
    (camera, RenderOrigin::containing(position).unwrap())
}

/// Draw `scene` into a fresh target and return what was drawn.
fn draw_frame(rhi: &mut WgpuRhi, scene: &Scene) -> (FrameStats, Vec<u8>, CameraState, Camera) {
    let color = rhi
        .create_texture(&TextureDesc {
            label: "pass test".into(),
            width: SIZE,
            height: SIZE,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::RENDER_TARGET | Usage::COPY_SRC,
        })
        .unwrap();
    let depth = rhi
        .create_texture(&TextureDesc {
            label: "pass test depth".into(),
            width: SIZE,
            height: SIZE,
            format: TextureFormat::Depth32Float,
            usage: Usage::RENDER_TARGET,
        })
        .unwrap();
    let pass = ChunkPass::new(rhi, TextureFormat::Rgba8Unorm).unwrap();
    let mesh = mesh_region(scene, scene.region());
    let mut frame = CommandList::new("pass test frame");
    let chunk = pass
        .upload(rhi, &mut frame, &mesh.opaque, scene.region())
        .unwrap();
    let (camera, origin) = camera_for(scene);
    let state = camera.sample(origin, SIZE, SIZE).unwrap();
    let stats = pass
        .record(&mut frame, &state, color, depth, &[chunk])
        .unwrap();
    let fence = rhi.submit(frame).unwrap();
    rhi.wait(fence).unwrap();
    let texels = rhi.read_texture(color).unwrap();
    chunk.destroy(rhi).unwrap();
    pass.destroy(rhi).unwrap();
    rhi.destroy_texture(color).unwrap();
    rhi.destroy_texture(depth).unwrap();
    rhi.poll().unwrap();
    (stats, texels, state, camera)
}

fn backend() -> Option<WgpuRhi> {
    match WgpuRhi::new() {
        Ok(rhi) => Some(rhi),
        Err(error) if std::env::var("NEXORA_GPU").as_deref() == Ok("none") => {
            eprintln!("not run: NEXORA_GPU=none ({error})");
            None
        }
        Err(error) => panic!(
            "no GPU adapter: {error}\n\
             install a Vulkan driver (Linux: mesa-vulkan-drivers) or set NEXORA_GPU=none"
        ),
    }
}

#[test]
fn every_pixel_shows_the_face_a_ray_through_it_hits_first() {
    let Some(mut rhi) = backend() else { return };
    let scene = Scene {
        base: BlockPos::new(32, 0, -48),
    };
    let (stats, texels, _, camera) = draw_frame(&mut rhi, &scene);
    assert_eq!(stats.drawn, 1);

    let check = check_frame(&scene, scene.region(), &camera, (SIZE, SIZE), &texels).unwrap();
    assert!(
        check.judged * 10 >= check.pixels * 8,
        "only {} of {} pixels were unambiguous",
        check.judged,
        check.pixels
    );
    assert_eq!(
        check.matching, check.judged,
        "{} of {} judged pixels match the ray cast",
        check.matching, check.judged
    );
    // The frame shows background, tops and at least two kinds of side.
    let mut colors: Vec<&[u8]> = texels.chunks_exact(4).collect();
    colors.sort_unstable();
    colors.dedup();
    assert!(colors.len() >= 4, "only {} distinct colours", colors.len());
    assert_eq!(rhi.allocated_bytes(), 0);
}

#[test]
fn the_same_scene_at_the_edge_of_the_world_draws_the_same_pixels() {
    // ADR-0029 end to end: vertices relative to their region, the region
    // relative to an integer origin near the camera. 2^40 blocks out, nothing
    // about the frame may change.
    let Some(mut rhi) = backend() else { return };
    let far = (MAX_BLOCK_COORD - (1 << 20)) & !15;
    let (_, centre, ..) = draw_frame(
        &mut rhi,
        &Scene {
            base: BlockPos::new(0, 0, 0),
        },
    );
    let (_, edge, ..) = draw_frame(
        &mut rhi,
        &Scene {
            base: BlockPos::new(far, 0, -far),
        },
    );
    assert_eq!(centre, edge);
}

#[test]
fn a_frame_records_and_submits_on_the_null_backend() {
    let mut rhi = NullRhi::new();
    let scene = Scene {
        base: BlockPos::ORIGIN,
    };
    let texture = |format, label: &str| TextureDesc {
        label: label.into(),
        width: SIZE,
        height: SIZE,
        format,
        usage: Usage::RENDER_TARGET,
    };
    let color = rhi
        .create_texture(&texture(TextureFormat::Rgba8Unorm, "c"))
        .unwrap();
    let depth = rhi
        .create_texture(&texture(TextureFormat::Depth32Float, "d"))
        .unwrap();
    let pass = ChunkPass::new(&mut rhi, TextureFormat::Rgba8Unorm).unwrap();
    let mut list = CommandList::new("null frame");
    let mesh = mesh_region(&scene, scene.region());
    let chunk = pass
        .upload(&mut rhi, &mut list, &mesh.opaque, scene.region())
        .unwrap();
    let empty_region = Extent::cubic(BlockPos::new(0, 64, 0), EDGE).unwrap();
    let empty = pass
        .upload(&mut rhi, &mut list, &mesh.cutout, empty_region)
        .unwrap();
    assert!(empty.vertices.is_none());

    let (camera, origin) = camera_for(&scene);
    let state = camera.sample(origin, SIZE, SIZE).unwrap();
    let stats = pass
        .record(&mut list, &state, color, depth, &[chunk, empty])
        .unwrap();
    assert_eq!(
        stats,
        FrameStats {
            drawn: 1,
            culled: 0,
            empty: 1,
            vertices: u64::from(chunk.vertex_count),
        }
    );
    assert_eq!(
        u64::from(chunk.vertex_count),
        mesh.opaque.vertex_count() / 4 * 6
    );
    let fence = rhi.submit(list).unwrap();
    rhi.wait(fence).unwrap();

    // Turned away from the chunk, the frame clears and draws nothing.
    let mut away = camera;
    away.set_orientation(camera.yaw() + std::f64::consts::PI, 0.0)
        .unwrap();
    let state = away.sample(origin, SIZE, SIZE).unwrap();
    let mut list = CommandList::new("null frame, looking away");
    let stats = pass
        .record(&mut list, &state, color, depth, &[chunk])
        .unwrap();
    assert_eq!((stats.drawn, stats.culled), (0, 1));
    let fence = rhi.submit(list).unwrap();
    rhi.wait(fence).unwrap();

    chunk.destroy(&mut rhi).unwrap();
    empty.destroy(&mut rhi).unwrap();
    pass.destroy(&mut rhi).unwrap();
}
