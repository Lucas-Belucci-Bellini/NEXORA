# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `0b7bcec5f97de788162654d151999b45eca39506` |
| branch | `main` |
| generated | 2026-09-25T02:51:29+00:00 |
| OS | Windows 10 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 1.0 | exit 0 |
| `tests` | **PASS** | 56.8 | 1138 passed, 0 failed |
| `headless_slice` | **PASS** | 0.7 | queries 76 answered, 2 refused; memory 2 pools, worst nominal, 0 suspected leaks; probes verified 76; result OK |
| `headless_slice_content` | **PASS** | 0.6 | queries 92 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); memory 2 pools, worst nominal, 0 suspected leaks; probes verified 92; result OK |
| `forge_first_generation` | **PASS** | 0.1 | build nexora-first-generation: 16 materials: 16 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (32 resources) |
| `headless_slice_textures` | **PASS** | 0.3 | queries 44 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); content textures 16 (resolved, verified and decoded); memory 3 pools, worst nominal, 0 suspected leaks; probes verified 44; result OK |
| `benchmark_cpu` | **PASS** | 4.8 | see the report's benchmark section |

## What this machine could not validate, and why

Not "not tested": the engine has no code for these yet, so no machine can.

| item | status | reason |
| --- | --- | --- |
| `window` | NOT_IMPLEMENTED | no window host exists in the engine |
| `rhi` | NOT_IMPLEMENTED | the RHI boundary is specified, not built (freeze checklist, Runtime) |
| `gpu_context` | NOT_IMPLEMENTED | needs the RHI |
| `swapchain` | NOT_IMPLEMENTED | needs the RHI and a window |
| `presentation` | NOT_IMPLEMENTED | needs a swapchain |
| `shaders` | NOT_IMPLEMENTED | no shader pipeline exists |
| `texture_upload` | NOT_IMPLEMENTED | textures decode on the CPU (ADR-0022); nothing uploads them |
| `rendering` | NOT_IMPLEMENTED | no renderer; meshes are built and checked as data (ADR-0012) |
| `input_devices` | NOT_IMPLEMENTED | no real device has produced a signal (DEBT-0043) |
| `client_mode` | NOT_IMPLEMENTED | the runtime starts headless only; client mode is Phase 1's exit |
| `benchmark_gpu_stages` | NOT_IMPLEMENTED | DEBT-0008: the plan's GPU stages have no implementation |

## CPU benchmark

### Environment

| | |
| --- | --- |
| logical CPUs | 12 |
| build profile | `release` |
| architecture | `x86_64` |
| peak resident memory | unavailable |
| benchmark binary size | 1.3 MiB |

### Measurements

