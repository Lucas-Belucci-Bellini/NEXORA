# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `bb159e4f229ae2455bea4d9b7d7e0ec4aba0ba73` |
| branch | `main` |
| generated | 2026-09-30T13:19:25+00:00 |
| OS | Windows 10 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 1.8 | exit 0 |
| `tests` | **PASS** | 138.8 | 1244 passed, 0 failed |
| `headless_slice` | **PASS** | 0.7 | queries 76 answered, 2 refused; rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 76; result OK |
| `headless_slice_content` | **PASS** | 0.7 | queries 92 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 92; result OK |
| `forge_first_generation` | **PASS** | 0.1 | build nexora-first-generation: 16 materials: 16 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (32 resources) |
| `headless_slice_textures` | **PASS** | 0.4 | queries 44 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); content textures 16 (resolved, verified and decoded); rhi null backend, conformance 11/11 cases, no GPU initialized; rhi uploads 16 textures as RGBA8, 16384 bytes, one fence; memory 4 pools, worst nominal, 0 suspected leaks; probes verified 44; result OK |
| `rhi_native` | **PASS** | 0.4 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); conformance 11/11 cases on wgpu; upload 1024 bytes to a 16x16 texture, read back identical; draw 16 of 16 texels shaded by the GPU; bound 256 of 256 texels sampled, 256 kept by depth, 256 tinted by a uniform; cull 16 of 16 texels shaded counter-clockwise, 0 clockwise, culling back faces; result OK |
| `window` | **PASS** | 1.4 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 256x256 on win32, 0 redraws waited for it to take frames; surface 256x256 Bgra8UnormSrgb, Fifo; conformance 11/11 cases on wgpu, presenting; frame 65536 of 65536 window texels show the 16x16 target, read back from the surface; presented 61 frames, 60 of them by the probe; result OK |
| `client_mode` | **PASS** | 5.5 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 384x256 on win32; first frame 79391 of 98304 pixels judged by the ray cast, 79390 matching, 1 snapped edges, 0 back faces, read back from the surface; frames 120 shown, ended by frame limit; frame loop 26 ticks, 0 discarded; 119 target, 1 warning, 0 critical, 0 emergency (budget: doubling from the 50 ms step, not measured); frame wall median 9.84 ms, p95 14.59 ms, max 71.42 ms; 0.20 ms unattributed in all; result OK |
| `input_devices` | **PASS** | 7.2 | input keyboard/0 button 26 (W) reached nexora:action/probe: pressed after 633 frames, held 13; signals 6 delivered, 0 keys without a HID usage, 0 repeats dropped; result OK |
| `benchmark_cpu` | **PASS** | 14.5 | see the report's benchmark section |

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
| `startup.world_create` | 2.34 µs | 5.15 µs | 36.8% | Create a world, register the block set, freeze the registry and bring it online |
| `spatial.section_of_division` | 2.4 ns | 2.9 ns | 12.0% | Block → section using div_euclid by a runtime extent (what the engine does) |
| `spatial.section_of_shift_runtime` | 6.0 ns | 6.3 ns | 2.5% | Block → section by shift, width read at runtime: what DEBT-0005 would actually build |
| `spatial.section_of_shift_const` | 6.0 ns | 6.5 ns | 4.8% | Block → section by shift with a compile-time width: the optimization's ceiling |
| `spatial.index_of` | 6.0 ns | 6.2 ns | 1.6% | Flatten a local position into a storage index, bounds-checked |
| `voxel.get_paletted` | 8.7 ns | 10.6 ns | 18.2% | Read one cell from a palette-indexed section |
| `voxel.get_uniform` | 5.2 ns | 5.5 ns | 2.1% | Read one cell from a uniform section (solid rock, the common case) |
| `voxel.set_existing_state` | 17.3 ns | 23.5 ns | 24.1% | Write a cell whose state is already in the palette |
| `voxel.first_write_to_uniform` | 123.5 ns | 195.0 ns | 23.8% | Break a uniform section into a palette: the allocation the air optimization defers |
| `voxel.compact_section` | 178.54 µs | 206.04 µs | 13.3% | Rebuild a section's palette from what is actually referenced |
| `chunk.change_feed_append` | 42.4 ns | 108.3 ns | 45.2% | One journalled voxel write with the change feed below its cap |
| `chunk.change_feed_at_cap` | 44.4 ns | 50.6 ns | 6.1% | The same write with the feed full, so each one discards the oldest entry |
| `chunk.change_feed_cap` | 4 096 | — | — | Entries a chunk's change feed holds before it starts discarding |
| `worldgen.chunk_16` | 168.50 µs | 227.50 µs | 12.9% | Generate one chunk column, 16³ sections (the plan's size) |
| `worldgen.chunk_32` | 1.29 ms | 1.74 ms | 18.7% | Generate one chunk column, 32³ sections (engine default) |
| `worldgen.surface_height` | 41.1 ns | 45.0 ns | 9.5% | One column's terrain height from the position-seeded stream |
| `entity.spawn` | 331.8 ns | 1.04 µs | 67.1% | Create one entity: slot allocation, component writes, persistent id |
| `entity.spawn_despawn_cycle` | 342.8 ns | 369.8 ns | 7.1% | Spawn then destroy: the slot reuse path a busy world runs constantly |
| `entity.resolve_handle` | 3.9 ns | 4.5 ns | 20.7% | Validate one handle: world, range, generation and liveness |
| `entity.step_1000` | 5.77 µs | 6.57 µs | 8.7% | Advance 1,000 entities by one tick: the batch walk over dense columns |
| `entity.query_type_1000` | 27.47 µs | 34.76 µs | 14.5% | Scan of 1,000 entities filtering by type: no position, so no index |
| `entity.query_tag_1000` | 28.31 µs | 33.28 µs | 14.2% | Scan of 1,000 entities filtering by tag: no position, so no index |
| `entity.query_radius_1000` | 17.83 µs | 21.51 µs | 14.1% | Radius query in the plan's dense 1,000-entity box: the index cannot narrow it |
| `entity.query_radius_100k` | 776.5 ns | 1.26 µs | 28.1% | Radius query over 100,000 entities spread across a world, through the index |
| `entity.query_chunk_100k` | 573.0 ns | 823.5 ns | 15.1% | Which of 100,000 entities are in one chunk column, through the index |
| `entity.query_radius_candidates_100k` | 41 | — | — | Entities a 16-block radius query examines, of 100,000 in the store |
| `entity.index_cells_100k` | 24 642 | — | — | Grid cells holding at least one of the 100,000 entities |
| `entity.step_1000_crossing_cells` | 214.10 µs | 245.19 µs | 10.0% | Advance 1,000 entities that all change grid cell every tick: the index at its worst |
| `entity.save_1000` | 384 MiB/s | 385.12 µs | 9.1% | Serialize 1,000 entities as logical state |
| `entity.load_1000` | 171 MiB/s | 1.12 ms | 15.7% | Restore 1,000 entities, re-resolving every persistent id |
| `entity.save_size_1000` | 141.6 KiB | — | — | Bytes of save section for 1,000 entities |
| `physics.character_step` | 733.8 ns | 954.5 ns | 11.7% | One character substep on generated terrain: gravity, sweep, ground, friction |
| `physics.thousand_bodies_step` | 162.53 µs | 343.55 µs | 38.3% | One substep of 1,000 awake dynamic bodies against generated terrain |
| `physics.thousand_bodies_step_flat` | 156.00 µs | 354.50 µs | 41.3% | The same substep against a flat fixture: the solver without the world lookup |
| `physics.world_reads_per_step` | 1 000 | — | — | Cells the world is asked about in one substep of 1,000 settled awake bodies |
| `physics.voxel_lookup` | 12.4 ns | 12.7 ns | 1.3% | One cell question answered by the world view, section already resident |
| `physics.axis_sweep` | 28.6 ns | 75.6 ns | 49.4% | One axis of a swept box against the voxel grid (paired with the C++ reference) |
| `physics.box_sweep` | 230.0 ns | 280.7 ns | 21.9% | Resolve one box move on three axes against generated terrain |
| `physics.depenetration_check` | 62.8 ns | 98.0 ns | 28.3% | The per-body test for having started inside terrain, when it has not |
| `physics.raycast_40m` | 1.16 µs | 1.25 µs | 12.6% | Walk a 40 m ray down through generated terrain until it hits |
| `physics.timestep_accumulate` | 3.3 ns | 3.5 ns | 2.5% | Fold one world tick into the fixed-step accumulator |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — | Bodies asleep after ten seconds: what sleeping actually saves |
| `physics.thousand_sleeping_step` | 810.0 ns | 820.0 ns | 0.7% | One substep with the same 1,000 bodies asleep |
| `streaming.idle_tick_r3` | 4.35 µs | 5.36 µs | 12.9% | A tick with nothing to do, 7x7 columns of interest: pure decision cost |
| `streaming.idle_tick_r12` | 94.76 µs | 107.78 µs | 6.7% | The same idle tick over 25x25 columns: how decision cost scales with radius |
| `streaming.walk_one_chunk` | 7.70 µs | 8.86 µs | 14.5% | A tick after the observer moves one column: decide, load the new ring, release the old |
| `streaming.fast_travel_r3` | 36.87 µs | 56.61 µs | 23.3% | Teleport and settle: release 49 columns and take 49 more, decision side only |
| `streaming.chunk_generate_cycle` | 1.28 ms | 1.68 ms | 13.7% | Activate a fresh column and drop it again: the cost of an unedited chunk |
| `streaming.chunk_retained_cycle` | 183.5 ns | 196.0 ns | 2.7% | Persist, drop and restore an edited column: retention instead of regeneration |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — | Memory an edited column occupies while it is evicted |
| `streaming.chunk_flushed_cycle` | 14.34 ms | 30.25 ms | 48.8% | Persist, flush to a region file, drop, and read the column back from disk |
| `streaming.retained_bytes_after_flush` | 0 B | — | — | Memory the same evicted column occupies once it is in its region file |
| `streaming.region_writes_per_eight_columns` | 4 | — | — | Region files a flush of eight columns touches, two columns to a region |
| `streaming.columns_of_interest_r12` | 625 | — | — | Columns a radius-12 observer makes the manager consider every tick |
| `jobs.submit_wait_roundtrip` | 4.25 µs | 17.20 µs | 107.0% | Submit one trivial job and block until it completes: the full scheduling round trip |
| `jobs.submit_only` | 1.18 µs | 2.61 µs | 58.3% | Enqueue a job without waiting: the cost a producer pays |
| `jobs.batch_1000_barrier` | 1.09 ms | 3.04 ms | 55.3% | Submit 1,000 jobs one at a time and barrier: a wake-up per job |
| `jobs.batch_1000_barrier_submit_all` | 818.40 µs | 3.32 ms | 86.4% | The same 1,000 jobs handed over as one batch: one wake-up for the wave |
| `jobs.submit_all_1000` | 202.10 µs | 317.70 µs | 41.5% | Hand over 1,000 jobs as one batch, without waiting: the producer's cost |
| `jobs.wakeups_per_1000_submitted` | 1 000 | — | — | Condvar wake-ups a wave of 1,000 jobs issues, submitted one at a time |
| `jobs.wakeups_per_1000_submit_all` | 8 | — | — | Condvar wake-ups the same wave issues through submit_all |
| `frame.schedule_advance` | 100.0 ns | 100.0 ns | 134.4% | The fixed-timestep accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | 100.0 ns | 100.0 ns | 43.5% | A whole frame opened, charged to one stage and closed, with no work in it |
| `frame.accounting_seven_stages` | 100.0 ns | 200.0 ns | 36.1% | The same frame with every stage in the sequence charged separately |
| `frame.stages` | 7 | — | — | Stages in the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | 6 | — | — | Of those, the ones a system in this repository can actually run in |
| `input.sample_idle` | 1.10 µs | 1.20 µs | 7.6% | One frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | 2.10 µs | 2.80 µs | 48.2% | One frame carrying a single key transition, resolved against the keymap |
| `input.sample_stick` | 2.90 µs | 3.60 µs | 66.7% | One frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | 200.0 ns | 300.0 ns | 39.3% | Checking a two-action snapshot that arrived from outside the trust boundary |
| `input.bindings` | 41 | — | — | Bindings the sampled keymap holds, across two contexts |
| `save.crc32_64kib` | 343.42 µs | 504.20 µs | 21.4% | CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference) |
| `save.encode` | 1 MiB/s | 84.70 ms | 62.5% | Serialize the whole world to bytes, including per-section checksums |
| `save.decode` | 15 MiB/s | 4.58 ms | 28.2% | Parse and verify every checksum on the way back in |
| `save.world_load` | 3.42 ms | 5.46 ms | 23.8% | Rebuild the world from a container, remapping every block through its identifier |
| `save.write_atomic_disk` | 1 MiB/s | 40.82 ms | 12.6% | Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006) |
| `save.region_write_all` | 60.94 ms | 84.06 ms | 15.1% | Write every region of the world: four region files and the header |
| `save.region_write_one_dirty` | 17.49 ms | 23.92 ms | 23.9% | Write after a single block edit: one region file and the header |
| `save.regions_written_per_edit` | 1 | — | — | Region files rewritten after one block changed, out of four |
| `save.region_bytes_all` | 41.7 KiB | — | — | Bytes across every region file when the whole world is written |
| `save.region_bytes_one_dirty` | 4.7 KiB | — | — | Bytes written after one block changed |
| `journal.append_unsynced` | 4.28 µs | 5.30 µs | 14.8% | Frame and checksum one edit record, without making it durable |
| `journal.append_durable` | 740.28 µs | 1.31 ms | 32.2% | One edit record, fsynced before returning: durability per edit |
| `journal.append_batched_sync` | 1.17 ms | 1.99 ms | 35.5% | One edit record when 64 share a single fsync |
| `journal.replay_10k_records` | 3.55 ms | 3.77 ms | 9.6% | Read back and verify a journal of 10,000 edit records |
| `journal.record_bytes` | 55 B | — | — | Encoded size of one block edit, framing included |
| `save.read_disk` | 20 MiB/s | 2.26 ms | 10.5% | Read and verify a save from disk |
| `save.size_9_chunks` | 40.8 KiB | — | — | Bytes on disk for a 3×3 chunk world with the full vertical range |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — | In-memory voxel storage for the same world |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — | Non-air blocks that storage represents |
| `mesh.region_16` | 2.05 ms | 2.12 ms | 2.1% | Cull and merge a 16 cubed region across the ground/air boundary |
| `mesh.region_32` | 14.99 ms | 18.16 ms | 10.9% | The same at 32 cubed, the engine's default section size |
| `mesh.cull_only_16` | 2.62 ms | 6.06 ms | 47.2% | Count visible faces without merging them: the culling half alone |
| `mesh.region_16_from_snapshot` | 261.51 µs | 271.31 µs | 3.1% | The same 16 cubed region, meshed from a pre-read dense array |
| `mesh.region_16_with_snapshot` | 926.40 µs | 983.30 µs | 6.9% | The same region, snapshot built and then meshed: what the snapshot costs in total |
| `mesh.cube_faces_16` | 24 576 | — | — | Faces a naive mesher would emit: every cell, all six sides |
| `mesh.visible_faces_16` | 2 471 | — | — | Faces left after culling |
| `mesh.quads_16` | 807 | — | — | Rectangles left after greedy merging |
| `mesh.vertices_16` | 3 228 | — | — | Vertices a renderer would upload, at four per rectangle |
| `camera.sample` | 200.0 ns | 300.0 ns | 77.7% | Resolve a camera 2^40 blocks out: view, reverse-Z projection, product, frustum |
| `camera.cull_columns_r12` | 6.60 µs | 7.00 µs | 3.0% | Test the 625 columns a radius-12 observer streams against the frustum |
| `camera.visible_columns_r12` | 261 | — | — | Of those 625 columns, the ones a 70-degree view looking down and ahead keeps |
| `rhi.device_open` | 342.47 ms | 370.76 ms | 5.8% | Find the adapter and open a device on it, with no surface: the RHI's startup |
| `rhi.fence_roundtrip` | 203.83 µs | 285.14 µs | 21.5% | Submit a list holding only a marker and wait for its fence: synchronization alone |
| `rhi.texture_create_destroy` | 4.59 µs | 5.72 µs | 11.3% | Create a 16x16 RGBA8 texture and destroy it with no work in flight |
| `rhi.upload_texture_16` | 4 MiB/s | 328.94 µs | 20.0% | Upload one 16x16 RGBA8 texture and wait for it: one write, one fence |
| `rhi.upload_first_generation` | 34 MiB/s | 515.76 µs | 17.9% | Upload sixteen 16x16 textures in one submission and one fence, as the slice does |
| `rhi.mesh_16_vertex_bytes` | 50.4 KiB | — | — | Vertex bytes for the meshed 16 cubed region, at 16 bytes a vertex (DEBT-0046) |
| `rhi.upload_mesh_16` | 240 MiB/s | 557.77 µs | 48.5% | Upload the meshed 16 cubed region's vertex buffer and wait for it |
| `rhi.draw_16` | 276.33 µs | 289.17 µs | 9.6% | Draw one full-target triangle into a 16x16 target and wait for it |
| `frame.draw_chunk_16` | 253.87 µs | 357.23 µs | 19.3% | One frame at 256x256: clear, camera, the meshed 16 cubed region with reverse-Z depth, one submission, one fence |
| `frame.chunk_16_vertices` | 7 377 | — | — | Vertices the frame draws: six per merged rectangle of the region |
| `frame.pixels_judged` | 51 376 | — | — | Of the frame's 65,536 pixels, those checked against the CPU ray cast (all matched) |
| `window.first_frame` | 503.15 ms | 503.15 ms | — | Open the event loop, the window, a device and its surface, upload the chunk, and show the first frame: startup, once |
| `window.present_chunk_16` | 10.02 ms | 10.92 ms | 15.0% | Interval between frames reaching a 256x256 window: the meshed 16 cubed region drawn and presented (FIFO: a real display paces it to its refresh) |
| `window.pixels_judged` | 51 376 | — | — | Of the first shown frame's 65,536 pixels, read back from the surface, those checked against the CPU ray cast (all matched) |
| `ffi.scalar_inlined` | 1.7 ns | 1.8 ns | 2.7% | Mix one u64, inlined Rust: the work with no call at all |
| `ffi.scalar_opaque_rust` | 1.7 ns | 1.7 ns | 3.0% | The same mix behind a Rust call the optimizer will not inline |
| `ffi.bulk_inlined` | 3.92 µs | 4.13 µs | 2.1% | Hash 4 KiB, inlined Rust |
| `ffi.bulk_opaque_rust` | 3.92 µs | 4.54 µs | 5.2% | Hash 4 KiB behind a Rust call the optimizer will not inline |

#### Published budgets

| measurement | owner | target | warning | critical | emergency | median | p95 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| `physics.thousand_bodies_step` | physics (`physics::budget`, DEBT-0013) | 250.00 µs | 500.00 µs | 1.00 ms | 2.00 ms | 162.53 µs · target | 343.55 µs · warning |

#### Not measured

| stage the plan asks for | why there is no number |
| --- | --- |
| input | keys reach the engine (ADR-0031), but key-to-frame latency needs a device timestamp winit does not give |
| mod boundary | mod runtime not implemented; Phase 7 |
| incremental build | measured by the build system, not by this process |
| debugging / tooling effort | qualitative; the plan scores it separately from timing |
| FFI overhead | built without the `cpp` feature; no boundary is linked in |

## Verdict

Every executed check passed.
