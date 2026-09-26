# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `786f77f3dcef2a9c001433f0b257f6055dc1fc54` |
| branch | `main` |
| generated | 2026-09-26T12:12:57+00:00 |
| OS | Windows 10 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 202.0 | exit 0 |
| `tests` | **PASS** | 169.3 | 1178 passed, 0 failed |
| `headless_slice` | **PASS** | 0.8 | queries 76 answered, 2 refused; rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 76; result OK |
| `headless_slice_content` | **PASS** | 0.8 | queries 92 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 92; result OK |
| `forge_first_generation` | **PASS** | 0.2 | build nexora-first-generation: 16 materials: 16 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (32 resources) |
| `headless_slice_textures` | **PASS** | 0.3 | queries 44 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); content textures 16 (resolved, verified and decoded); rhi null backend, conformance 11/11 cases, no GPU initialized; rhi uploads 16 textures as RGBA8, 16384 bytes, one fence; memory 4 pools, worst nominal, 0 suspected leaks; probes verified 44; result OK |
| `rhi_native` | **PASS** | 0.8 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); conformance 11/11 cases on wgpu; upload 1024 bytes to a 16x16 texture, read back identical; draw 16 of 16 texels shaded by the GPU; bound 256 of 256 texels sampled, 256 kept by depth, 256 tinted by a uniform; result OK |
| `window` | **PASS** | 1.5 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 256x256 on win32, 0 redraws waited for it to take frames; surface 256x256 Bgra8UnormSrgb, Fifo; conformance 11/11 cases on wgpu, presenting; frame 65536 of 65536 window texels show the 16x16 target, read back from the surface; presented 61 frames, 60 of them by the probe; result OK |
| `benchmark_cpu` | **PASS** | 7.6 | see the report's benchmark section |

## What this machine could not validate, and why

Not "not tested": the engine has no code for these yet, so no machine can.

| item | status | reason |
| --- | --- | --- |
| `rendering` | NOT_IMPLEMENTED | no renderer: the native backend draws one triangle (rhi_native) and presents a 16x16 target (window); nothing draws the world; meshes are data (ADR-0012) |
| `input_devices` | NOT_IMPLEMENTED | no real device has produced a signal (DEBT-0043) |
| `client_mode` | NOT_IMPLEMENTED | the runtime starts headless only; client mode is Phase 1's exit |
| `benchmark_gpu_stages` | NOT_IMPLEMENTED | partly built: the RHI stage (device, fence, upload, draw) runs inside benchmark_cpu on this machine's adapter; window frame time and camera have no implementation (DEBT-0008) |

## Benchmark (CPU stages, and the RHI stage on this machine's adapter)

### Environment

| | |
| --- | --- |
| logical CPUs | 12 |
| build profile | `release` |
| architecture | `x86_64` |
| peak resident memory | unavailable |
| benchmark binary size | 7.1 MiB |
| GPU adapter (RHI stage) | AMD Radeon RX 6650 XT (vulkan, discretegpu) |

### Measurements

