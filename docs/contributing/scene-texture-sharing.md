# Scene texture ownership and budgets

Scene remains experimental and assessed against its bounded feature subset. The
initial sharing foundation supplies readonly uploads within one live EGL context.
Current image, timed-frame and history integration is described in
[Scene images, timed frames and history](./scene-miku-runtime.md).

## Identity and sampling

Preparation computes SHA-256 identities on the resource worker from the exact
bounded descriptor bytes that were decoded, decoded RGBA, the complete texture
metadata and animation bit patterns, source origin, canonical resource root,
normalized path, original binding, atlas mode, and decode contract version.
Replacing a same-size file at the same path cannot reuse its earlier pixels.
No resource I/O, decoding, or production fingerprinting occurs on the engine
thread.

One engine owns an `Rc<TexturePool>`. Renderers hold `Rc` leases and the pool
indexes only `Weak` references. A key also includes the fixed sampling variant:
linear/repeat and linear/clamp use different GL texture objects. Rendering never
changes the sampler state of shared uploads. This works with the existing
OpenGL 3.0 requirement and does not require OpenGL 3.3 sampler objects.

The engine binds the domain to its actual EGL context and an owner lifetime
token. A different current context, destroyed context owner, closed domain, or
ambiguous retirement failure rejects subsequent texture operations. A fresh
engine/context gets a fresh domain; a recycled native handle cannot validate an
old owner. Standalone `PreparedScene::initialize` keeps its existing contract:
the caller holds the context current through renderer cleanup and destruction.

## Allocation and transactions

The 512 MiB logical GPU budget is unchanged. Before candidate upload, the engine
unions readonly keys for every retained current, candidate, rollback, and
uninitialized output reservation. It charges each immutable sampling variant
once, every generation's private resources separately, and retired uploads until
GL deletion succeeds. Checked arithmetic and conflicting allocation sizes fail
closed. A rejected plan does not allocate a candidate or evict the current one.

Scratch targets, named targets, complete-frame targets, bloom, geometry, scripts,
audio, and accepted state remain private. Resizing retains the old private
layout and charges the full replacement layout until allocation succeeds.
Sharing preserves first-frame acknowledgement, commit, atomic assignment save,
finalization, cancellation, and rollback ordering.

Last-lease release only queues the GL object for retirement. The owning current
context drains it during renderer cleanup. A successful delete removes its
charge; an ambiguous GL failure quarantines the domain rather than retrying an
old numeric object ID. Closing with live leases reports failure and retains the
allocation accounting until context teardown.

CPU decoded resources use a shared 512 MiB allocation/reservation gate. It charges
actual retained static RGBA, encoded/BC3 sources, regions and imported images,
plus conservative decoder/font work reservations. Source bytes are revalidated
before weak-cache reuse; only actual shared owner identities are deduplicated.
Independently allocated equal-content owners count separately. Per-preparation,
resource-read, script and audio limits also remain. This is a logical working-set
policy, not total process RSS, exact third-party allocator use or driver VRAM.

## Diagnostics and verification

`EngineStatus.scene_gpu_budget` reports context generation, resident pool bytes,
private/reserved bytes, immutable reservations, retired bytes, the current
logical plan bound, allocation count, and lease count. The legacy
`immutable_resident_bytes` pool view includes retained timed-frame uploads.
Their mutable work is already charged once in `plan.work`, so that view overlaps
`private_and_reserved_bytes`: do not add those two diagnostic fields.
`logical_peak_bytes` is the current plan union, not measured VRAM or a historical
high-water mark.
`scene_decoded_texture_bytes` reports retained prepared CPU source/font owner
charges; it is not a total of all in-flight worker allocations.
Per-output `scene_payload_bytes` includes its referenced immutable union and
private generations; summing outputs can count the same immutable upload twice.

CPU regressions cover source replacement and provenance, metadata/animation,
sampler planning, real vector capacities, union accounting, overflow, and
retained resize peaks. Tiny authored offscreen regressions cover shared IDs and
pixels, isolated sampler variants, changed pixels, failed initialization,
preflight rejection, independent release, retirement with the wrong current
context, and fresh context domains.

Run the authored sharing fixtures explicitly, without applying desktop state:

```bash
EGL_PLATFORM=surfaceless LIBGL_ALWAYS_SOFTWARE=1 LP_NUM_THREADS=2 CARGO_BUILD_JOBS=1 \
  cargo test -p lwe-engine scene::renderer::sharing_tests::offscreen_ \
  -- --ignored --test-threads=1
```

An offscreen software renderer validates GL ownership and pixel behavior. It
does not establish native GPU memory use, visible Wayland playback, broad Scene
support, suspend/resume, or compatibility of a larger scene that still exceeds
the retained CPU or private GPU budgets. Keep the original long-running
desktop acceptance source and process unchanged when testing this branch.
