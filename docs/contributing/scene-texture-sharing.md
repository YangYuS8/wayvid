# Scene texture ownership and budgets

Scene remains experimental and assessed against its bounded feature subset. This
change shares readonly uploads within one live EGL context; it does not add
Scene formats, effects, animation types, or Workshop item compatibility.

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

CPU decoded storage has a separate conservative 512 MiB retained-allocation
gate. It sums actual RGBA `Vec` capacities, deduplicated by `PreparedScene` Arc
identity across sessions and retained request/rollback sources. Independently
prepared equal-content vectors count separately. Thus GPU sharing does not
silently relax the earlier decoded-memory bound. Existing per-preparation,
resource-read, script, and audio limits also remain. This number covers retained
decoded RGBA, not total process RSS, decoder scratch, or driver VRAM.

## Diagnostics and verification

`EngineStatus.scene_gpu_budget` reports context generation, live immutable
bytes, private/reserved bytes, immutable reservations, retired bytes, the
current logical plan bound, allocation count, and lease count. The bound is a
snapshot, not a measured VRAM value or historical high-water mark.
`scene_decoded_texture_bytes` reports retained decoded RGBA capacities.
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