| measurement | median | p95 | rel. σ | note |
| --- | ---: | ---: | ---: | --- |
| `startup.world_create` | 1.34 µs | 2.31 µs | 25.7% | Create a world, register the block set, freeze the registry and bring it online |
| `spatial.section_of_division` | 1.8 ns | 2.9 ns | 19.9% | Block → section using div_euclid by a runtime extent (what the engine does) |
| `spatial.section_of_shift_runtime` | 6.0 ns | 6.0 ns | 1.4% | Block → section by shift, width read at runtime: what DEBT-0005 would actually build |
| `spatial.section_of_shift_const` | 6.0 ns | 6.2 ns | 2.3% | Block → section by shift with a compile-time width: the optimization's ceiling |
| `spatial.index_of` | 4.4 ns | 6.2 ns | 18.9% | Flatten a local position into a storage index, bounds-checked |
| `voxel.get_paletted` | 5.8 ns | 6.2 ns | 7.3% | Read one cell from a palette-indexed section |
| `voxel.get_uniform` | 4.0 ns | 4.3 ns | 4.0% | Read one cell from a uniform section (solid rock, the common case) |
| `voxel.set_existing_state` | 11.3 ns | 13.1 ns | 15.4% | Write a cell whose state is already in the palette |
| `voxel.first_write_to_uniform` | 126.5 ns | 239.5 ns | 24.3% | Break a uniform section into a palette: the allocation the air optimization defers |
| `voxel.compact_section` | 131.01 µs | 147.59 µs | 5.2% | Rebuild a section's palette from what is actually referenced |
| `chunk.change_feed_append` | 30.4 ns | 75.5 ns | 57.6% | One journalled voxel write with the change feed below its cap |
| `chunk.change_feed_at_cap` | 31.9 ns | 54.1 ns | 30.0% | The same write with the feed full, so each one discards the oldest entry |
| `chunk.change_feed_cap` | 4 096 | — | — | Entries a chunk's change feed holds before it starts discarding |
| `worldgen.chunk_16` | 167.90 µs | 250.80 µs | 18.1% | Generate one chunk column, 16³ sections (the plan's size) |
| `worldgen.chunk_32` | 1.11 ms | 1.59 ms | 18.1% | Generate one chunk column, 32³ sections (engine default) |
| `worldgen.surface_height` | 42.5 ns | 44.3 ns | 5.2% | One column's terrain height from the position-seeded stream |
| `entity.spawn` | 190.2 ns | 328.8 ns | 36.9% | Create one entity: slot allocation, component writes, persistent id |
| `entity.spawn_despawn_cycle` | 238.2 ns | 251.8 ns | 2.2% | Spawn then destroy: the slot reuse path a busy world runs constantly |
| `entity.resolve_handle` | 3.6 ns | 4.3 ns | 7.4% | Validate one handle: world, range, generation and liveness |
| `entity.step_1000` | 5.55 µs | 8.32 µs | 25.0% | Advance 1,000 entities by one tick: the batch walk over dense columns |
| `entity.query_type_1000` | 24.23 µs | 33.20 µs | 18.7% | Scan of 1,000 entities filtering by type: no position, so no index |
| `entity.query_tag_1000` | 19.97 µs | 26.19 µs | 12.1% | Scan of 1,000 entities filtering by tag: no position, so no index |
| `entity.query_radius_1000` | 10.79 µs | 11.79 µs | 4.2% | Radius query in the plan's dense 1,000-entity box: the index cannot narrow it |
| `entity.query_radius_100k` | 1.15 µs | 2.33 µs | 44.3% | Radius query over 100,000 entities spread across a world, through the index |
| `entity.query_chunk_100k` | 862.0 ns | 1.07 µs | 11.9% | Which of 100,000 entities are in one chunk column, through the index |
| `entity.query_radius_candidates_100k` | 41 | — | — | Entities a 16-block radius query examines, of 100,000 in the store |
| `entity.index_cells_100k` | 24 642 | — | — | Grid cells holding at least one of the 100,000 entities |
| `entity.step_1000_crossing_cells` | 171.67 µs | 213.85 µs | 10.9% | Advance 1,000 entities that all change grid cell every tick: the index at its worst |
| `entity.save_1000` | 488 MiB/s | 322.19 µs | 9.9% | Serialize 1,000 entities as logical state |
| `entity.load_1000` | 233 MiB/s | 608.35 µs | 4.8% | Restore 1,000 entities, re-resolving every persistent id |
| `entity.save_size_1000` | 141.6 KiB | — | — | Bytes of save section for 1,000 entities |
| `physics.character_step` | 366.8 ns | 417.0 ns | 5.3% | One character substep on generated terrain: gravity, sweep, ground, friction |
| `physics.thousand_bodies_step` | 84.28 µs | 117.25 µs | 14.1% | One substep of 1,000 awake dynamic bodies against generated terrain |
| `physics.thousand_bodies_step_flat` | 79.38 µs | 91.95 µs | 5.8% | The same substep against a flat fixture: the solver without the world lookup |
| `physics.world_reads_per_step` | 1 000 | — | — | Cells the world is asked about in one substep of 1,000 settled awake bodies |
| `physics.voxel_lookup` | 12.1 ns | 12.5 ns | 1.2% | One cell question answered by the world view, section already resident |
| `physics.axis_sweep` | 29.4 ns | 35.8 ns | 8.2% | One axis of a swept box against the voxel grid (paired with the C++ reference) |
| `physics.box_sweep` | 142.9 ns | 152.1 ns | 3.2% | Resolve one box move on three axes against generated terrain |
| `physics.depenetration_check` | 47.3 ns | 50.2 ns | 2.2% | The per-body test for having started inside terrain, when it has not |
| `physics.raycast_40m` | 825.2 ns | 898.1 ns | 4.1% | Walk a 40 m ray down through generated terrain until it hits |
| `physics.timestep_accumulate` | 3.3 ns | 3.3 ns | 0.1% | Fold one world tick into the fixed-step accumulator |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — | Bodies asleep after ten seconds: what sleeping actually saves |
| `physics.thousand_sleeping_step` | 785.0 ns | 795.0 ns | 0.6% | One substep with the same 1,000 bodies asleep |
| `streaming.idle_tick_r3` | 3.37 µs | 3.83 µs | 8.4% | A tick with nothing to do, 7x7 columns of interest: pure decision cost |
| `streaming.idle_tick_r12` | 88.45 µs | 93.94 µs | 3.9% | The same idle tick over 25x25 columns: how decision cost scales with radius |
| `streaming.walk_one_chunk` | 6.11 µs | 7.41 µs | 8.1% | A tick after the observer moves one column: decide, load the new ring, release the old |
| `streaming.fast_travel_r3` | 33.04 µs | 37.77 µs | 8.6% | Teleport and settle: release 49 columns and take 49 more, decision side only |
| `streaming.chunk_generate_cycle` | 1.08 ms | 1.13 ms | 3.2% | Activate a fresh column and drop it again: the cost of an unedited chunk |
| `streaming.chunk_retained_cycle` | 179.0 ns | 182.0 ns | 0.8% | Persist, drop and restore an edited column: retention instead of regeneration |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — | Memory an edited column occupies while it is evicted |
| `streaming.chunk_flushed_cycle` | 5.70 ms | 6.21 ms | 4.3% | Persist, flush to a region file, drop, and read the column back from disk |
| `streaming.retained_bytes_after_flush` | 0 B | — | — | Memory the same evicted column occupies once it is in its region file |
| `streaming.region_writes_per_eight_columns` | 4 | — | — | Region files a flush of eight columns touches, two columns to a region |
| `streaming.columns_of_interest_r12` | 625 | — | — | Columns a radius-12 observer makes the manager consider every tick |
| `jobs.submit_wait_roundtrip` | 8.07 µs | 10.90 µs | 15.8% | Submit one trivial job and block until it completes: the full scheduling round trip |
| `jobs.submit_only` | 4.71 µs | 7.56 µs | 38.3% | Enqueue a job without waiting: the cost a producer pays |
| `jobs.batch_1000_barrier` | 3.74 ms | 5.66 ms | 54.2% | Submit 1,000 jobs one at a time and barrier: a wake-up per job |
| `jobs.batch_1000_barrier_submit_all` | 832.10 µs | 878.50 µs | 3.6% | The same 1,000 jobs handed over as one batch: one wake-up for the wave |
| `jobs.submit_all_1000` | 110.50 µs | 206.90 µs | 42.5% | Hand over 1,000 jobs as one batch, without waiting: the producer's cost |
| `jobs.wakeups_per_1000_submitted` | 1 000 | — | — | Condvar wake-ups a wave of 1,000 jobs issues, submitted one at a time |
| `jobs.wakeups_per_1000_submit_all` | 8 | — | — | Condvar wake-ups the same wave issues through submit_all |
| `frame.schedule_advance` | 0.0 ns | 100.0 ns | 132.5% | The fixed-timestep accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | 100.0 ns | 100.0 ns | 30.1% | A whole frame opened, charged to one stage and closed, with no work in it |
| `frame.accounting_seven_stages` | 100.0 ns | 100.0 ns | 63.6% | The same frame with every stage in the sequence charged separately |
| `frame.stages` | 7 | — | — | Stages in the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | 4 | — | — | Of those, the ones a system in this repository can actually run in |
| `input.sample_idle` | 500.0 ns | 600.0 ns | 8.7% | One frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | 1.20 µs | 1.30 µs | 24.9% | One frame carrying a single key transition, resolved against the keymap |
| `input.sample_stick` | 1.50 µs | 1.50 µs | 2.7% | One frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | 100.0 ns | 200.0 ns | 32.3% | Checking a two-action snapshot that arrived from outside the trust boundary |
| `input.bindings` | 41 | — | — | Bindings the sampled keymap holds, across two contexts |
| `save.crc32_64kib` | 266.24 µs | 272.62 µs | 1.3% | CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference) |
| `save.encode` | 5 MiB/s | 8.74 ms | 4.9% | Serialize the whole world to bytes, including per-section checksums |
| `save.decode` | 26 MiB/s | 1.83 ms | 9.1% | Parse and verify every checksum on the way back in |
| `save.world_load` | 1.64 ms | 2.17 ms | 15.4% | Rebuild the world from a container, remapping every block through its identifier |
| `save.write_atomic_disk` | 2 MiB/s | 20.73 ms | 6.2% | Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006) |
| `save.region_write_all` | 47.03 ms | 53.09 ms | 8.5% | Write every region of the world: four region files and the header |
| `save.region_write_one_dirty` | 14.28 ms | 17.46 ms | 9.9% | Write after a single block edit: one region file and the header |
| `save.regions_written_per_edit` | 1 | — | — | Region files rewritten after one block changed, out of four |
| `save.region_bytes_all` | 41.7 KiB | — | — | Bytes across every region file when the whole world is written |
| `save.region_bytes_one_dirty` | 4.7 KiB | — | — | Bytes written after one block changed |
| `journal.append_unsynced` | 3.41 µs | 3.82 µs | 12.1% | Frame and checksum one edit record, without making it durable |
| `journal.append_durable` | 622.99 µs | 1.17 ms | 35.8% | One edit record, fsynced before returning: durability per edit |
| `journal.append_batched_sync` | 1.35 ms | 1.54 ms | 17.5% | One edit record when 64 share a single fsync |
| `journal.replay_10k_records` | 3.37 ms | 4.03 ms | 9.6% | Read back and verify a journal of 10,000 edit records |
| `journal.record_bytes` | 55 B | — | — | Encoded size of one block edit, framing included |
| `save.read_disk` | 21 MiB/s | 2.19 ms | 7.8% | Read and verify a save from disk |
| `save.size_9_chunks` | 40.8 KiB | — | — | Bytes on disk for a 3×3 chunk world with the full vertical range |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — | In-memory voxel storage for the same world |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — | Non-air blocks that storage represents |
| `mesh.region_16` | 1.98 ms | 2.14 ms | 8.7% | Cull and merge a 16 cubed region across the ground/air boundary |
| `mesh.region_32` | 14.19 ms | 15.87 ms | 6.3% | The same at 32 cubed, the engine's default section size |
| `mesh.cull_only_16` | 1.72 ms | 2.22 ms | 11.7% | Count visible faces without merging them: the culling half alone |
| `mesh.region_16_from_snapshot` | 179.25 µs | 188.66 µs | 3.1% | The same 16 cubed region, meshed from a pre-read dense array |
| `mesh.region_16_with_snapshot` | 962.90 µs | 1.18 ms | 19.8% | The same region, snapshot built and then meshed: what the snapshot costs in total |
| `mesh.cube_faces_16` | 24 576 | — | — | Faces a naive mesher would emit: every cell, all six sides |
| `mesh.visible_faces_16` | 2 471 | — | — | Faces left after culling |
| `mesh.quads_16` | 807 | — | — | Rectangles left after greedy merging |
| `mesh.vertices_16` | 3 228 | — | — | Vertices a renderer would upload, at four per rectangle |
| `rhi.device_open` | 275.64 ms | 325.36 ms | 13.1% | Find the adapter and open a device on it, with no surface: the RHI's startup |
| `rhi.fence_roundtrip` | 111.79 µs | 180.35 µs | 28.5% | Submit a list holding only a marker and wait for its fence: synchronization alone |
| `rhi.texture_create_destroy` | 2.36 µs | 2.63 µs | 5.4% | Create a 16x16 RGBA8 texture and destroy it with no work in flight |
| `rhi.upload_texture_16` | 6 MiB/s | 212.13 µs | 19.9% | Upload one 16x16 RGBA8 texture and wait for it: one write, one fence |
| `rhi.upload_first_generation` | 58 MiB/s | 328.50 µs | 18.9% | Upload sixteen 16x16 textures in one submission and one fence, as the slice does |
| `rhi.mesh_16_vertex_bytes` | 50.4 KiB | — | — | Vertex bytes for the meshed 16 cubed region, at 16 bytes a vertex (DEBT-0046) |
| `rhi.upload_mesh_16` | 342 MiB/s | 281.07 µs | 36.6% | Upload the meshed 16 cubed region's vertex buffer and wait for it |
| `rhi.draw_16` | 192.04 µs | 212.93 µs | 9.8% | Draw one full-target triangle into a 16x16 target and wait for it |
| `ffi.scalar_inlined` | 1.7 ns | 1.7 ns | 2.0% | Mix one u64, inlined Rust: the work with no call at all |
| `ffi.scalar_opaque_rust` | 1.7 ns | 1.8 ns | 2.2% | The same mix behind a Rust call the optimizer will not inline |
| `ffi.bulk_inlined` | 4.00 µs | 4.12 µs | 1.3% | Hash 4 KiB, inlined Rust |
| `ffi.bulk_opaque_rust` | 4.02 µs | 4.37 µs | 7.4% | Hash 4 KiB behind a Rust call the optimizer will not inline |

#### Published budgets

| measurement | owner | target | warning | critical | emergency | median | p95 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| `physics.thousand_bodies_step` | physics (`physics::budget`, DEBT-0013) | 250.00 µs | 500.00 µs | 1.00 ms | 2.00 ms | 84.28 µs · target | 117.25 µs · target |

#### Not measured

| stage the plan asks for | why there is no number |
| --- | --- |
| window | a window host exists (ADR-0027); the benchmark does not open one (DEBT-0008) |
| input | no device signal reaches the engine yet (DEBT-0043) |
| camera | no view or projection exists: the renderer owns the camera, and there is no renderer |
| mod boundary | mod runtime not implemented; Phase 7 |
| frame time | the window host presents a test target; no renderer draws a frame to time |
| incremental build | measured by the build system, not by this process |
| debugging / tooling effort | qualitative; the plan scores it separately from timing |
| FFI overhead | built without the `cpp` feature; no boundary is linked in |

## Verdict

Every executed check passed.
