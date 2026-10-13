# Timed atlas CPU resources

`lwe-scene` provides a separate `TimedAtlasSource` API for bounded animated-image resource work. It does not change `Texture::decode`, particle `Texture::decode_atlas`, renderer admission or the Scene compatibility policy. Workshop `3420277613` still requires its complete material, history, script, resource-property and text paths before playback can be admitted.

## Accepted resource shape

The source accepts the verified TEXV0005 / TEXB0004 / TEXS0003 shape: BC3, flags 7, one image and one top mip, known container/body framing, matching storage and image dimensions, integral rectangular frames without shear, and finite positive durations. It validates every frame, even when only one region is requested. Existing texture dimension, decoded-extent, source-size and aggregate inspection limits still apply; other versions and shapes remain unsupported. The opaque TEXI `image_info_extra` field is retained verbatim in metadata and the complete encoded hash; as with the existing CPU decoder, it does not affect pixel decoding. Its nonzero values in the two original resources do not establish renderer semantics.

Raw BC3 length must equal `ceil(width / 4) * ceil(height / 4) * 16`. A compressed mip's declared decompressed length is checked against that exact result before reserving or allocating LZ4 output. Decompression must produce exactly that length. Shared `TextureInspectionBudget` values bound aggregate mip and animation-frame inspection across resources.

`TimedAtlasSource::from_resource` reads the supplied resource descriptor once. Hashing, metadata parsing and payload decoding use the same retained encoded buffer. Replacing the path after opening the descriptor cannot substitute the new file. This is not a filesystem snapshot against concurrent writes to that same inode; callers requiring that guarantee must acquire a stable source first.

## Ownership and memory

Use one `AtlasMemoryBudget` for sources and decoded regions belonging to the same worker. Its maximum is 512 MiB. The ledger charges actual retained `Vec` capacities for encoded bytes, decompressed BC3 when needed, frame/mip metadata and RGBA regions, plus conservative reservations while parsing or decoding. Cloned source or region handles share their actual allocation and lease; independently read sources are charged independently even if their hashes match. The last owner releases both storage and accounting. There is no API to extract a retained `Vec` while dropping its lease.

`usage()` reports charged bytes, peak charges and whether a region decode is active. Only one region decode may be active in that shared budget. Allocation, framing, LZ4 and decode failures release their temporary leases and retain existing owners. A failed allocation may still increase the historical peak; it does not leave a live charge.

These are allocation-capacity and reservation counters, not total process RSS, allocator overhead, VM memory or GPU residency. The API is not wired into the engine: future integration must charge existing static textures, atlas sources and retained frames together, rather than allow independent 512 MiB pools to bypass the engine's existing limit.

## Region and time semantics

`Bc3RegionLayout::checked` validates the original atlas dimensions and requested `PixelRect`, including block row stride and checked bounds. The decoder reads intersecting BC3 blocks directly into the final region RGBA allocation, using a fixed block workspace. It creates neither a full-atlas RGBA buffer nor packed or padded region heap buffers. The legacy DDS path remains available for static/particle resources and as an independent test oracle.

`frame_neighborhood(index, radius)` includes real neighboring atlas pixels, clipped only at the outer atlas boundary. It does not invent transparency, repeat/clamp behavior or replicated edge pixels. `frame_at(elapsed)` selects half-open intervals using the author's original f32 durations promoted to f64 and their accumulated cycle. It rejects negative or non-finite time. The caller still owns the clock epoch, pause/resume, first-frame acceptance and transaction behavior.

CPU decoding does not establish shader UV orientation, half-texel offsets, native filtering or original-engine rounding. GPU uploads, frame residency, history/copy operations, stale-job rejection and rollback remain subsequent implementation gates. In particular, decoding one frame cannot establish the peak cost of current, candidate, rollback and staging resources.

## Verification

Ordinary crate tests cover nonuniform BC3 blocks, every RGB565 endpoint, both alpha interpolation branches, selector order, all block offsets, multiblock and partial-edge crops, complete-table validation, exact LZ4 sizes, timing boundaries, neighboring pixels, ownership and failure cleanup. They compare authored results with the retained `image` DDS decoder.

An ignored CPU-only oracle additionally compares all sixteen frames of the two unchanged original `3420277613` atlases and their one-pixel neighborhoods with full DDS reference crops. It requires the private original package and explicit `LWE_TIMED_ATLAS_ORACLE=1`, `LWE_TIMED_ATLAS_PROJECT`, optional `LWE_TIMED_ATLAS_ASSETS`, and optional `LWE_TIMED_ATLAS_ORACLE_REPORT`. Run the named test with `--ignored --exact --nocapture`; keep author assets outside Git. The report records original resource hashes, region hashes and capacities. Its deliberately large full-atlas reference allocation is test-only and reported separately from the production ledger.

That oracle is resource-level CPU evidence. It does not run GL, apply a wallpaper, write desktop state or establish Windows parity. Native desktop acceptance remains a separate opt-in procedure in the [contributor guide](./guide.md).

The 2026-10-13 original-resource run passed all sixteen frame crops and sixteen neighborhoods. A frame retained 23,328,000 RGBA bytes; the largest one-pixel neighborhood retained 23,366,896 bytes. The two source capacities, including encoded bytes, raw BC3 and metadata, were 65,754,173 and 65,751,588 bytes. With one source and one output at a time, the production ledger peaked at 89,121,153 bytes, including an 84-byte codec-workspace reservation. Each test-only whole-DDS reference separately retained 209,952,000 bytes. These measured CPU capacities do not predict the complete renderer's GPU budget.
