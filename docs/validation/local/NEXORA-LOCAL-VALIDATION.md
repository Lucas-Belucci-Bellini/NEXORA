# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `9a2260ca5bde158f9f9ad4d0a9007612c468008f` |
| branch | `claude/nexora-vertical-slice-interaction` |
| generated | 2026-10-05T08:11:50+00:00 |
| OS | Windows 11 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 85.5 | exit 0 |
| `tests` | **PASS** | 123.9 | 1318 passed, 0 failed |
| `headless_slice` | **PASS** | 1.0 | queries 77 answered, 2 refused; hands broke 1, placed 1, refused 1 in its own body, stood 1.00 higher on its own block; rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 77; result OK |
| `headless_slice_content` | **PASS** | 0.8 | queries 98 answered, 2 refused; hands broke 1, placed 1, refused 1 in its own body, stood 1.00 higher on its own block; content blocks 21 (21 surfaces in the mesh); rhi null backend, conformance 11/11 cases, no GPU initialized; memory 3 pools, worst nominal, 0 suspected leaks; probes verified 98; result OK |
| `forge_first_generation` | **PASS** | 0.2 | build nexora-first-generation: 21 materials: 21 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (42 resources) |
| `headless_slice_textures` | **PASS** | 0.5 | queries 50 answered, 2 refused; hands broke 1, placed 1, refused 1 in its own body, stood 1.00 higher on its own block; content blocks 21 (21 surfaces in the mesh); content textures 21 (resolved, verified and decoded); rhi null backend, conformance 11/11 cases, no GPU initialized; rhi uploads 21 textures as RGBA8, 21504 bytes, one fence; memory 4 pools, worst nominal, 0 suspected leaks; probes verified 50; result OK |
| `rhi_native` | **PASS** | 1.5 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); conformance 11/11 cases on wgpu; upload 1024 bytes to a 16x16 texture, read back identical; draw 16 of 16 texels shaded by the GPU; bound 256 of 256 texels sampled, 256 kept by depth, 256 tinted by a uniform; cull 16 of 16 texels shaded counter-clockwise, 0 clockwise, culling back faces; result OK |
| `window` | **PASS** | 1.9 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 256x256 on win32, 0 redraws waited for it to take frames; surface 256x256 Bgra8UnormSrgb, Fifo; conformance 11/11 cases on wgpu, presenting; frame 65536 of 65536 window texels show the 16x16 target, read back from the surface; presented 61 frames, 60 of them by the probe; result OK |
| `client_mode` | **PASS** | 3.6 | adapter AMD Radeon RX 6650 XT (vulkan, discretegpu); window 384x256 on win32; first frame 97505 of 98304 pixels judged by the ray cast, 97505 matching, 0 snapped edges, 0 back faces, read back from the surface; frames 120 shown, ended by frame limit; frame loop 24 ticks, 0 discarded; 120 target, 0 warning, 0 critical, 0 emergency (budget: doubling from the 50 ms step, not measured); frame wall median 9.82 ms, p95 10.77 ms, max 42.45 ms; 0.29 ms unattributed in all; frame work median 1.18 ms, p95 1.71 ms, max 15.99 ms; presentation wait median 8.63 ms, p95 9.56 ms, max 26.58 ms; player walked 0.00 forward, 0.00 right, 0.00 up, grounded; hands broke 0, placed 0, refused 0, 0 clicks met nothing in reach; 0 columns meshed again, 0 uploaded; result OK |
| `input_devices` | **SKIPPED** | 0.0 | NEXORA_INPUT=none: no one will press a key |
| `benchmark_cpu` | **PASS** | 15.7 | see the report's benchmark section |

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
| benchmark binary size | 7.7 MiB |
| GPU adapter (RHI stage) | AMD Radeon RX 6650 XT (vulkan, discretegpu) |
| tool interpreter (tool-call stage) | Python 3.11.9 (cpython, via python3) |

### Measurements