| measurement | median | p95 | rel. σ | note |
| --- | ---: | ---: | ---: | --- |
| `startup.world_create` | 1.39 µs | 1.52 µs | 3.9% | Create a world, register the block set, freeze the registry and bring it online |
| `spatial.section_of_division` | 1.8 ns | 1.8 ns | 1.6% | Block → section using div_euclid by a runtime extent (what the engine does) |
| `spatial.section_of_shift_runtime` | 6.0 ns | 6.0 ns | 0.4% | Block → section by shift, width read at runtime: what DEBT-0005 would actually build |
| `spatial.section_of_shift_const` | 6.0 ns | 6.0 ns | 0.7% | Block → section by shift with a compile-time width: the optimization's ceiling |
| `spatial.index_of` | 4.0 ns | 4.0 ns | 0.6% | Flatten a local position into a storage index, bounds-checked |
| `voxel.get_paletted` | 5.7 ns | 5.9 ns | 1.3% | Read one cell from a palette-indexed section |
| `voxel.get_uniform` | 3.9 ns | 4.8 ns | 31.1% | Read one cell from a uniform section (solid rock, the common case) |
| `voxel.set_existing_state` | 10.8 ns | 11.0 ns | 0.9% | Write a cell whose state is already in the palette |
| `voxel.first_write_to_uniform` | 136.5 ns | 207.5 ns | 22.6% | Break a uniform section into a palette: the allocation the air optimization defers |
| `voxel.compact_section` | 127.23 µs | 133.99 µs | 2.4% | Rebuild a section's palette from what is actually referenced |
| `chunk.change_feed_append` | 30.6 ns | 33.7 ns | 15.8% | One journalled voxel write with the change feed below its cap |
| `chunk.change_feed_at_cap` | 31.8 ns | 33.8 ns | 2.3% | The same write with the feed full, so each one discards the oldest entry |
| `chunk.change_feed_cap` | 4 096 | — | — | Entries a chunk's change feed holds before it starts discarding |
| `worldgen.chunk_16` | 189.60 µs | 217.10 µs | 6.1% | Generate one chunk column, 16³ sections (the plan's size) |
| `worldgen.chunk_32` | 1.11 ms | 1.13 ms | 1.5% | Generate one chunk column, 32³ sections (engine default) |
| `worldgen.surface_height` | 32.2 ns | 33.1 ns | 1.0% | One column's terrain height from the position-seeded stream |
| `entity.spawn` | 210.6 ns | 386.8 ns | 39.5% | Create one entity: slot allocation, component writes, persistent id |
| `entity.spawn_despawn_cycle` | 245.6 ns | 271.8 ns | 3.9% | Spawn then destroy: the slot reuse path a busy world runs constantly |
| `entity.resolve_handle` | 2.1 ns | 2.1 ns | 0.1% | Validate one handle: world, range, generation and liveness |
| `entity.step_1000` | 4.75 µs | 5.95 µs | 9.3% | Advance 1,000 entities by one tick: the batch walk over dense columns |
| `entity.query_type_1000` | 15.98 µs | 16.34 µs | 0.9% | Scan of 1,000 entities filtering by type: no position, so no index |
| `entity.query_tag_1000` | 19.32 µs | 20.01 µs | 1.6% | Scan of 1,000 entities filtering by tag: no position, so no index |
| `entity.query_radius_1000` | 10.63 µs | 10.76 µs | 0.6% | Radius query in the plan's dense 1,000-entity box: the index cannot narrow it |
| `entity.query_radius_100k` | 687.5 ns | 707.5 ns | 1.2% | Radius query over 100,000 entities spread across a world, through the index |
| `entity.query_chunk_100k` | 556.0 ns | 576.0 ns | 1.5% | Which of 100,000 entities are in one chunk column, through the index |
| `entity.query_radius_candidates_100k` | 41 | — | — | Entities a 16-block radius query examines, of 100,000 in the store |
| `entity.index_cells_100k` | 24 642 | — | — | Grid cells holding at least one of the 100,000 entities |
| `entity.step_1000_crossing_cells` | 154.05 µs | 158.54 µs | 1.8% | Advance 1,000 entities that all change grid cell every tick: the index at its worst |
| `entity.save_1000` | 578 MiB/s | 247.29 µs | 1.7% | Serialize 1,000 entities as logical state |
| `entity.load_1000` | 263 MiB/s | 595.33 µs | 5.8% | Restore 1,000 entities, re-resolving every persistent id |
| `entity.save_size_1000` | 141.6 KiB | — | — | Bytes of save section for 1,000 entities |
| `physics.character_step` | 370.4 ns | 373.9 ns | 0.5% | One character substep on generated terrain: gravity, sweep, ground, friction |
| `physics.thousand_bodies_step` | 84.62 µs | 119.40 µs | 14.6% | One substep of 1,000 awake dynamic bodies against generated terrain |
| `physics.thousand_bodies_step_flat` | 80.62 µs | 92.42 µs | 6.4% | The same substep against a flat fixture: the solver without the world lookup |
| `physics.world_reads_per_step` | 1 000 | — | — | Cells the world is asked about in one substep of 1,000 settled awake bodies |
| `physics.voxel_lookup` | 12.3 ns | 12.5 ns | 1.0% | One cell question answered by the world view, section already resident |
| `physics.axis_sweep` | 29.1 ns | 31.8 ns | 3.5% | One axis of a swept box against the voxel grid (paired with the C++ reference) |
| `physics.box_sweep` | 144.6 ns | 149.2 ns | 1.5% | Resolve one box move on three axes against generated terrain |
| `physics.depenetration_check` | 48.6 ns | 49.3 ns | 0.6% | The per-body test for having started inside terrain, when it has not |
| `physics.raycast_40m` | 839.8 ns | 863.4 ns | 1.1% | Walk a 40 m ray down through generated terrain until it hits |
| `physics.timestep_accumulate` | 3.3 ns | 3.3 ns | 0.1% | Fold one world tick into the fixed-step accumulator |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — | Bodies asleep after ten seconds: what sleeping actually saves |
| `physics.thousand_sleeping_step` | 850.0 ns | 855.0 ns | 0.3% | One substep with the same 1,000 bodies asleep |
| `streaming.idle_tick_r3` | 3.16 µs | 3.56 µs | 4.7% | A tick with nothing to do, 7x7 columns of interest: pure decision cost |
| `streaming.idle_tick_r12` | 83.44 µs | 87.24 µs | 2.5% | The same idle tick over 25x25 columns: how decision cost scales with radius |
| `streaming.walk_one_chunk` | 6.09 µs | 6.12 µs | 1.0% | A tick after the observer moves one column: decide, load the new ring, release the old |
| `streaming.fast_travel_r3` | 31.31 µs | 34.79 µs | 4.4% | Teleport and settle: release 49 columns and take 49 more, decision side only |
| `streaming.chunk_generate_cycle` | 1.02 ms | 1.06 ms | 1.5% | Activate a fresh column and drop it again: the cost of an unedited chunk |
| `streaming.chunk_retained_cycle` | 192.0 ns | 256.0 ns | 12.0% | Persist, drop and restore an edited column: retention instead of regeneration |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — | Memory an edited column occupies while it is evicted |
| `streaming.chunk_flushed_cycle` | 5.47 ms | 6.58 ms | 8.4% | Persist, flush to a region file, drop, and read the column back from disk |
| `streaming.retained_bytes_after_flush` | 0 B | — | — | Memory the same evicted column occupies once it is in its region file |
| `streaming.region_writes_per_eight_columns` | 4 | — | — | Region files a flush of eight columns touches, two columns to a region |
| `streaming.columns_of_interest_r12` | 625 | — | — | Columns a radius-12 observer makes the manager consider every tick |
| `jobs.submit_wait_roundtrip` | 7.82 µs | 9.17 µs | 13.0% | Submit one trivial job and block until it completes: the full scheduling round trip |
| `jobs.submit_only` | 7.78 µs | 8.81 µs | 16.8% | Enqueue a job without waiting: the cost a producer pays |
| `jobs.batch_1000_barrier` | 6.67 ms | 10.48 ms | 35.7% | Submit 1,000 jobs one at a time and barrier: a wake-up per job |
| `jobs.batch_1000_barrier_submit_all` | 790.20 µs | 804.30 µs | 2.6% | The same 1,000 jobs handed over as one batch: one wake-up for the wave |
| `jobs.submit_all_1000` | 129.10 µs | 235.80 µs | 42.6% | Hand over 1,000 jobs as one batch, without waiting: the producer's cost |
| `jobs.wakeups_per_1000_submitted` | 1 000 | — | — | Condvar wake-ups a wave of 1,000 jobs issues, submitted one at a time |
| `jobs.wakeups_per_1000_submit_all` | 8 | — | — | Condvar wake-ups the same wave issues through submit_all |
| `frame.schedule_advance` | 0.0 ns | 100.0 ns | 197.7% | The fixed-timestep accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | 0.0 ns | 0.0 ns | 500.0% | A whole frame opened, charged to one stage and closed, with no work in it |
| `frame.accounting_seven_stages` | 100.0 ns | 100.0 ns | 63.6% | The same frame with every stage in the sequence charged separately |
| `frame.stages` | 7 | — | — | Stages in the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | 4 | — | — | Of those, the ones a system in this repository can actually run in |
| `input.sample_idle` | 600.0 ns | 600.0 ns | 57.0% | One frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | 1.20 µs | 1.50 µs | 22.7% | One frame carrying a single key transition, resolved against the keymap |
| `input.sample_stick` | 1.50 µs | 1.60 µs | 2.9% | One frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | 100.0 ns | 200.0 ns | 35.2% | Checking a two-action snapshot that arrived from outside the trust boundary |
| `input.bindings` | 41 | — | — | Bindings the sampled keymap holds, across two contexts |
| `save.crc32_64kib` | 263.67 µs | 286.49 µs | 3.3% | CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference) |
| `save.encode` | 5 MiB/s | 8.17 ms | 2.5% | Serialize the whole world to bytes, including per-section checksums |
| `save.decode` | 28 MiB/s | 1.49 ms | 3.0% | Parse and verify every checksum on the way back in |
| `save.world_load` | 1.78 ms | 1.84 ms | 1.3% | Rebuild the world from a container, remapping every block through its identifier |
| `save.write_atomic_disk` | 2 MiB/s | 23.88 ms | 13.1% | Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006) |
| `save.region_write_all` | 41.05 ms | 45.28 ms | 4.4% | Write every region of the world: four region files and the header |
| `save.region_write_one_dirty` | 9.03 ms | 13.55 ms | 25.2% | Write after a single block edit: one region file and the header |
| `save.regions_written_per_edit` | 1 | — | — | Region files rewritten after one block changed, out of four |
| `save.region_bytes_all` | 41.7 KiB | — | — | Bytes across every region file when the whole world is written |
| `save.region_bytes_one_dirty` | 4.7 KiB | — | — | Bytes written after one block changed |
| `journal.append_unsynced` | 2.81 µs | 3.13 µs | 4.6% | Frame and checksum one edit record, without making it durable |
| `journal.append_durable` | 544.95 µs | 942.89 µs | 29.4% | One edit record, fsynced before returning: durability per edit |
| `journal.append_batched_sync` | 723.68 µs | 1.35 ms | 29.8% | One edit record when 64 share a single fsync |
| `journal.replay_10k_records` | 2.73 ms | 3.01 ms | 5.5% | Read back and verify a journal of 10,000 edit records |
| `journal.record_bytes` | 55 B | — | — | Encoded size of one block edit, framing included |
| `save.read_disk` | 27 MiB/s | 1.65 ms | 4.0% | Read and verify a save from disk |
| `save.size_9_chunks` | 40.8 KiB | — | — | Bytes on disk for a 3×3 chunk world with the full vertical range |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — | In-memory voxel storage for the same world |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — | Non-air blocks that storage represents |
| `mesh.region_16` | 1.58 ms | 1.60 ms | 0.7% | Cull and merge a 16 cubed region across the ground/air boundary |
| `mesh.region_32` | 12.13 ms | 12.29 ms | 0.7% | The same at 32 cubed, the engine's default section size |
| `mesh.cull_only_16` | 1.57 ms | 1.59 ms | 1.0% | Count visible faces without merging them: the culling half alone |
| `mesh.region_16_from_snapshot` | 149.84 µs | 162.55 µs | 3.2% | The same 16 cubed region, meshed from a pre-read dense array |
| `mesh.region_16_with_snapshot` | 556.93 µs | 616.55 µs | 4.0% | The same region, snapshot built and then meshed: what the snapshot costs in total |
| `mesh.cube_faces_16` | 24 576 | — | — | Faces a naive mesher would emit: every cell, all six sides |
| `mesh.visible_faces_16` | 2 471 | — | — | Faces left after culling |
| `mesh.quads_16` | 807 | — | — | Rectangles left after greedy merging |
| `mesh.vertices_16` | 3 228 | — | — | Vertices a renderer would upload, at four per rectangle |
| `ffi.scalar_inlined` | 1.6 ns | 1.8 ns | 3.8% | Mix one u64, inlined Rust: the work with no call at all |
| `ffi.scalar_opaque_rust` | 1.6 ns | 1.7 ns | 1.2% | The same mix behind a Rust call the optimizer will not inline |
| `ffi.bulk_inlined` | 3.85 µs | 3.91 µs | 2.8% | Hash 4 KiB, inlined Rust |
| `ffi.bulk_opaque_rust` | 3.84 µs | 3.88 µs | 1.3% | Hash 4 KiB behind a Rust call the optimizer will not inline |

#### Not measured

| stage the plan asks for | why there is no number |
| --- | --- |
| window | no windowing layer; ADR-0005 |
| input | meaningless without a window |
| RHI | boundary specified, no backend implemented |
| camera | depends on the RHI |
| mod boundary | mod runtime not implemented; Phase 7 |
| frame time | no render loop exists to time |
| incremental build | measured by the build system, not by this process |
| debugging / tooling effort | qualitative; the plan scores it separately from timing |
| FFI overhead | built without the `cpp` feature; no boundary is linked in |

## Verdict

Every executed check passed.
