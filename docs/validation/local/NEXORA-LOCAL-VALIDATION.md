# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `6ad11f295fdbb0dd20f6ea2a9e0e8dbf3f3f4c97` |
| branch | `main` |
| generated | 2026-09-29T15:54:59+00:00 |
| OS | Windows 10 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 70.2 | exit 0 |
| `tests` | **FAIL** | 207.0 | exit 101 |
| `headless_slice` | **PASS** | 0.9 | queries 76 answered, 2 refused; rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 76; result OK |
| `headless_slice_content` | **PASS** | 0.8 | queries 92 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 92; result OK |
| `forge_first_generation` | **PASS** | 0.2 | build nexora-first-generation: 16 materials: 16 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (32 resources) |
| `headless_slice_textures` | **PASS** | 0.5 | queries 44 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); content textures 16 (resolved, verified and decoded); rhi null backend, conformance 11/11 cases, no GPU initialized; rhi uploads 16 textures as RGBA8, 16384 bytes, one fence; memory 4 pools, worst nominal, 0 suspected leaks; probes verified 44; result OK |
| `rhi_native` | **PASS** | 0.5 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); conformance 11/11 cases on wgpu; upload 1024 bytes to a 16x16 texture, read back identical; draw 16 of 16 texels shaded by the GPU; bound 256 of 256 texels sampled, 256 kept by depth, 256 tinted by a uniform; cull 16 of 16 texels shaded counter-clockwise, 0 clockwise, culling back faces; result OK |
| `window` | **PASS** | 1.3 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 256x256 on win32, 0 redraws waited for it to take frames; surface 256x256 Bgra8UnormSrgb, Fifo; conformance 11/11 cases on wgpu, presenting; frame 65536 of 65536 window texels show the 16x16 target, read back from the surface; presented 61 frames, 60 of them by the probe; result OK |
| `client_mode` | **PASS** | 5.6 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 384x256 on win32; first frame 79391 of 98304 pixels judged by the ray cast, 79390 matching, 1 snapped edges, 0 back faces, read back from the surface; frames 120 shown, ended by frame limit; frame loop 24 ticks, 0 discarded; 120 target, 0 warning, 0 critical, 0 emergency (budget: doubling from the 50 ms step, not measured); frame wall median 9.90 ms, p95 10.48 ms, max 28.03 ms; 0.11 ms unattributed in all; result OK |
| `input_devices` | **FAIL** | 60.7 | exit 1 |
| `benchmark_cpu` | **PASS** | 9.8 | see the report's benchmark section |

## What this machine could not validate, and why

Not "not tested": the engine has no code for these yet, so no machine can.