| measurement | median | p95 | rel. σ | note |
| --- | ---: | ---: | ---: | --- |
| `startup.world_create` | 1.45 µs | 2.38 µs | 21.4% | Create a world, register the block set, freeze the registry and bring it online |
| `spatial.section_of_division` | 1.8 ns | 3.2 ns | 22.5% | Block → section using div_euclid by a runtime extent (what the engine does) |
| `spatial.section_of_shift_runtime` | 6.0 ns | 13.1 ns | 35.4% | Block → section by shift, width read at runtime: what DEBT-0005 would actually build |
| `spatial.section_of_shift_const` | 6.1 ns | 20.3 ns | 56.3% | Block → section by shift with a compile-time width: the optimization's ceiling |
| `spatial.index_of` | 5.7 ns | 10.1 ns | 21.0% | Flatten a local position into a storage index, bounds-checked |
| `voxel.get_paletted` | 9.1 ns | 11.8 ns | 17.6% | Read one cell from a palette-indexed section |
| `voxel.get_uniform` | 5.4 ns | 10.2 ns | 39.8% | Read one cell from a uniform section (solid rock, the common case) |
| `voxel.set_existing_state` | 19.4 ns | 20.8 ns | 22.2% | Write a cell whose state is already in the palette |
| `voxel.first_write_to_uniform` | 252.0 ns | 351.5 ns | 38.7% | Break a uniform section into a palette: the allocation the air optimization defers |
| `voxel.compact_section` | 183.45 µs | 240.20 µs | 24.9% | Rebuild a section's palette from what is actually referenced |
| `chunk.change_feed_append` | 43.5 ns | 51.5 ns | 12.1% | One journalled voxel write with the change feed below its cap |
| `chunk.change_feed_at_cap` | 47.0 ns | 72.8 ns | 66.7% | The same write with the feed full, so each one discards the oldest entry |
| `chunk.change_feed_cap` | 4 096 | — | — | Entries a chunk's change feed holds before it starts discarding |
| `worldgen.chunk_16` | 243.50 µs | 302.80 µs | 11.4% | Generate one chunk column, 16³ sections (the plan's size) |
| `worldgen.chunk_32` | 1.64 ms | 2.32 ms | 18.3% | Generate one chunk column, 32³ sections (engine default) |
| `worldgen.surface_height` | 42.2 ns | 45.9 ns | 10.3% | One column's terrain height from the position-seeded stream |
| `entity.spawn` | 303.8 ns | 511.8 ns | 33.2% | Create one entity: slot allocation, component writes, persistent id |
| `entity.spawn_despawn_cycle` | 393.8 ns | 618.4 ns | 20.0% | Spawn then destroy: the slot reuse path a busy world runs constantly |
| `entity.resolve_handle` | 3.8 ns | 4.2 ns | 5.6% | Validate one handle: world, range, generation and liveness |
| `entity.step_1000` | 7.86 µs | 8.34 µs | 6.9% | Advance 1,000 entities by one tick: the batch walk over dense columns |
| `entity.query_type_1000` | 32.97 µs | 45.26 µs | 22.6% | Scan of 1,000 entities filtering by type: no position, so no index |
| `entity.query_tag_1000` | 35.07 µs | 41.22 µs | 11.8% | Scan of 1,000 entities filtering by tag: no position, so no index |
| `entity.query_radius_1000` | 19.76 µs | 23.66 µs | 17.0% | Radius query in the plan's dense 1,000-entity box: the index cannot narrow it |
| `entity.query_radius_100k` | 1.36 µs | 3.15 µs | 40.1% | Radius query over 100,000 entities spread across a world, through the index |
| `entity.query_chunk_100k` | 982.5 ns | 1.01 µs | 2.2% | Which of 100,000 entities are in one chunk column, through the index |
| `entity.query_radius_candidates_100k` | 41 | — | — | Entities a 16-block radius query examines, of 100,000 in the store |
| `entity.index_cells_100k` | 24 642 | — | — | Grid cells holding at least one of the 100,000 entities |
| `entity.step_1000_crossing_cells` | 252.62 µs | 400.90 µs | 23.0% | Advance 1,000 entities that all change grid cell every tick: the index at its worst |
| `entity.save_1000` | 501 MiB/s | 477.56 µs | 27.6% | Serialize 1,000 entities as logical state |
| `entity.load_1000` | 126 MiB/s | 1.36 ms | 21.2% | Restore 1,000 entities, re-resolving every persistent id |
| `entity.save_size_1000` | 141.6 KiB | — | — | Bytes of save section for 1,000 entities |
| `physics.character_step` | 736.8 ns | 1.92 µs | 49.8% | One character substep on generated terrain: gravity, sweep, ground, friction |
| `physics.thousand_bodies_step` | 166.60 µs | 357.68 µs | 37.6% | One substep of 1,000 awake dynamic bodies against generated terrain |
| `physics.thousand_bodies_step_flat` | 171.05 µs | 187.95 µs | 6.1% | The same substep against a flat fixture: the solver without the world lookup |
| `physics.world_reads_per_step` | 1 000 | — | — | Cells the world is asked about in one substep of 1,000 settled awake bodies |
| `physics.voxel_lookup` | 23.8 ns | 85.0 ns | 73.6% | One cell question answered by the world view, section already resident |
| `physics.axis_sweep` | 50.7 ns | 82.5 ns | 35.7% | One axis of a swept box against the voxel grid (paired with the C++ reference) |
| `physics.box_sweep` | 237.6 ns | 349.3 ns | 21.8% | Resolve one box move on three axes against generated terrain |
| `physics.depenetration_check` | 113.1 ns | 277.2 ns | 49.2% | The per-body test for having started inside terrain, when it has not |
| `physics.raycast_40m` | 1.06 µs | 1.36 µs | 15.2% | Walk a 40 m ray down through generated terrain until it hits |
| `physics.timestep_accumulate` | 3.3 ns | 3.3 ns | 0.5% | Fold one world tick into the fixed-step accumulator |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — | Bodies asleep after ten seconds: what sleeping actually saves |
| `physics.thousand_sleeping_step` | 830.0 ns | 1.74 µs | 35.8% | One substep with the same 1,000 bodies asleep |
| `player.walk_route` | 31.00 µs | 32.60 µs | 3.4% | A player walks forward for one second of world time: 20 ticks of control and physics on generated terrain |
| `player.walk_route_cm` | 420 | — | — | How far that route walks the player, in centimetres |
| `streaming.idle_tick_r3` | 5.92 µs | 7.37 µs | 18.9% | A tick with nothing to do, 7x7 columns of interest: pure decision cost |
| `streaming.idle_tick_r12` | 141.94 µs | 157.42 µs | 10.8% | The same idle tick over 25x25 columns: how decision cost scales with radius |
| `streaming.walk_one_chunk` | 12.77 µs | 17.81 µs | 18.4% | A tick after the observer moves one column: decide, load the new ring, release the old |
| `streaming.fast_travel_r3` | 49.77 µs | 65.25 µs | 22.4% | Teleport and settle: release 49 columns and take 49 more, decision side only |
| `streaming.chunk_generate_cycle` | 1.54 ms | 1.85 ms | 14.6% | Activate a fresh column and drop it again: the cost of an unedited chunk |
| `streaming.chunk_retained_cycle` | 382.5 ns | 486.0 ns | 9.9% | Persist, drop and restore an edited column: retention instead of regeneration |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — | Memory an edited column occupies while it is evicted |
| `streaming.chunk_flushed_cycle` | 8.01 ms | 10.77 ms | 13.8% | Persist, flush to a region file, drop, and read the column back from disk |
| `streaming.retained_bytes_after_flush` | 0 B | — | — | Memory the same evicted column occupies once it is in its region file |
| `streaming.region_writes_per_eight_columns` | 4 | — | — | Region files a flush of eight columns touches, two columns to a region |
| `streaming.columns_of_interest_r12` | 625 | — | — | Columns a radius-12 observer makes the manager consider every tick |
| `jobs.submit_wait_roundtrip` | 1.88 µs | 6.70 µs | 84.3% | Submit one trivial job and block until it completes: the full scheduling round trip |
| `jobs.submit_only` | 201.5 ns | 473.2 ns | 47.5% | Enqueue a job without waiting: the cost a producer pays |
| `jobs.batch_1000_barrier` | 862.60 µs | 5.61 ms | 92.2% | Submit 1,000 jobs one at a time and barrier: a wake-up per job |
| `jobs.batch_1000_barrier_submit_all` | 765.10 µs | 806.60 µs | 7.6% | The same 1,000 jobs handed over as one batch: one wake-up for the wave |
| `jobs.submit_all_1000` | 109.60 µs | 393.70 µs | 68.8% | Hand over 1,000 jobs as one batch, without waiting: the producer's cost |
| `jobs.wakeups_per_1000_submitted` | 1 000 | — | — | Condvar wake-ups a wave of 1,000 jobs issues, submitted one at a time |
| `jobs.wakeups_per_1000_submit_all` | 8 | — | — | Condvar wake-ups the same wave issues through submit_all |
| `frame.schedule_advance` | 0.0 ns | 100.0 ns | 144.3% | The fixed-timestep accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | 100.0 ns | 200.0 ns | 40.8% | A whole frame opened, charged to one stage and closed, with no work in it |
| `frame.accounting_seven_stages` | 100.0 ns | 200.0 ns | 40.7% | The same frame with every stage in the sequence charged separately |
| `frame.stages` | 7 | — | — | Stages in the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | 6 | — | — | Of those, the ones a system in this repository can actually run in |
| `input.sample_idle` | 1.00 µs | 1.10 µs | 6.5% | One frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | 2.40 µs | 5.60 µs | 368.1% | One frame carrying a single key transition, resolved against the keymap |
| `input.sample_stick` | 2.50 µs | 2.90 µs | 8.7% | One frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | 300.0 ns | 400.0 ns | 135.0% | Checking a two-action snapshot that arrived from outside the trust boundary |
| `input.bindings` | 41 | — | — | Bindings the sampled keymap holds, across two contexts |
| `save.crc32_64kib` | 390.63 µs | 1.67 ms | 81.0% | CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference) |
| `save.encode` | 3 MiB/s | 17.97 ms | 10.5% | Serialize the whole world to bytes, including per-section checksums |
| `save.decode` | 18 MiB/s | 2.68 ms | 9.2% | Parse and verify every checksum on the way back in |
| `save.world_load` | 3.21 ms | 5.32 ms | 27.3% | Rebuild the world from a container, remapping every block through its identifier |
| `save.write_atomic_disk` | 1 MiB/s | 32.52 ms | 4.5% | Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006) |
| `save.region_write_all` | 53.87 ms | 68.40 ms | 11.6% | Write every region of the world: four region files and the header |
| `save.region_write_one_dirty` | 17.49 ms | 19.43 ms | 8.1% | Write after a single block edit: one region file and the header |
| `save.regions_written_per_edit` | 1 | — | — | Region files rewritten after one block changed, out of four |
| `save.region_bytes_all` | 41.7 KiB | — | — | Bytes across every region file when the whole world is written |
| `save.region_bytes_one_dirty` | 4.7 KiB | — | — | Bytes written after one block changed |
| `journal.append_unsynced` | 3.57 µs | 4.42 µs | 12.0% | Frame and checksum one edit record, without making it durable |
| `journal.append_durable` | 695.06 µs | 961.84 µs | 23.8% | One edit record, fsynced before returning: durability per edit |
| `journal.append_batched_sync` | 802.55 µs | 1.62 ms | 34.8% | One edit record when 64 share a single fsync |
| `journal.replay_10k_records` | 2.93 ms | 3.67 ms | 12.5% | Read back and verify a journal of 10,000 edit records |
| `journal.record_bytes` | 55 B | — | — | Encoded size of one block edit, framing included |
| `save.read_disk` | 20 MiB/s | 2.41 ms | 9.4% | Read and verify a save from disk |
| `save.size_9_chunks` | 40.8 KiB | — | — | Bytes on disk for a 3×3 chunk world with the full vertical range |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — | In-memory voxel storage for the same world |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — | Non-air blocks that storage represents |
| `mesh.region_16` | 2.24 ms | 3.31 ms | 22.8% | Cull and merge a 16 cubed region across the ground/air boundary |
| `mesh.region_32` | 23.10 ms | 29.71 ms | 13.3% | The same at 32 cubed, the engine's default section size |
| `mesh.cull_only_16` | 2.23 ms | 2.40 ms | 6.8% | Count visible faces without merging them: the culling half alone |
| `mesh.region_16_from_snapshot` | 230.72 µs | 302.67 µs | 15.1% | The same 16 cubed region, meshed from a pre-read dense array |
| `mesh.region_16_with_snapshot` | 756.64 µs | 862.79 µs | 10.3% | The same region, snapshot built and then meshed: what the snapshot costs in total |
| `mesh.cube_faces_16` | 24 576 | — | — | Faces a naive mesher would emit: every cell, all six sides |
| `mesh.visible_faces_16` | 2 471 | — | — | Faces left after culling |
| `mesh.quads_16` | 807 | — | — | Rectangles left after greedy merging |
| `mesh.vertices_16` | 3 228 | — | — | Vertices a renderer would upload, at four per rectangle |
| `camera.sample` | 100.0 ns | 200.0 ns | 53.8% | Resolve a camera 2^40 blocks out: view, reverse-Z projection, product, frustum |
| `camera.cull_columns_r12` | 4.60 µs | 4.60 µs | 2.3% | Test the 625 columns a radius-12 observer streams against the frustum |
| `camera.visible_columns_r12` | 261 | — | — | Of those 625 columns, the ones a 70-degree view looking down and ahead keeps |
| `rhi.device_open` | 349.30 ms | 465.85 ms | 16.7% | Find the adapter and open a device on it, with no surface: the RHI's startup |
| `rhi.fence_roundtrip` | 135.03 µs | 573.05 µs | 86.7% | Submit a list holding only a marker and wait for its fence: synchronization alone |
| `rhi.texture_create_destroy` | 4.09 µs | 4.43 µs | 3.4% | Create a 16x16 RGBA8 texture and destroy it with no work in flight |
| `rhi.upload_texture_16` | 5 MiB/s | 698.03 µs | 76.3% | Upload one 16x16 RGBA8 texture and wait for it: one write, one fence |
| `rhi.upload_first_generation` | 55 MiB/s | 1.48 ms | 90.2% | Upload sixteen 16x16 textures in one submission and one fence, as the slice does |
| `rhi.mesh_16_vertex_bytes` | 50.4 KiB | — | — | Vertex bytes for the meshed 16 cubed region, at 16 bytes a vertex (DEBT-0046) |
| `rhi.upload_mesh_16` | 220 MiB/s | 1.14 ms | 90.5% | Upload the meshed 16 cubed region's vertex buffer and wait for it |
| `rhi.draw_16` | 204.69 µs | 667.57 µs | 65.6% | Draw one full-target triangle into a 16x16 target and wait for it |
| `frame.draw_chunk_16` | 274.17 µs | 1.24 ms | 85.5% | One frame at 256x256: clear, camera, the meshed 16 cubed region with reverse-Z depth, one submission, one fence |
| `frame.chunk_16_vertices` | 7 377 | — | — | Vertices the frame draws: two triangles per merged rectangle of the region, more where a neighbouring corner splits an edge (ADR-0033) |
| `frame.pixels_judged` | 51 376 | — | — | Of the frame's 65,536 pixels, those checked against the CPU ray cast (all matched) |
| `window.first_frame` | 757.11 ms | 757.11 ms | — | Open the event loop, the window, a device and its surface, upload the chunk, and show the first frame: startup, once |
| `window.present_chunk_16` | 10.01 ms | 11.69 ms | 18.1% | Interval between frames reaching a 256x256 window: the meshed 16 cubed region drawn and presented (FIFO: a real display paces it to its refresh) |
| `window.present_wait_chunk_16` | 8.97 ms | 10.61 ms | 20.5% | Of each interval, the time present was blocked on the window system for a free surface image (the display's refresh, the GPU's earlier frames): not work (ADR-0017 amendment) |
| `window.pixels_judged` | 51 376 | — | — | Of the first shown frame's 65,536 pixels, read back from the surface, those checked against the CPU ray cast (all matched) |
| `ffi.scalar_inlined` | 1.7 ns | 6.3 ns | 64.6% | Mix one u64, inlined Rust: the work with no call at all |
| `ffi.scalar_opaque_rust` | 2.0 ns | 2.3 ns | 10.8% | The same mix behind a Rust call the optimizer will not inline |
| `ffi.bulk_inlined` | 3.95 µs | 4.01 µs | 1.7% | Hash 4 KiB, inlined Rust |
| `ffi.bulk_opaque_rust` | 3.90 µs | 4.03 µs | 2.1% | Hash 4 KiB behind a Rust call the optimizer will not inline |
| `tool.cold_call` | 97.46 ms | 101.21 ms | 2.0% | Start the Python tool, check its banner, one digest request answered and checked, exit: a tool run once |
| `tool.warm_roundtrip` | 188.80 µs | 192.99 µs | 4.9% | One digest request and its checked answer over a running tool's stdin/stdout (IPC); against tool.digest_in_rust, and ffi.empty_crossing at ~1.2 ns (Appendix D) |
| `tool.python_work` | 150.68 µs | 155.03 µs | 4.9% | Of each round trip, the tool's own time for the digest in Python, by its clock: the rest is the boundary |
| `tool.digest_in_rust` | 96.13 µs | 103.92 µs | 7.0% | The same digest in this process, in Rust: read the 16x16 PNG, FNV-1a 64 and its IHDR, no boundary crossed |
| `tool.payload_bytes` | 256 B | — | — | Bytes of the 16x16 RGBA8 PNG every call digests: the catalog's size column |

#### Published budgets

| measurement | owner | target | warning | critical | emergency | median | p95 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| `physics.thousand_bodies_step` | physics (`physics::budget`, DEBT-0013) | 250.00 µs | 500.00 µs | 1.00 ms | 2.00 ms | 166.60 µs · target | 357.68 µs · warning |

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