| item | status | reason |
| --- | --- | --- |
| `rendering` | NOT_IMPLEMENTED | partly built: the first render pass (nexora-render) draws meshed chunk regions through the camera -- one 16^3 region into a texture and into a window (benchmark_cpu's frame-time and window stages) and nine chunk columns of a generated world in client mode (client_mode) -- every frame checked against a CPU ray cast, culling back faces over quads split at every corner (ADR-0033); textures and streaming into the pass are not built |

## Benchmark (CPU stages, and the RHI stage on this machine's adapter)

### Environment

| | |
| --- | --- |
| logical CPUs | 12 |
| build profile | `release` |
| architecture | `x86_64` |
| peak resident memory | unavailable |
| benchmark binary size | 7.5 MiB |
| GPU adapter (RHI stage) | AMD Radeon RX 6650 XT (vulkan, discretegpu) |

### Measurements

| measurement | median | p95 | rel. σ | note |
| --- | ---: | ---: | ---: | --- |
| `startup.world_create` | 1.27 µs | 1.33 µs | 3.4% | Create a world, register the block set, freeze the registry and bring it online |
| `spatial.section_of_division` | 1.8 ns | 1.9 ns | 5.1% | Block → section using div_euclid by a runtime extent (what the engine does) |
| `spatial.section_of_shift_runtime` | 6.0 ns | 6.6 ns | 3.1% | Block → section by shift, width read at runtime: what DEBT-0005 would actually build |
| `spatial.section_of_shift_const` | 6.0 ns | 6.5 ns | 2.6% | Block → section by shift with a compile-time width: the optimization's ceiling |
| `spatial.index_of` | 4.0 ns | 5.5 ns | 13.0% | Flatten a local position into a storage index, bounds-checked |
| `voxel.get_paletted` | 5.7 ns | 9.8 ns | 21.4% | Read one cell from a palette-indexed section |
| `voxel.get_uniform` | 4.0 ns | 4.3 ns | 3.6% | Read one cell from a uniform section (solid rock, the common case) |
| `voxel.set_existing_state` | 10.9 ns | 20.8 ns | 29.0% | Write a cell whose state is already in the palette |
| `voxel.first_write_to_uniform` | 122.5 ns | 175.0 ns | 21.8% | Break a uniform section into a palette: the allocation the air optimization defers |
| `voxel.compact_section` | 138.52 µs | 182.92 µs | 10.2% | Rebuild a section's palette from what is actually referenced |
| `chunk.change_feed_append` | 30.2 ns | 45.1 ns | 17.1% | One journalled voxel write with the change feed below its cap |
| `chunk.change_feed_at_cap` | 31.9 ns | 31.9 ns | 1.6% | The same write with the feed full, so each one discards the oldest entry |
| `chunk.change_feed_cap` | 4 096 | — | — | Entries a chunk's change feed holds before it starts discarding |
| `worldgen.chunk_16` | 166.30 µs | 178.30 µs | 3.2% | Generate one chunk column, 16³ sections (the plan's size) |
| `worldgen.chunk_32` | 1.26 ms | 1.75 ms | 21.3% | Generate one chunk column, 32³ sections (engine default) |
| `worldgen.surface_height` | 32.2 ns | 33.3 ns | 1.3% | One column's terrain height from the position-seeded stream |
| `entity.spawn` | 197.4 ns | 310.2 ns | 34.0% | Create one entity: slot allocation, component writes, persistent id |
| `entity.spawn_despawn_cycle` | 280.0 ns | 626.4 ns | 41.9% | Spawn then destroy: the slot reuse path a busy world runs constantly |
| `entity.resolve_handle` | 2.2 ns | 7.1 ns | 55.4% | Validate one handle: world, range, generation and liveness |
| `entity.step_1000` | 4.79 µs | 6.14 µs | 10.5% | Advance 1,000 entities by one tick: the batch walk over dense columns |
| `entity.query_type_1000` | 16.67 µs | 19.36 µs | 7.1% | Scan of 1,000 entities filtering by type: no position, so no index |
| `entity.query_tag_1000` | 19.57 µs | 21.04 µs | 4.8% | Scan of 1,000 entities filtering by tag: no position, so no index |
| `entity.query_radius_1000` | 11.09 µs | 16.37 µs | 17.6% | Radius query in the plan's dense 1,000-entity box: the index cannot narrow it |
| `entity.query_radius_100k` | 686.0 ns | 702.0 ns | 0.9% | Radius query over 100,000 entities spread across a world, through the index |
| `entity.query_chunk_100k` | 572.0 ns | 595.5 ns | 2.1% | Which of 100,000 entities are in one chunk column, through the index |
| `entity.query_radius_candidates_100k` | 41 | — | — | Entities a 16-block radius query examines, of 100,000 in the store |
| `entity.index_cells_100k` | 24 642 | — | — | Grid cells holding at least one of the 100,000 entities |
| `entity.step_1000_crossing_cells` | 162.09 µs | 177.84 µs | 4.5% | Advance 1,000 entities that all change grid cell every tick: the index at its worst |
| `entity.save_1000` | 537 MiB/s | 308.31 µs | 11.6% | Serialize 1,000 entities as logical state |
| `entity.load_1000` | 223 MiB/s | 804.43 µs | 12.0% | Restore 1,000 entities, re-resolving every persistent id |
| `entity.save_size_1000` | 141.6 KiB | — | — | Bytes of save section for 1,000 entities |
| `physics.character_step` | 741.1 ns | 752.5 ns | 27.0% | One character substep on generated terrain: gravity, sweep, ground, friction |
| `physics.thousand_bodies_step` | 165.88 µs | 243.43 µs | 18.1% | One substep of 1,000 awake dynamic bodies against generated terrain |
| `physics.thousand_bodies_step_flat` | 77.78 µs | 113.58 µs | 15.4% | The same substep against a flat fixture: the solver without the world lookup |
| `physics.world_reads_per_step` | 1 000 | — | — | Cells the world is asked about in one substep of 1,000 settled awake bodies |
| `physics.voxel_lookup` | 12.2 ns | 14.1 ns | 5.5% | One cell question answered by the world view, section already resident |
| `physics.axis_sweep` | 28.3 ns | 28.9 ns | 0.9% | One axis of a swept box against the voxel grid (paired with the C++ reference) |
| `physics.box_sweep` | 138.0 ns | 270.2 ns | 33.6% | Resolve one box move on three axes against generated terrain |
| `physics.depenetration_check` | 47.8 ns | 48.5 ns | 0.6% | The per-body test for having started inside terrain, when it has not |
| `physics.raycast_40m` | 822.3 ns | 853.5 ns | 1.9% | Walk a 40 m ray down through generated terrain until it hits |
| `physics.timestep_accumulate` | 3.3 ns | 3.4 ns | 1.6% | Fold one world tick into the fixed-step accumulator |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — | Bodies asleep after ten seconds: what sleeping actually saves |
| `physics.thousand_sleeping_step` | 800.0 ns | 815.0 ns | 0.7% | One substep with the same 1,000 bodies asleep |
| `streaming.idle_tick_r3` | 3.09 µs | 3.51 µs | 6.1% | A tick with nothing to do, 7x7 columns of interest: pure decision cost |
| `streaming.idle_tick_r12` | 82.44 µs | 94.04 µs | 6.1% | The same idle tick over 25x25 columns: how decision cost scales with radius |
| `streaming.walk_one_chunk` | 5.97 µs | 6.43 µs | 4.2% | A tick after the observer moves one column: decide, load the new ring, release the old |
| `streaming.fast_travel_r3` | 30.89 µs | 39.14 µs | 9.6% | Teleport and settle: release 49 columns and take 49 more, decision side only |
| `streaming.chunk_generate_cycle` | 1.02 ms | 1.14 ms | 4.2% | Activate a fresh column and drop it again: the cost of an unedited chunk |
| `streaming.chunk_retained_cycle` | 186.0 ns | 186.5 ns | 0.4% | Persist, drop and restore an edited column: retention instead of regeneration |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — | Memory an edited column occupies while it is evicted |
| `streaming.chunk_flushed_cycle` | 6.18 ms | 7.15 ms | 7.5% | Persist, flush to a region file, drop, and read the column back from disk |
| `streaming.retained_bytes_after_flush` | 0 B | — | — | Memory the same evicted column occupies once it is in its region file |
| `streaming.region_writes_per_eight_columns` | 4 | — | — | Region files a flush of eight columns touches, two columns to a region |
| `streaming.columns_of_interest_r12` | 625 | — | — | Columns a radius-12 observer makes the manager consider every tick |
| `jobs.submit_wait_roundtrip` | 6.02 µs | 9.61 µs | 25.0% | Submit one trivial job and block until it completes: the full scheduling round trip |
| `jobs.submit_only` | 4.87 µs | 7.26 µs | 35.8% | Enqueue a job without waiting: the cost a producer pays |
| `jobs.batch_1000_barrier` | 5.15 ms | 6.86 ms | 22.7% | Submit 1,000 jobs one at a time and barrier: a wake-up per job |
| `jobs.batch_1000_barrier_submit_all` | 849.90 µs | 897.20 µs | 3.4% | The same 1,000 jobs handed over as one batch: one wake-up for the wave |
| `jobs.submit_all_1000` | 109.10 µs | 213.80 µs | 46.2% | Hand over 1,000 jobs as one batch, without waiting: the producer's cost |
| `jobs.wakeups_per_1000_submitted` | 1 000 | — | — | Condvar wake-ups a wave of 1,000 jobs issues, submitted one at a time |
| `jobs.wakeups_per_1000_submit_all` | 8 | — | — | Condvar wake-ups the same wave issues through submit_all |
| `frame.schedule_advance` | 0.0 ns | 300.0 ns | 166.7% | The fixed-timestep accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | 0.0 ns | 100.0 ns | 132.5% | A whole frame opened, charged to one stage and closed, with no work in it |
| `frame.accounting_seven_stages` | 100.0 ns | 100.0 ns | 56.3% | The same frame with every stage in the sequence charged separately |
| `frame.stages` | 7 | — | — | Stages in the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | 6 | — | — | Of those, the ones a system in this repository can actually run in |
| `input.sample_idle` | 500.0 ns | 600.0 ns | 9.1% | One frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | 1.30 µs | 1.60 µs | 31.1% | One frame carrying a single key transition, resolved against the keymap |
| `input.sample_stick` | 1.40 µs | 1.50 µs | 3.5% | One frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | 100.0 ns | 200.0 ns | 29.6% | Checking a two-action snapshot that arrived from outside the trust boundary |
| `input.bindings` | 41 | — | — | Bindings the sampled keymap holds, across two contexts |
| `save.crc32_64kib` | 251.24 µs | 260.61 µs | 2.0% | CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference) |
| `save.encode` | 5 MiB/s | 7.90 ms | 4.2% | Serialize the whole world to bytes, including per-section checksums |
| `save.decode` | 27 MiB/s | 1.90 ms | 10.5% | Parse and verify every checksum on the way back in |
| `save.world_load` | 1.81 ms | 2.22 ms | 8.5% | Rebuild the world from a container, remapping every block through its identifier |
| `save.write_atomic_disk` | 2 MiB/s | 21.32 ms | 7.9% | Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006) |
| `save.region_write_all` | 40.92 ms | 45.03 ms | 4.7% | Write every region of the world: four region files and the header |
| `save.region_write_one_dirty` | 12.39 ms | 22.45 ms | 29.0% | Write after a single block edit: one region file and the header |
| `save.regions_written_per_edit` | 1 | — | — | Region files rewritten after one block changed, out of four |
| `save.region_bytes_all` | 41.7 KiB | — | — | Bytes across every region file when the whole world is written |
| `save.region_bytes_one_dirty` | 4.7 KiB | — | — | Bytes written after one block changed |
| `journal.append_unsynced` | 2.63 µs | 2.96 µs | 4.9% | Frame and checksum one edit record, without making it durable |
| `journal.append_durable` | 514.00 µs | 1.03 ms | 32.7% | One edit record, fsynced before returning: durability per edit |
| `journal.append_batched_sync` | 717.52 µs | 921.17 µs | 14.7% | One edit record when 64 share a single fsync |
| `journal.replay_10k_records` | 2.54 ms | 3.05 ms | 12.2% | Read back and verify a journal of 10,000 edit records |
| `journal.record_bytes` | 55 B | — | — | Encoded size of one block edit, framing included |
| `save.read_disk` | 25 MiB/s | 1.70 ms | 4.4% | Read and verify a save from disk |
| `save.size_9_chunks` | 40.8 KiB | — | — | Bytes on disk for a 3×3 chunk world with the full vertical range |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — | In-memory voxel storage for the same world |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — | Non-air blocks that storage represents |
| `mesh.region_16` | 1.64 ms | 1.98 ms | 8.7% | Cull and merge a 16 cubed region across the ground/air boundary |
| `mesh.region_32` | 12.66 ms | 13.64 ms | 3.1% | The same at 32 cubed, the engine's default section size |
| `mesh.cull_only_16` | 1.58 ms | 1.74 ms | 4.2% | Count visible faces without merging them: the culling half alone |
| `mesh.region_16_from_snapshot` | 154.26 µs | 167.98 µs | 4.8% | The same 16 cubed region, meshed from a pre-read dense array |
| `mesh.region_16_with_snapshot` | 597.11 µs | 607.50 µs | 2.3% | The same region, snapshot built and then meshed: what the snapshot costs in total |
| `mesh.cube_faces_16` | 24 576 | — | — | Faces a naive mesher would emit: every cell, all six sides |
| `mesh.visible_faces_16` | 2 471 | — | — | Faces left after culling |
| `mesh.quads_16` | 807 | — | — | Rectangles left after greedy merging |
| `mesh.vertices_16` | 3 228 | — | — | Vertices a renderer would upload, at four per rectangle |
| `camera.sample` | 100.0 ns | 200.0 ns | 35.8% | Resolve a camera 2^40 blocks out: view, reverse-Z projection, product, frustum |
| `camera.cull_columns_r12` | 4.40 µs | 4.50 µs | 2.0% | Test the 625 columns a radius-12 observer streams against the frustum |
| `camera.visible_columns_r12` | 261 | — | — | Of those 625 columns, the ones a 70-degree view looking down and ahead keeps |
| `rhi.device_open` | 185.40 ms | 191.59 ms | 3.3% | Find the adapter and open a device on it, with no surface: the RHI's startup |
| `rhi.fence_roundtrip` | 102.62 µs | 126.00 µs | 9.2% | Submit a list holding only a marker and wait for its fence: synchronization alone |
| `rhi.texture_create_destroy` | 2.67 µs | 5.68 µs | 42.3% | Create a 16x16 RGBA8 texture and destroy it with no work in flight |
| `rhi.upload_texture_16` | 7 MiB/s | 142.10 µs | 4.1% | Upload one 16x16 RGBA8 texture and wait for it: one write, one fence |
| `rhi.upload_first_generation` | 68 MiB/s | 257.92 µs | 6.4% | Upload sixteen 16x16 textures in one submission and one fence, as the slice does |
| `rhi.mesh_16_vertex_bytes` | 50.4 KiB | — | — | Vertex bytes for the meshed 16 cubed region, at 16 bytes a vertex (DEBT-0046) |
| `rhi.upload_mesh_16` | 339 MiB/s | 160.67 µs | 5.4% | Upload the meshed 16 cubed region's vertex buffer and wait for it |
| `rhi.draw_16` | 139.78 µs | 986.41 µs | 122.0% | Draw one full-target triangle into a 16x16 target and wait for it |
| `frame.draw_chunk_16` | 241.95 µs | 268.45 µs | 6.6% | One frame at 256x256: clear, camera, the meshed 16 cubed region with reverse-Z depth, one submission, one fence |
| `frame.chunk_16_vertices` | 7 377 | — | — | Vertices the frame draws: six per merged rectangle of the region |
| `frame.pixels_judged` | 51 376 | — | — | Of the frame's 65,536 pixels, those checked against the CPU ray cast (all matched) |
| `window.first_frame` | 433.67 ms | 433.67 ms | — | Open the event loop, the window, a device and its surface, upload the chunk, and show the first frame: startup, once |
| `window.present_chunk_16` | 9.97 ms | 10.49 ms | 15.1% | Interval between frames reaching a 256x256 window: the meshed 16 cubed region drawn and presented (FIFO: a real display paces it to its refresh) |
| `window.pixels_judged` | 51 376 | — | — | Of the first shown frame's 65,536 pixels, read back from the surface, those checked against the CPU ray cast (all matched) |
| `ffi.scalar_inlined` | 1.6 ns | 1.7 ns | 2.4% | Mix one u64, inlined Rust: the work with no call at all |
| `ffi.scalar_opaque_rust` | 1.6 ns | 1.7 ns | 1.4% | The same mix behind a Rust call the optimizer will not inline |
| `ffi.bulk_inlined` | 3.86 µs | 3.96 µs | 0.9% | Hash 4 KiB, inlined Rust |
| `ffi.bulk_opaque_rust` | 3.86 µs | 3.96 µs | 1.6% | Hash 4 KiB behind a Rust call the optimizer will not inline |

#### Published budgets

| measurement | owner | target | warning | critical | emergency | median | p95 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| `physics.thousand_bodies_step` | physics (`physics::budget`, DEBT-0013) | 250.00 µs | 500.00 µs | 1.00 ms | 2.00 ms | 165.88 µs · target | 243.43 µs · target |

#### Not measured

| stage the plan asks for | why there is no number |
| --- | --- |
| input | keys reach the engine (ADR-0031), but key-to-frame latency needs a device timestamp winit does not give |
| mod boundary | mod runtime not implemented; Phase 7 |
| incremental build | measured by the build system, not by this process |
| debugging / tooling effort | qualitative; the plan scores it separately from timing |
| FFI overhead | built without the `cpp` feature; no boundary is linked in |

## Verdict

Failed: `tests`, `input_devices`.
